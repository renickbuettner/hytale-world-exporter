# Hytale World Exporter

<p align="center">
  <img src="assets/app_icon_256.png" alt="Hytale World Exporter Icon" width="128">
</p>

A cross-platform desktop application for backing up and restoring Hytale game worlds.

## Features

- 🖥️ Cross-platform (Linux, macOS and Windows)
- 🌍 View and select worlds with details (size, last played)
- 🗜️ Export worlds as ZIP with optional logs/backups exclusion
- 📥 Import/restore worlds from ZIP backups
- 📋 View server logs with syntax highlighting (WARN/ERROR)
- 📦 Manage existing world backups
- 🌐 Localized (English & German, auto-detected)

## Installation

### Download

<p align="center">
  <a href="https://github.com/renickbuettner/hytale-world-exporter/releases/download/build-de6edad/Hytale-World-Exporter-Windows.zip">
    <img src="assets/download_win.png" alt="Download for Windows" height="120">
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/renickbuettner/hytale-world-exporter/releases/download/build-e278890/Hytale-World-Exporter-signed.zip">
    <img src="assets/download_mac.png" alt="Download for macOS" height="120">
  </a>
</p>

For Linux, see the Building from Source instructions.

I can provide a signed macOS app build, find the most recent link through the download button. 
Download alternate versions from the [GitHub Releases](https://github.com/renickbuettner/hytale-world-exporter/releases).

### Building from Source

```bash
git clone https://github.com/renickbuettner/hytale-world-exporter.git
cd hytale-world-exporter
cargo build --release
```

### macOS: Open app by bypassing Gatekeeper

Since a usual app build is not signed with an Apple Developer ID yet, macOS Gatekeeper may block it. To open the app:

1. Open Terminal
2. Run the following command, replacing the path with your app's path. You can drag-and-drop the app into the Terminal to get the correct path.
3. ```bash
   xattr -cr "/path/to/Hytale World Exporter.app"
   ```
4. Now try opening the app again.

## World Paths

- **Windows**: `%APPDATA%/Hytale/UserData/Saves`
- **macOS**: `~/Library/Application Support/Hytale/UserData/Saves`
- **Linux**: `~/.var/app/com.hypixel.HytaleLauncher/data/Hytale/UserData/Saves/`

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Screenshots

<p align="center">
  <img src="assets/screenshots/macos_01_backups.png" alt="Backups Tab" width="600">
</p>

<p align="center">
  <img src="assets/screenshots/macos_02_logs.png" alt="Logs Tab" width="600">
</p>

<p align="center">
<img width="818" height="660" alt="image" src="https://github.com/user-attachments/assets/bf82030c-7472-49ad-a443-c2802e041b70" />
</p>

