# Hytale World Exporter

A simple cross-platform desktop application written in Rust that backs up Hytale game worlds by compressing them into a ZIP archive.

## Features

- 🖥️ Cross-platform support (macOS and Windows)
- 🎨 Simple and intuitive GUI using eframe/egui
- 🗜️ Automatic compression of all Hytale world folders
- 📅 Timestamped backup files (e.g., `hytale_worlds_2026-01-13_14-30-00.zip`)
- 💾 Saves backups to your Downloads folder
- 🇩🇪 German language interface
- ⚠️ Clear error messages for missing directories

## Requirements

- Rust 1.70 or later (for building from source)
- macOS or Windows operating system

## Installation

### Building from Source

1. Make sure you have [Rust installed](https://rustup.rs/)
2. Clone this repository:
   ```bash
   git clone https://github.com/renickbuettner/hytale-world-exporter.git
   cd hytale-world-exporter
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The executable will be in `target/release/hytale-world-exporter` (or `hytale-world-exporter.exe` on Windows)

## Usage

1. Launch the application
2. Click the "🗜️ Welten komprimieren" button
3. The application will:
   - Find your Hytale worlds folder
   - Compress all world data into a ZIP file
   - Save it to your Downloads folder with a timestamp
   - Display a success message with the file path or an error message if something went wrong

## Hytale World Paths

The application looks for Hytale worlds in the following locations:

- **Windows**: `%APPDATA%/Hytale/worlds`
- **macOS**: `~/Library/Application Support/Hytale/worlds`

## Technical Details

### Dependencies

- **eframe/egui**: Cross-platform GUI framework
- **zip**: ZIP file creation and compression
- **dirs**: Platform-specific directory paths
- **walkdir**: Recursive directory traversal
- **chrono**: Timestamp generation

### Project Structure

- `src/main.rs`: Main application code containing:
  - GUI implementation
  - World path detection
  - ZIP compression logic
  - Error handling

## Error Handling

The application handles common error scenarios:

- Missing Hytale worlds folder
- Missing Downloads folder
- File read/write errors
- Platform-specific path issues

All errors are displayed in German with user-friendly messages.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
