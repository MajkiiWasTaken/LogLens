/************************************************
* File: main.rs
* Author: Michal Švrček
*
* LogLens command line interface.
*
* ver. 0.1.0
*************************************************/

mod follow;
mod journal;
mod output;
mod reader;
mod remote;

use clap::{Parser, Subcommand};
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
        } => reader::read_file(&path, lines as usize, level.as_deref(), search.as_deref()),

        Commands::Journal {
            unit,
            lines,
            level,
            search,
            follow,
        } => {
            if follow {
                journal::follow(
                    unit.as_deref(),
                    lines as usize,
                    level.as_deref(),
                    search.as_deref(),
                )
            } else {
                journal::read(
                    unit.as_deref(),
                    lines as usize,
                    level.as_deref(),
                    search.as_deref(),
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
        } => remote::run(remote::RemoteOptions {
            device: &device,
            host: host.as_deref(),
            user: user.as_deref(),
            port,
            unit: unit.as_deref(),
            lines,
            level: level.as_deref(),
            search: search.as_deref(),
            follow,
        }),
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
