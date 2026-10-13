# LogLens

**LogLens** is a lightweight, cross-platform log inspection and diagnostics CLI written in Rust.

Read, filter, follow, and export logs from local files, Linux systemd journals, and remote devices over SSH.

**Platforms:** Windows & Linux | **License:** MIT

## Features

- Local log viewing and text filtering
- Real-time file monitoring
- Linux systemd journal integration
- Remote log inspection over SSH
- Priority and time-based journal filtering
- Saved SSH device profiles
- TXT and JSON export
- Colored terminal output

## Installation

Download the latest package from [GitHub Releases](https://github.com/MajkiiWasTaken/LogLens/releases).

### Windows

Extract `LogLens-windows-x86_64.zip` and run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1 -BinaryPath .\lens.exe
```

### Linux

Extract `LogLens-linux-x86_64.tar.gz` and run:

```bash
bash ./install.sh ./lens
```

Ensure `~/.local/bin` is in your `PATH`.

### Build from source

```bash
git clone https://github.com/MajkiiWasTaken/LogLens.git
cd LogLens
cargo build --release
```

## Usage

| Command | Description |
|---|---|
| `lens file app.log` | Read a local log |
| `lens file app.log --level error` | Filter log messages |
| `lens follow app.log` | Monitor new log entries |
| `lens journal --unit ssh` | Read systemd journal |
| `lens journal --follow` | Follow Linux journal |
| `lens remote raspberry-pi --unit ssh` | Read logs over SSH |
| `lens remote raspberry-pi --priority err` | Filter journal severity |
| `lens remote raspberry-pi --since "1 hour ago"` | Filter by time |
| `lens device list` | List SSH device profiles |

Use `lens --help` for the complete command reference.

## Device Profiles

Save commonly used SSH devices:

```powershell
lens device add raspberry-pi --host 192.168.1.20 --user pi
lens device list
lens remote raspberry-pi --lines 50
```

Profiles are stored in `%APPDATA%\LogLens\config.toml` on Windows or `~/.config/loglens/config.toml` on Linux.

## Export

Export filtered log entries to text or JSON:

```powershell
lens file app.log --export logs.txt
lens journal --priority err --export errors.json --format json
lens remote raspberry-pi --since "1 hour ago" --export remote.json --format json
```

Export is available for one-time log reads, not continuous `--follow` sessions.

## Development

```bash
cargo fmt --check
cargo check
cargo test
cargo build --release
```

## Notes

- Local `journalctl` requires Linux.
- Remote SSH uses the system SSH client and existing SSH credentials.
- Journal priority filtering is based on systemd severity metadata.
- Reading protected journals requires appropriate permissions.
- Only inspect systems you are authorized to access.

## Author

**Michal Švrček**

## License

Distributed under the MIT License. See [LICENSE](LICENSE).
