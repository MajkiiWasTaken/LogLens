/************************************************
* File: main.rs
* Author: Michal Švrček
*
* LogLens command line interface.
*
* ver. 0.1.0
*************************************************/

mod config;
mod device;
mod export;
mod filter;
mod follow;
mod journal;
mod output;
mod reader;
mod remote;

use clap::{Parser, Subcommand};
use export::Format;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "lens",
    version,
    about = "Cross-platform log inspection and diagnostics",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum DeviceCommands {
    /// Add an SSH device profile
    Add {
        name: String,

        #[arg(long)]
        host: String,

        #[arg(long)]
        user: Option<String>,

        #[arg(long)]
        port: Option<u16>,

        #[arg(long, default_value = "")]
        description: String,
    },

    /// List configured devices
    List,

    /// Show a device profile
    Info { name: String },

    /// Remove a device profile
    Remove {
        name: String,

        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Read and filter a local log file
    File {
        /// Path to a log file
        path: PathBuf,

        /// Number of matching lines to display
        #[arg(
            short = 'n',
            long,
            default_value_t = 50,
            value_parser = clap::value_parser!(u32).range(1..=100_000)
        )]
        lines: u32,

        /// Text-based severity filter, e.g. error or warn
        #[arg(long)]
        level: Option<String>,

        /// Case-insensitive text search
        #[arg(long)]
        search: Option<String>,

        /// Save displayed log entries to a file
        #[arg(long)]
        export: Option<PathBuf>,

        /// Export format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Read Linux systemd journal logs
    Journal {
        /// systemd unit, e.g. ssh or nginx
        #[arg(long)]
        unit: Option<String>,

        /// Number of recent journal entries to examine
        #[arg(
            short = 'n',
            long,
            default_value_t = 50,
            value_parser = clap::value_parser!(u32).range(1..=100_000)
        )]
        lines: u32,

        /// Text-based severity filter
        #[arg(long)]
        level: Option<String>,

        /// Case-insensitive text search
        #[arg(long)]
        search: Option<String>,

        /// Continuously follow new journal entries
        #[arg(short = 'f', long)]
        follow: bool,

        /// Filter by systemd journal priority
        #[arg(long)]
        priority: Option<String>,

        /// Show entries since a given time
        #[arg(long)]
        since: Option<String>,

        /// Show entries until a given time
        #[arg(long)]
        until: Option<String>,

        /// Save displayed log entries to a file
        #[arg(long)]
        export: Option<PathBuf>,

        /// Export format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Follow new lines appended to a local log file
    Follow {
        /// Path to the log file
        path: PathBuf,

        /// Text-based severity filter
        #[arg(long)]
        level: Option<String>,

        /// Case-insensitive text search
        #[arg(long)]
        search: Option<String>,

        /// Read existing contents before following new lines
        #[arg(long)]
        from_start: bool,

        /// Polling interval in milliseconds
        #[arg(
            long,
            default_value_t = 250,
            value_parser = clap::value_parser!(u64).range(50..=5000)
        )]
        interval_ms: u64,
    },

    /// Read remote Linux systemd logs over SSH
    Remote {
        /// SSH host, IP address or configured SSH alias
        device: String,

        /// Override SSH hostname or IP address
        #[arg(long)]
        host: Option<String>,

        /// SSH username
        #[arg(long)]
        user: Option<String>,

        /// SSH port (uses SSH configuration when omitted)
        #[arg(long)]
        port: Option<u16>,

        /// systemd unit
        #[arg(long)]
        unit: Option<String>,

        /// Number of recent journal entries
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: u32,

        /// Case-insensitive severity filter
        #[arg(long)]
        level: Option<String>,

        /// Case-insensitive text filter
        #[arg(long)]
        search: Option<String>,

        /// Follow remote logs continuously
        #[arg(short = 'f', long)]
        follow: bool,

        /// Filter by systemd journal priority
        #[arg(long)]
        priority: Option<String>,

        /// Show entries since a given time
        #[arg(long)]
        since: Option<String>,

        /// Show entries until a given time
        #[arg(long)]
        until: Option<String>,

        /// Export matching entries to a file
        #[arg(long)]
        export: Option<PathBuf>,

        /// Export format: text or json
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Manage saved SSH devices
    Device {
        #[command(subcommand)]
        command: DeviceCommands,
    },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    output::banner();

    match cli.command {
        Commands::File {
            path,
            lines,
            level,
            search,
            export,
            format,
        } => {
            let format = Format::parse(&format)?;

            reader::read_file(
                &path,
                lines as usize,
                level.as_deref(),
                search.as_deref(),
                export.as_deref(),
                format,
            )
        }

        Commands::Journal {
            unit,
            lines,
            level,
            search,
            follow,
            priority,
            since,
            until,
            export,
            format,
        } => {
            let filters = filter::JournalFilter {
                priority: priority.as_deref(),
                since: since.as_deref(),
                until: until.as_deref(),
            };

            let format = Format::parse(&format)?;

            if follow {
                if export.is_some() {
                    return Err(
                        "Export during live monitoring is not supported. Omit --follow.".into(),
                    );
                }

                journal::follow(
                    unit.as_deref(),
                    lines as usize,
                    level.as_deref(),
                    search.as_deref(),
                    &filters,
                )
            } else {
                journal::read(
                    unit.as_deref(),
                    lines as usize,
                    level.as_deref(),
                    search.as_deref(),
                    &filters,
                    export.as_deref(),
                    format,
                )
            }
        }

        Commands::Follow {
            path,
            level,
            search,
            from_start,
            interval_ms,
        } => follow::file(
            &path,
            level.as_deref(),
            search.as_deref(),
            from_start,
            interval_ms,
        ),

        Commands::Remote {
            device,
            host,
            user,
            port,
            unit,
            lines,
            level,
            search,
            follow,
            priority,
            since,
            until,
            export,
            format,
        } => {
            let config = config::Config::load()?;
            let profile = config.devices.get(&device);

            let resolved_host = host
                .as_deref()
                .or_else(|| profile.map(|saved| saved.host.as_str()));

            let resolved_user = user
                .as_deref()
                .or_else(|| profile.and_then(|saved| saved.user.as_deref()));

            let resolved_port = port.or_else(|| profile.and_then(|saved| saved.port));

            let format = Format::parse(&format)?;

            remote::run(remote::RemoteOptions {
                device: &device,
                host: resolved_host,
                user: resolved_user,
                port: resolved_port,
                unit: unit.as_deref(),
                lines,
                level: level.as_deref(),
                search: search.as_deref(),
                follow,
                priority: priority.as_deref(),
                since: since.as_deref(),
                until: until.as_deref(),
                export: export.as_deref(),
                format,
            })
        }

        Commands::Device { command } => match command {
            DeviceCommands::Add {
                name,
                host,
                user,
                port,
                description,
            } => device::add(&name, &host, user.as_deref(), port, &description),

            DeviceCommands::List => device::list(),

            DeviceCommands::Info { name } => device::info(&name),

            DeviceCommands::Remove { name, yes } => device::remove(&name, yes),
        },
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            output::error(&error);
            ExitCode::FAILURE
        }
    }
}
