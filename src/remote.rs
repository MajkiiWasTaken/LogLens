/************************************************
* File: remote.rs
* Author: Michal Švrček
*
* Remote Linux journal diagnostics over SSH.
*
* ver. 0.3.0
*************************************************/

use crate::export::{self, Format};
use crate::filter::{self, JournalFilter};
use crate::{follow, output, reader};
use std::path::Path;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

pub struct RemoteOptions<'a> {
    pub device: &'a str,
    pub host: Option<&'a str>,
    pub user: Option<&'a str>,
    pub port: Option<u16>,
    pub unit: Option<&'a str>,
    pub lines: u32,
    pub level: Option<&'a str>,
    pub search: Option<&'a str>,
    pub follow: bool,
    pub priority: Option<&'a str>,
    pub since: Option<&'a str>,
    pub until: Option<&'a str>,
    pub export: Option<&'a Path>,
    pub format: Format,
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '@'))
        && !value.starts_with('-')
}

fn ssh_command(options: &RemoteOptions<'_>) -> Result<Command, String> {
    let host = options.host.unwrap_or(options.device);

    if !safe_identifier(host) {
        return Err("Invalid SSH host or alias".into());
    }

    if let Some(user) = options.user {
        if !safe_identifier(user) {
            return Err("Invalid SSH username".into());
        }
    }

    if options.port == Some(0) {
        return Err("SSH port cannot be zero".into());
    }

    if let Some(unit) = options.unit {
        if !safe_identifier(unit) {
            return Err("Invalid systemd unit name".into());
        }
    }

    // Journal filters
    let filters = JournalFilter {
        priority: options.priority,
        since: options.since,
        until: options.until,
    };

    // Build remote journalctl arguments
    let mut args = vec![
        "journalctl".to_string(),
        "--no-pager".to_string(),
        "--quiet".to_string(),
        "--output=short-iso".to_string(),
        format!("--lines={}", options.lines),
    ];

    if let Some(unit) = options.unit {
        args.push(format!("--unit={unit}"));
    }

    if options.follow {
        args.push("--follow".to_string());
    }

    // Add priority and time filters
    filter::append_remote(&mut args, &filters)?;

    // SSH executes the remote command through a shell
    let remote = args.join(" ");

    let mut command = Command::new("ssh");

    command
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("ConnectTimeout=10")
        .arg("-o")
        .arg("ServerAliveInterval=15")
        .arg("-o")
        .arg("ServerAliveCountMax=2");

    if let Some(user) = options.user {
        command.arg("-l").arg(user);
    }

    if let Some(port) = options.port {
        command.arg("-p").arg(port.to_string());
    }

    command.arg("--").arg(host).arg(remote);

    Ok(command)
}

pub fn run(options: RemoteOptions<'_>) -> Result<(), String> {
    let mut command = ssh_command(&options)?;

    output::heading(&format!("Remote journal: {}", options.device));

    if let Some(unit) = options.unit {
        output::info(&format!("Unit: {unit}"));
    }

    if options.follow && options.export.is_some() {
        return Err("Export during live monitoring is not supported. Omit --follow.".into());
    }

    if options.follow {
        run_follow(&mut command, &options)
    } else {
        run_once(&mut command, &options)
    }
}

fn run_once(command: &mut Command, options: &RemoteOptions<'_>) -> Result<(), String> {
    let result = command
        .output()
        .map_err(|e| format!("Cannot start SSH: {e}"))?;

    if !result.status.success() {
        return Err(format!(
            "SSH command failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }

    let content = String::from_utf8_lossy(&result.stdout);
    let mut entries = Vec::new();

    for line in content.lines() {
        // Ignore journalctl informational messages.
        if line.starts_with("-- Journal begins") || line.starts_with("-- No entries") {
            continue;
        }

        if reader::matches(line, options.level, options.search) {
            output::log_line(line);
            entries.push(line.to_string());
        }
    }

    println!();
    output::ok(&format!("{} matching journal entries", entries.len()));

    if let Some(path) = options.export {
        export::write(path, options.format, options.device, &entries)?;
    }

    Ok(())
}

fn run_follow(command: &mut Command, options: &RemoteOptions<'_>) -> Result<(), String> {
    follow::init()?;

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Cannot start SSH: {e}"))?;

    let stdout = child.stdout.take().ok_or("Cannot capture SSH output")?;

    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    let reader_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.map_err(|e| e.to_string())).is_err() {
                break;
            }
        }
    });

    output::info("Following remote logs. Press Ctrl+C to stop.");

    let mut read_error = None;
    let mut exit_status = None;

    while follow::running() {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(line)) => {
                if reader::matches(&line, options.level, options.search) {
                    output::log_line(&line);
                }
            }
            Ok(Err(error)) => {
                read_error = Some(error);
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                    exit_status = Some(status);
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                exit_status = Some(child.wait().map_err(|e| e.to_string())?);
                break;
            }
        }
    }

    if exit_status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }

    let _ = reader_thread.join();

    if let Some(error) = read_error {
        return Err(format!("Cannot read SSH output: {error}"));
    }

    if let Some(status) = exit_status {
        if !status.success() && follow::running() {
            return Err(format!("SSH exited with {status}"));
        }
    }

    println!();
    output::ok("Remote monitoring stopped.");

    Ok(())
}
