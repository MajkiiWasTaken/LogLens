/************************************************
* File: reader.rs
* Author: Michal Švrček
*
* Local log file reading and filtering.
*
* ver. 0.1.0
*************************************************/

use crate::export::{self, Format};
use crate::output;
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    path::Path,
};

pub fn matches(line: &str, level: Option<&str>, search: Option<&str>) -> bool {
    let lower = line.to_ascii_lowercase();

    if let Some(level) = level {
        if !lower.contains(&level.to_ascii_lowercase()) {
            return false;
        }
    }

    if let Some(search) = search {
        if !lower.contains(&search.to_ascii_lowercase()) {
            return false;
        }
    }

    true
}

pub fn read_file(
    path: &Path,
    limit: usize,
    level: Option<&str>,
    search: Option<&str>,
    export_path: Option<&Path>,
    format: Format,
) -> Result<(), String> {
    let file = crate::follow::open_log_file(path)?;
    let reader = BufReader::new(file);

    let mut selected = VecDeque::with_capacity(limit.min(1024));
    let mut matching = 0usize;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Cannot read log: {e}"))?;

        if !matches(&line, level, search) {
            continue;
        }

        matching += 1;

        if selected.len() == limit {
            selected.pop_front();
        }

        selected.push_back(line);
    }

    output::heading(&format!("Log file: {}", path.display()));

    let entries: Vec<String> = selected.into_iter().collect();

    for line in &entries {
        output::log_line(line);
    }

    println!();

    output::ok(&format!(
        "{matching} matching lines (showing last {})",
        entries.len()
    ));

    if let Some(destination) = export_path {
        export::write(destination, format, &path.display().to_string(), &entries)?;
    }

    Ok(())
}
