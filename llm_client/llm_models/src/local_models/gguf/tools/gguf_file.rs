//! Support for the GGUF file format.
//!
//! Spec: https://github.com/philpax/ggml/blob/gguf-spec/docs/gguf.md
//! Adapted from: https://github.com/huggingface/candle/blob/main/candle-core/src/quantized/gguf_file.rs

use super::gguf_tensors::{GgmlDType, TensorInfo};
use crate::Error; // Import local Error type
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_ALIGNMENT: u32 = 32;

pub struct GgufFile {
    pub magic: VersionedMagic,
    pub metadata: HashMap<String, Value>,
    pub tensors: Vec<TensorInfo>,
    pub tensor_data_offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Magic {
    Gguf,
}

impl TryFrom<u32> for Magic {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Error> {
        match value {
            0x46554747 | 0x47475546 => Ok(Self::Gguf), // "GGUF" LE or BE
            _ => Err(Error::Gguf(format!("Unknown GGUF magic number: 0x{value:08x}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionedMagic {
    GgufV1,
    GgufV2,
    GgufV3,
}

impl VersionedMagic {
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, Error> {
        let magic_int = reader.read_u32::<LittleEndian>()?;
        let magic = Magic::try_from(magic_int)?;
        let version = reader.read_u32::<LittleEndian>()?;
        match (magic, version) {
            (Magic::Gguf, 1) => Ok(Self::GgufV1),
            (Magic::Gguf, 2) => Ok(Self::GgufV2),
            (Magic::Gguf, 3) => Ok(Self::GgufV3),
            _ => Err(Error::Gguf(format!(
                "Unsupported GGUF magic/version: {magic:?}/v{version}"
            ))),
        }
    }
}

impl GgufFile {
    pub fn read<R: std::io::Seek + std::io::Read>(reader: &mut R) -> Result<Self, Error> {
        let magic = VersionedMagic::read(reader)?;

        let tensor_count: u64 = match magic {
            VersionedMagic::GgufV1 => reader.read_u32::<LittleEndian>()? as u64,
            VersionedMagic::GgufV2 | VersionedMagic::GgufV3 => reader.read_u64::<LittleEndian>()?,
        };
        let metadata_kv_count: u64 = match magic {
            VersionedMagic::GgufV1 => reader.read_u32::<LittleEndian>()? as u64,
            VersionedMagic::GgufV2 | VersionedMagic::GgufV3 => reader.read_u64::<LittleEndian>()?,
        };

        let mut metadata = HashMap::with_capacity(metadata_kv_count as usize);
        for _ in 0..metadata_kv_count {
            let key = read_string(reader, &magic)?;
            let value_type_u32 = reader.read_u32::<LittleEndian>()?;
            let value_type = ValueType::from_u32(value_type_u32)?;
            let value = Value::read(reader, value_type, &magic)?;
            metadata.insert(key, value);
        }

        let mut tensors = Vec::with_capacity(tensor_count as usize);
        for _ in 0..tensor_count {
            let name = read_string(reader, &magic)?;
            let n_dimensions = reader.read_u32::<LittleEndian>()? as usize;

            let mut shape: Vec<u64> = vec![0; n_dimensions];
            match magic {
                VersionedMagic::GgufV1 => {
                    let mut dims32 = vec![0u32; n_dimensions];
                    reader.read_u32_into::<LittleEndian>(&mut dims32)?;
                    for (i, dim) in dims32.into_iter().enumerate() {
                        shape[i] = dim as u64;
                    }
                }
                VersionedMagic::GgufV2 | VersionedMagic::GgufV3 => {
                    reader.read_u64_into::<LittleEndian>(&mut shape)?;
                }
            };
            shape.reverse(); // GGUF dimensions are stored in reverse order.

            let ggml_dtype_u32 = reader.read_u32::<LittleEndian>()?;
            let ggml_dtype = GgmlDType::from_u32(ggml_dtype_u32)?;

            let offset = reader.read_u64::<LittleEndian>()?;
            tensors.push(TensorInfo {
                name,
                shape: shape.into_iter().map(|d| d as usize).collect(), // Convert shape to usize
                offset: offset as usize, // Convert offset to usize
                ggml_dtype,
            });
        }
        let tensor_data_offset = reader.stream_position()?;
        let alignment = match metadata.get("general.alignment") {
            Some(Value::U8(v)) => *v as u32,
            Some(Value::U16(v)) => *v as u32,
            Some(Value::U32(v)) => *v as u32,
            Some(Value::I8(v)) if *v >= 0 => *v as u32,
            Some(Value::I16(v)) if *v >= 0 => *v as u32,
            Some(Value::I32(v)) if *v >= 0 => *v as u32,
            _ => DEFAULT_ALIGNMENT,
        };
        metadata.insert("general.alignment".to_string(), Value::U32(alignment)); // Ensure alignment is stored
        let alignment = alignment as u64;
        // Pad to alignment
        let tensor_data_offset = (tensor_data_offset + alignment - 1) / alignment * alignment;
        Ok(Self {
            magic,
            metadata,
            tensors, // Use the renamed variable
            tensor_data_offset,
        })
    }

    pub fn get_value<T: FromValue>(&self, key: &str) -> Result<T, Error> {
        self.metadata
            .get(key)
            .map_or_else(|| T::from_none(key), T::from_value)
    }

    pub fn get_pathed_value<T: FromValue>(
        &self,
        path_prefixes: &[&str],
        field_name: &str,
    ) -> Result<T, Error> {
        let prop_key = if path_prefixes.is_empty() {
            field_name.to_string()
        } else {
            let prefix = path_prefixes.join(".");
            format!("{}.{}", prefix, field_name)
        };
        self.get_value(&prop_key)
    }

    pub fn size(&self) -> usize {
        self.tensors.iter().map(|t| t.size()).sum()
    }
}

fn read_string<R: std::io::Read>(reader: &mut R, magic: &VersionedMagic) -> Result<String, Error> {
    let len: u64 = match magic {
        VersionedMagic::GgufV1 => reader.read_u32::<LittleEndian>()? as u64,
        VersionedMagic::GgufV2 | VersionedMagic::GgufV3 => reader.read_u64::<LittleEndian>()?,
    };
    let len = usize::try_from(len).map_err(|_| {
        Error::Gguf(format!("String length {len} exceeds usize capacity"))
    })?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    // GGUF strings are supposed to be non-null terminated but sometimes are.
    while buf.last() == Some(&0) {
        buf.pop();
    }
    String::from_utf8(buf).map_err(Error::Utf8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    // The value is a 8-bit unsigned integer.
    U8,
    // The value is a 8-bit signed integer.
    I8,
    // The value is a 16-bit unsigned little-endian integer.
    U16,
    // The value is a 16-bit signed little-endian integer.
    I16,
    // The value is a 32-bit unsigned little-endian integer.
    U32,
    // The value is a 32-bit signed little-endian integer.
    I32,
    // The value is a 64-bit unsigned little-endian integer.
    U64,
    // The value is a 64-bit signed little-endian integer.
    I64,
    // The value is a 32-bit IEEE754 floating point number.
    F32,
    // The value is a 64-bit IEEE754 floating point number.
    F64,
    // The value is a boolean.
    // 1-byte value where 0 is false and 1 is true.
    // Anything else is invalid, and should be treated as either the model being invalid or the reader being buggy.
    Bool,
    // The value is a UTF-8 non-null-terminated string, with length prepended.
    String,
    // The value is an array of other values, with the length and type prepended.
    // Arrays can be nested, and the length of the array is the number of elements in the array, not the number of bytes.
    Array,
}

impl ValueType {
    fn from_u32(v: u32) -> Result<Self, Error> {
        match v {
            0 => Ok(Self::U8),
            1 => Ok(Self::I8),
            2 => Ok(Self::U16),
            3 => Ok(Self::I16),
            4 => Ok(Self::U32),
            5 => Ok(Self::I32),
            6 => Ok(Self::F32),
            7 => Ok(Self::Bool),
            8 => Ok(Self::String),
            9 => Ok(Self::Array),
            10 => Ok(Self::U64),
            11 => Ok(Self::I64),
            12 => Ok(Self::F64),
            _ => Err(Error::Gguf(format!("Unrecognized GGUF value type: {v}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::U8(_) => ValueType::U8,
            Self::I8(_) => ValueType::I8,
            Self::U16(_) => ValueType::U16,
            Self::I16(_) => ValueType::I16,
            Self::U32(_) => ValueType::U32,
            Self::I32(_) => ValueType::I32,
            Self::U64(_) => ValueType::U64,
            Self::I64(_) => ValueType::I64,
            Self::F32(_) => ValueType::F32,
            Self::F64(_) => ValueType::F64,
            Self::Bool(_) => ValueType::Bool,
            Self::String(_) => ValueType::String,
            Self::Array(_) => ValueType::Array,
        }
    }

    fn type_error(expected: &str, found: &Value) -> Error {
        Error::Gguf(format!("Expected GGUF value type {expected}, found {found:?}"))
    }

    pub fn to_u8(&self) -> Result<u8, Error> {
        match self {
            Self::U8(v) => Ok(*v),
            v => Err(Self::type_error("U8", v)),
        }
    }

    pub fn to_i8(&self) -> Result<i8, Error> {
        match self {
            Self::I8(v) => Ok(*v),
            v => Err(Self::type_error("I8", v)),
        }
    }

    pub fn to_u16(&self) -> Result<u16, Error> {
        match self {
            Self::U16(v) => Ok(*v),
            v => Err(Self::type_error("U16", v)),
        }
    }

    pub fn to_i16(&self) -> Result<i16, Error> {
        match self {
            Self::I16(v) => Ok(*v),
            v => Err(Self::type_error("I16", v)),
        }
    }

    pub fn to_u32(&self) -> Result<u32, Error> {
        match self {
            Self::U32(v) => Ok(*v),
            v => Err(Self::type_error("U32", v)),
        }
    }

    pub fn to_i32(&self) -> Result<i32, Error> {
        match self {
            Self::I32(v) => Ok(*v),
            v => Err(Self::type_error("I32", v)),
        }
    }

    /// This will also automatically upcast any integral types which will not truncate.
    pub fn to_u64(&self) -> Result<u64, Error> {
        match self {
            Self::U64(v) => Ok(*v),
            // Autoupcast cases
            Self::U8(v) => Ok(*v as u64),
            Self::U16(v) => Ok(*v as u64),
            Self::U32(v) => Ok(*v as u64),
            Self::Bool(v) => Ok(*v as u64),
            v => Err(Self::type_error("U64 (or upcastable)", v)),
        }
    }

    pub fn to_i64(&self) -> Result<i64, Error> {
        match self {
            Self::I64(v) => Ok(*v),
            v => Err(Self::type_error("I64", v)),
        }
    }

    pub fn to_f32(&self) -> Result<f32, Error> {
        match self {
            Self::F32(v) => Ok(*v),
            v => Err(Self::type_error("F32", v)),
        }
    }

    pub fn to_f64(&self) -> Result<f64, Error> {
        match self {
            Self::F64(v) => Ok(*v),
            v => Err(Self::type_error("F64", v)),
        }
    }

    pub fn to_bool(&self) -> Result<bool, Error> {
        match self {
            Self::Bool(v) => Ok(*v),
            v => Err(Self::type_error("Bool", v)),
        }
    }

    pub fn to_vec(&self) -> Result<&Vec<Value>, Error> {
        match self {
            Self::Array(v) => Ok(v),
            v => Err(Self::type_error("Array", v)),
        }
    }

    pub fn to_string(&self) -> Result<&String, Error> {
        match self {
            Self::String(v) => Ok(v),
            v => Err(Self::type_error("String", v)),
        }
    }

    fn read<R: std::io::Read>(
        reader: &mut R,
        value_type: ValueType,
        magic: &VersionedMagic,
    ) -> Result<Self, Error> {
        match value_type {
            ValueType::U8 => Ok(Self::U8(reader.read_u8()?)),
            ValueType::I8 => Ok(Self::I8(reader.read_i8()?)),
            ValueType::U16 => Ok(Self::U16(reader.read_u16::<LittleEndian>()?)),
            ValueType::I16 => Ok(Self::I16(reader.read_i16::<LittleEndian>()?)),
            ValueType::U32 => Ok(Self::U32(reader.read_u32::<LittleEndian>()?)),
            ValueType::I32 => Ok(Self::I32(reader.read_i32::<LittleEndian>()?)),
            ValueType::U64 => Ok(Self::U64(reader.read_u64::<LittleEndian>()?)),
            ValueType::I64 => Ok(Self::I64(reader.read_i64::<LittleEndian>()?)),
            ValueType::F32 => Ok(Self::F32(reader.read_f32::<LittleEndian>()?)),
            ValueType::F64 => Ok(Self::F64(reader.read_f64::<LittleEndian>()?)),
            ValueType::Bool => match reader.read_u8()? {
                0 => Ok(Self::Bool(false)),
                1 => Ok(Self::Bool(true)),
                b => Err(Error::Gguf(format!("Invalid boolean value in GGUF: {b}"))),
            },
            ValueType::String => Ok(Self::String(read_string(reader, magic)?)),
            ValueType::Array => {
                let element_type_u32 = reader.read_u32::<LittleEndian>()?;
                let element_type = ValueType::from_u32(element_type_u32)?;
                let len: u64 = match magic {
                    VersionedMagic::GgufV1 => reader.read_u32::<LittleEndian>()? as u64,
                    VersionedMagic::GgufV2 | VersionedMagic::GgufV3 => {
                        reader.read_u64::<LittleEndian>()?
                    }
                };
                let len = usize::try_from(len).map_err(|_| {
                    Error::Gguf(format!("Array length {len} exceeds usize capacity"))
                })?;
                let mut vs = Vec::with_capacity(len);
                for _ in 0..len {
                    vs.push(Value::read(reader, element_type, magic)?)
                }
                Ok(Self::Array(vs))
            }
        }
    }
}

/// Trait for converting a GGUF `Value` into a Rust type.
pub trait FromValue: Sized {
    /// Attempt conversion from a `Value`.
    fn from_value(value: &Value) -> Result<Self, Error>;

    /// Handle the case where the metadata key is missing.
    /// Default implementation returns an error.
    fn from_none(key: &str) -> Result<Self, Error> {
        Err(Error::Gguf(format!("Missing required metadata key: {key}")))
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value.to_string().cloned() // Cloned because to_string returns &String
    }
}

impl FromValue for usize {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value
            .to_u64()
            .and_then(|v| usize::try_from(v).map_err(Error::TryFromInt))
    }
}

impl FromValue for u64 {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value.to_u64()
    }
}

impl FromValue for u32 {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value.to_u32()
    }
}

impl FromValue for f32 {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value.to_f32()
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self, Error> {
        value.to_bool()
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: &Value) -> Result<Self, Error> {
        match value {
            Value::Array(arr) => arr.iter().map(T::from_value).collect(),
            v => Err(Value::type_error("Array", v)),
        }
    }
}

/// Implement `FromValue` for `Option<T>` to handle optional metadata fields.
impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &Value) -> Result<Self, Error> {
        // If the key exists, try to convert the value.
        T::from_value(value).map(Some)
    }

    fn from_none(_key: &str) -> Result<Self, Error> {
        // If the key is missing, return Ok(None).
        Ok(None)
    }
}
