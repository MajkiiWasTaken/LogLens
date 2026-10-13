/************************************************
* File: export.rs
* Author: Michal Švrček
*
* Text and JSON log export.
*
* ver. 0.5.0
*************************************************/

use crate::output;
use serde::Serialize;
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

impl Format {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "Unsupported export format '{value}'. Use text or json"
            )),
        }
    }
}

#[derive(Serialize)]
struct Report<'a> {
    source: &'a str,
    exported_at_unix: u64,
    count: usize,
    entries: &'a [String],
}

pub fn write(path: &Path, format: Format, source: &str, entries: &[String]) -> Result<(), String> {
    let content = match format {
        Format::Text => {
            if entries.is_empty() {
                String::new()
            } else {
                format!("{}\n", entries.join("\n"))
            }
        }

        Format::Json => {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs();

            let report = Report {
                source,
                exported_at_unix: timestamp,
                count: entries.len(),
                entries,
            };

            serde_json::to_string_pretty(&report).map_err(|e| format!("JSON export failed: {e}"))?
        }
    };

    fs::write(path, content).map_err(|e| format!("Cannot save {}: {e}", path.display()))?;

    output::ok(&format!(
        "Exported {} entries to {}",
        entries.len(),
        path.display()
    ));

    Ok(())
}
