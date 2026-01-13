# Hytale World Exporter

<p align="center">
  <img src="assets/app_icon_256.png" alt="Hytale World Exporter Icon" width="128">
</p>

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

#### macOS Installation Notes

Due to macOS security requirements, you may see a warning when opening the app for the first time. To run the app:

1. **First time opening**: Right-click (or Control-click) on the app and select "Open"
2. Click "Open" in the dialog that appears
3. The app will now run and can be opened normally in the future

Alternatively, you can allow the app in System Preferences:
1. Go to **System Preferences** > **Security & Privacy** > **General**
2. Click "Open Anyway" next to the message about the blocked app
3. Confirm by clicking "Open"

**Für deutsche Anleitung zur Code-Signierung, siehe [docs/MACOS_CODESIGNING_DE.md](docs/MACOS_CODESIGNING_DE.md)**

### Building from Source

```bash
git clone https://github.com/renickbuettner/hytale-world-exporter.git
cd hytale-world-exporter
cargo build --release
```

#### macOS: Code Signing for Local Builds

When building locally on macOS, you can sign the app bundle with an ad-hoc signature:

```bash
# Build the binary
cargo build --release

# Create app bundle
mkdir -p "Hytale World Exporter.app/Contents/MacOS"
mkdir -p "Hytale World Exporter.app/Contents/Resources"
cp target/release/hytale-world-exporter "Hytale World Exporter.app/Contents/MacOS/"

# Create Info.plist (see build workflow for full example)
# ...

# Sign with ad-hoc signature
codesign --force --deep --sign - "Hytale World Exporter.app"
```

## World Paths

- **Windows**: `%APPDATA%/Hytale/UserData/Saves`
- **macOS**: `~/Library/Application Support/Hytale/UserData/Saves`

## License

MIT License - see [LICENSE](LICENSE) file for details.
