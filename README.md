# Hytale World Exporter

A cross-platform desktop application for backing up and restoring Hytale game worlds.

## Features

- 🖥️ Cross-platform (macOS & Windows)
- 🌍 View and select worlds with details (size, last played)
- 🗜️ Export worlds as ZIP with optional logs/backups exclusion
- 📥 Import/restore worlds from ZIP backups
- 📋 View server logs with syntax highlighting (WARN/ERROR)
- 📦 Manage existing world backups
- 🌐 Localized (English & German, auto-detected)
- 📊 Progress indicator during compression

## Installation

### Download

Download the latest release from [GitHub Releases](https://github.com/renickbuettner/hytale-world-exporter/releases).

### Building from Source

```bash
git clone https://github.com/renickbuettner/hytale-world-exporter.git
cd hytale-world-exporter
cargo build --release
```

## World Paths

- **Windows**: `%APPDATA%/Hytale/worlds`
- **macOS**: `~/Library/Application Support/Hytale/UserData/Saves`

## License

MIT License - see [LICENSE](LICENSE) file for details.
