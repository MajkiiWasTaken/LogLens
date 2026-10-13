/************************************************
* File: device.rs
* Author: Michal Švrček
*
* Saved SSH device profile management.
*
* ver. 0.5.0
*************************************************/

use crate::{
    config::{Config, Device},
    output,
};

pub fn add(
    name: &str,
    host: &str,
    user: Option<&str>,
    port: Option<u16>,
    description: &str,
) -> Result<(), String> {
    if !valid_name(name) {
        return Err("Invalid device name".into());
    }

    if !valid_name(host) {
        return Err("Invalid hostname or IP address".into());
    }

    if user.is_some_and(|value| !valid_name(value)) {
        return Err("Invalid SSH username".into());
    }

    if port == Some(0) {
        return Err("SSH port cannot be zero".into());
    }

    let mut config = Config::load()?;

    if config.devices.contains_key(name) {
        return Err(format!("Device '{name}' already exists"));
    }

    config.devices.insert(
        name.to_string(),
        Device {
            host: host.to_string(),
            user: user.map(str::to_owned),
            port,
            description: description.to_string(),
        },
    );

    config.save()?;

    output::ok(&format!("Device '{name}' added"));
    output::info(&format!("Host: {host}"));

    Ok(())
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '@'))
}

pub fn list() -> Result<(), String> {
    let config = Config::load()?;

    output::heading("Configured devices");

    if config.devices.is_empty() {
        output::info("No devices configured.");
        return Ok(());
    }

    for (name, device) in &config.devices {
        output::ok(&format!("{name:<20} {}", device.host));
    }

    output::info(&format!("{} configured device(s)", config.devices.len()));

    Ok(())
}

pub fn info(name: &str) -> Result<(), String> {
    let config = Config::load()?;

    let device = config
        .devices
        .get(name)
        .ok_or_else(|| format!("Device '{name}' not found"))?;

    output::heading(&format!("Device: {name}"));
    output::info(&format!("Host: {}", device.host));
    output::info(&format!(
        "User: {}",
        device.user.as_deref().unwrap_or("SSH default")
    ));
    output::info(&format!(
        "Port: {}",
        device.port.map_or("SSH default".into(), |v| v.to_string())
    ));

    if !device.description.is_empty() {
        output::info(&format!("Description: {}", device.description));
    }

    Ok(())
}

pub fn remove(name: &str, yes: bool) -> Result<(), String> {
    if !yes {
        return Err("Use --yes to confirm removal".into());
    }

    let mut config = Config::load()?;

    if config.devices.remove(name).is_none() {
        return Err(format!("Device '{name}' not found"));
    }

    config.save()?;

    output::ok(&format!("Device '{name}' removed"));
    Ok(())
}
