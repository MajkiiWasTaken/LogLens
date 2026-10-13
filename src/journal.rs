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

pub fn follow(
    unit: Option<&str>,
    lines: usize,
    level: Option<&str>,
    search: Option<&str>,
) -> Result<(), String> {
    use crate::follow;
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
        sync::mpsc,
        thread,
        time::Duration,
    };

    if !cfg!(target_os = "linux") {
        return Err("journalctl is supported only on Linux".into());
    }

    follow::init()?;

    let mut command = Command::new("journalctl");

    command
        .arg("--no-pager")
        .arg("--output=short-iso")
        .arg("--follow")
        .arg("--lines")
        .arg(lines.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    if let Some(unit) = unit {
        command.arg("--unit").arg(unit);
    }

    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot launch journalctl: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or("Cannot capture journalctl output")?;

    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    let reader_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let message = line.map_err(|e| e.to_string());

            if tx.send(message).is_err() {
                break;
            }
        }
    });

    output::heading("Live systemd journal");

    if let Some(unit) = unit {
        output::info(&format!("Unit: {unit}"));
    }

    output::info("Press Ctrl+C to stop.");

    let mut read_error = None;
    let mut child_status = None;

    while follow::running() {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(line)) => {
                if reader::matches(&line, level, search) {
                    output::log_line(&line);
                }
            }
            Ok(Err(error)) => {
                read_error = Some(error);
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                    child_status = Some(status);
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                child_status = child.wait().ok();
                break;
            }
        }
    }

    if child_status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }

    let _ = reader_thread.join();

    if let Some(error) = read_error {
        return Err(format!("Cannot read journal: {error}"));
    }

    if let Some(status) = child_status {
        if !status.success() && follow::running() {
            return Err(format!("journalctl exited with {status}"));
        }
    }

    println!();
    output::ok("Journal monitoring stopped.");

    Ok(())
}
