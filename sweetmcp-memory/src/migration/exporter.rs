//! Export functionality for memory data

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::migration::{MigrationError, Result};

/// Data export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Binary format
    Binary,
}

/// Data exporter
pub struct DataExporter {
    format: ExportFormat,
}

impl DataExporter {
    /// Create a new exporter
    pub fn new(format: ExportFormat) -> Self {
        Self { format }
    }

    /// Export data to file for JSON/CSV formats
    /// Note: Binary format requires bincode::Encode trait - use export_binary directly
    pub async fn export_to_file<T>(&self, data: &[T], path: &Path) -> Result<()>
    where
        T: Serialize,
    {
        match self.format {
            ExportFormat::Json => self.export_json(data, path),
            ExportFormat::Csv => self.export_csv(data, path),
            ExportFormat::Binary => Err(crate::migration::MigrationError::UnsupportedFormat(
                "Binary export requires bincode::Encode trait - use export_binary directly"
                    .to_string(),
            )),
        }
    }

    /// Export as JSON
    fn export_json<T: Serialize>(&self, data: &[T], path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Export as CSV
    fn export_csv<T: Serialize>(&self, data: &[T], path: &Path) -> Result<()> {
        // Simplified CSV export - would use csv crate in production
        let mut file = File::create(path)?;

        if let Some(first) = data.first() {
            let json = serde_json::to_value(first)?;
            if let serde_json::Value::Object(map) = json {
                // Write headers
                let headers: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
                writeln!(file, "{}", headers.join(","))?;
            }
        }

        // Write data rows
        for item in data {
            let json = serde_json::to_value(item)?;
            if let serde_json::Value::Object(map) = json {
                let values: Vec<String> = map
                    .values()
                    .map(|v| match v {
                        serde_json::Value::String(s) => format!("\"{}\"", s),
                        _ => v.to_string(),
                    })
                    .collect();
                writeln!(file, "{}", values.join(","))?;
            }
        }

        Ok(())
    }

    /// Export as binary
    pub fn export_binary<T>(&self, data: &[T], path: &Path) -> Result<()>
    where
        T: bincode::Encode,
    {
        let bytes = bincode::encode_to_vec(data, bincode::config::standard()).map_err(|e| {
            MigrationError::UnsupportedFormat(format!("Binary encoding failed: {}", e))
        })?;
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,

    /// Include metadata
    pub include_metadata: bool,

    /// Include relationships
    pub include_relationships: bool,

    /// Batch size for large exports
    pub batch_size: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_relationships: true,
            batch_size: 1000,
        }
    }
}
