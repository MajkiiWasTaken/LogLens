/************************************************
* File: follow.rs
* Author: Michal Švrček
*
* Live log file monitoring and shutdown handling.
*
* ver. 0.2.0
*************************************************/

use crate::{output, reader};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    },
    thread,
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

static RUNNING: AtomicBool = AtomicBool::new(true);
static HANDLER: OnceLock<Result<(), String>> = OnceLock::new();

pub fn init() -> Result<(), String> {
    RUNNING.store(true, Ordering::SeqCst);

    HANDLER
        .get_or_init(|| {
            ctrlc::set_handler(|| {
                RUNNING.store(false, Ordering::SeqCst);
            })
            .map_err(|e| format!("Cannot install Ctrl+C handler: {e}"))
        })
        .clone()
}

pub fn running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn file(
    path: &Path,
    level: Option<&str>,
    search: Option<&str>,
    from_start: bool,
    interval_ms: u64,
) -> Result<(), String> {
    init()?;

    let metadata =
        fs::metadata(path).map_err(|e| format!("Cannot access {}: {e}", path.display()))?;

    let mut position = if from_start { 0 } else { metadata.len() };

    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    #[cfg(unix)]
    let mut identity = (metadata.dev(), metadata.ino());

    let mut pending = String::new();

    output::heading(&format!("Following: {}", path.display()));
    output::info("Press Ctrl+C to stop.");

    while running() {
        match fs::metadata(path) {
            Ok(metadata) => {
                #[cfg(unix)]
                {
                    let current = (metadata.dev(), metadata.ino());

                    if current != identity {
                        position = 0;
                        pending.clear();
                        identity = current;
                        output::info("Log file rotated.");
                    }
                }

                if metadata.len() < position {
                    position = 0;
                    pending.clear();
                    output::info("Log file truncated.");
                }

                if metadata.len() > position {
                    // File handle exists only during this read.
                    let mut reader = BufReader::new(open_log_file(path)?);

                    reader
                        .seek(SeekFrom::Start(position))
                        .map_err(|e| e.to_string())?;

                    loop {
                        let mut chunk = String::new();

                        let bytes = reader
                            .read_line(&mut chunk)
                            .map_err(|e| format!("Cannot read log: {e}"))?;

                        if bytes == 0 {
                            break;
                        }

                        position += bytes as u64;
                        pending.push_str(&chunk);

                        while let Some(index) = pending.find('\n') {
                            let line = pending[..index].trim_end_matches('\r').to_string();

                            pending.drain(..=index);

                            if reader::matches(&line, level, search) {
                                output::log_line(&line);
                            }
                        }
                    }

                    // Reader and file handle close here.
                }
            }

            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // Log rotation can temporarily remove the file.
            }

            Err(error) => {
                return Err(format!("Cannot inspect log: {error}"));
            }
        }

        thread::sleep(Duration::from_millis(interval_ms));
    }

    println!();
    output::ok("Log monitoring stopped.");

    Ok(())
}

pub(crate) fn open_log_file(path: &Path) -> Result<File, String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        // Permit concurrent readers, writers, and file rotation.
        const FILE_SHARE_READ: u32 = 0x00000001;
        const FILE_SHARE_WRITE: u32 = 0x00000002;
        const FILE_SHARE_DELETE: u32 = 0x00000004;

        options.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE);
    }

    options
        .open(path)
        .map_err(|e| format!("Cannot open {}: {e}", path.display()))
}
