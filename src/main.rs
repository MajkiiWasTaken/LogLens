/************************************************
* File: main.rs
* Author: Michal Švrček
*
* LogLens command line interface.
*
* ver. 0.1.0
*************************************************/

mod journal;
mod output;
mod reader;

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
        } => journal::read(
            unit.as_deref(),
            lines as usize,
            level.as_deref(),
            search.as_deref(),
        ),
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
