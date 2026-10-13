/************************************************
* File: journal.rs
* Author: Michal Švrček
*
* Linux systemd journal integration.
*
* ver. 0.1.0
*************************************************/

use crate::{output, reader};
use std::process::Command;

pub fn read(
    unit: Option<&str>,
    lines: usize,
    level: Option<&str>,
    search: Option<&str>,
) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("journalctl is supported only on Linux".into());
    }

    let mut command = Command::new("journalctl");

    command
        .arg("--no-pager")
        .arg("--output=short-iso")
        .arg("--lines")
        .arg(lines.to_string());

    if let Some(unit) = unit {
        command.arg("--unit").arg(unit);
    }

    let result = command
        .output()
        .map_err(|e| format!("Cannot execute journalctl: {e}"))?;

    if !result.status.success() {
        let reason = String::from_utf8_lossy(&result.stderr);
        return Err(format!("journalctl failed: {}", reason.trim()));
    }

    let content = String::from_utf8_lossy(&result.stdout);

    output::heading(match unit {
        Some(_) => "Systemd service logs",
        None => "System journal",
    });

    if let Some(unit) = unit {
        output::info(&format!("Unit: {unit}"));
    }

    let mut count = 0usize;

    for line in content.lines() {
        if reader::matches(line, level, search) {
            output::log_line(line);
            count += 1;
        }
    }

    println!();
    output::ok(&format!("{count} matching journal entries"));

    Ok(())
}
