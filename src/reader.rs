/************************************************
* File: reader.rs
* Author: Michal Švrček
*
* Local log file reading and filtering.
*
* ver. 0.1.0
*************************************************/

use crate::output;
use std::{
    collections::VecDeque,
    fs::File,
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
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;

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

    for line in selected {
        output::log_line(&line);
    }

    println!();
    output::ok(&format!(
        "{matching} matching lines (showing last {})",
        matching.min(limit)
    ));

    Ok(())
}
