/************************************************
* File: filter.rs
* Author: Michal Švrček
*
* Journal priority and time filter validation.
*
* ver. 0.4.0
*************************************************/

pub struct JournalFilter<'a> {
    pub priority: Option<&'a str>,
    pub since: Option<&'a str>,
    pub until: Option<&'a str>,
}

pub fn normalize_priority(value: &str) -> Result<&'static str, String> {
    match value.to_ascii_lowercase().as_str() {
        "0" | "emerg" | "emergency" => Ok("0"),
        "1" | "alert" => Ok("1"),
        "2" | "crit" | "critical" => Ok("2"),
        "3" | "err" | "error" => Ok("3"),
        "4" | "warning" | "warn" => Ok("4"),
        "5" | "notice" => Ok("5"),
        "6" | "info" => Ok("6"),
        "7" | "debug" => Ok("7"),
        _ => Err(format!(
            "Invalid journal priority '{value}'. \
             Use emerg, alert, crit, err, warning, notice, info or debug"
        )),
    }
}

pub fn validate_time(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("Time filter cannot be empty".into());
    }

    if value.starts_with('-') {
        return Err("Time filter cannot start with '-'".into());
    }

    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\'' | '"' | '`' | '$' | '\\' | ';' | '|'))
    {
        return Err("Time filter contains unsupported characters".into());
    }

    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | ':' | '-' | '+' | '/' | '.' | '_'))
    {
        return Err("Time filter contains unsupported characters".into());
    }

    Ok(())
}

pub fn validate(filter: &JournalFilter<'_>) -> Result<(), String> {
    if let Some(priority) = filter.priority {
        normalize_priority(priority)?;
    }

    if let Some(since) = filter.since {
        validate_time(since)?;
    }

    if let Some(until) = filter.until {
        validate_time(until)?;
    }

    Ok(())
}

pub fn append_local(
    command: &mut std::process::Command,
    filter: &JournalFilter<'_>,
) -> Result<(), String> {
    validate(filter)?;

    if let Some(priority) = filter.priority {
        command.arg("--priority").arg(normalize_priority(priority)?);
    }

    if let Some(since) = filter.since {
        command.arg("--since").arg(since);
    }

    if let Some(until) = filter.until {
        command.arg("--until").arg(until);
    }

    Ok(())
}

pub fn append_remote(
    arguments: &mut Vec<String>,
    filter: &JournalFilter<'_>,
) -> Result<(), String> {
    validate(filter)?;

    if let Some(priority) = filter.priority {
        arguments.push(format!("--priority={}", normalize_priority(priority)?));
    }

    for (flag, value) in [("--since", filter.since), ("--until", filter.until)] {
        if let Some(value) = value {
            arguments.push(format!("{flag}={}", shell_quote(value)));
        }
    }

    Ok(())
}

// POSIX single-quote escaping for the remote SSH shell.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_aliases() {
        assert_eq!(normalize_priority("ERROR").unwrap(), "3");
        assert_eq!(normalize_priority("warn").unwrap(), "4");
        assert!(normalize_priority("verbose").is_err());
    }

    #[test]
    fn reject_unsafe_time() {
        assert!(validate_time("1 hour ago").is_ok());
        assert!(validate_time("2026-10-13 09:00:00").is_ok());
        assert!(validate_time("today; reboot").is_err());
        assert!(validate_time("$(whoami)").is_err());
    }
}
