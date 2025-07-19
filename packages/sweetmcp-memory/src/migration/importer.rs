//! Import functionality for memory data

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::migration::{MigrationError, Result};

/// Data importer
pub struct DataImporter;

impl DataImporter {
    /// Create a new importer
    pub fn new() -> Self {
        Self
    }

    /// Import data from JSON file
    pub async fn import_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<Vec<T>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data: Vec<T> = serde_json::from_str(&contents)?;
        Ok(data)
    }

    /// Import data from CSV file
    pub async fn import_csv<T: for<'de> Deserialize<'de>>(&self, _path: &Path) -> Result<Vec<T>> {
        // Simplified CSV import - would use csv crate in production
        Err(MigrationError::UnsupportedFormat(
            "CSV import not yet implemented".to_string(),
        ))
    }

    /// Import data from binary file
    pub async fn import_binary<T>(&self, path: &Path) -> Result<Vec<T>>
    where
        T: bincode::Decode<()>,
    {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let data: Vec<T> = bincode::decode_from_slice(&buffer, bincode::config::standard())
            .map_err(|e| {
                MigrationError::UnsupportedFormat(format!("Binary decoding failed: {}", e))
            })?
            .0;
        Ok(data)
    }

    /// Import with validation for JSON/CSV formats only
    /// Note: Binary format requires separate validation due to different trait bounds
    pub async fn import_with_validation<T, F>(
        &self,
        path: &Path,
        format: ImportFormat,
        validator: F,
    ) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
        F: Fn(&T) -> Result<()>,
    {
        let data = match format {
            ImportFormat::Json => self.import_json(path).await?,
            ImportFormat::Csv => self.import_csv(path).await?,
            ImportFormat::Binary => {
                return Err(MigrationError::UnsupportedFormat(
                    "Binary validation requires bincode::Decode trait - use import_binary directly"
                        .to_string(),
                ));
            }
        };

        // Validate each item
        for item in &data {
            validator(item)?;
        }

        Ok(data)
    }
}

impl Default for DataImporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Import format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Binary format
    Binary,
}

/// Import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Import format
    pub format: ImportFormat,

    /// Skip validation
    pub skip_validation: bool,

    /// Batch size for large imports
    pub batch_size: usize,

    /// Continue on error
    pub continue_on_error: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            format: ImportFormat::Json,
            skip_validation: false,
            batch_size: 1000,
            continue_on_error: false,
        }
    }
}
