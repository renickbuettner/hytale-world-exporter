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

## Developer Documentation

### macOS Code Signing with Apple Developer Certificate

For distributing the app without security warnings, you need an Apple Developer account and certificate.

#### Prerequisites

1. **Apple Developer Account**: Enroll at [developer.apple.com](https://developer.apple.com) ($99/year)
2. **Developer ID Application Certificate**: Create in Apple Developer Portal

#### Setting up Certificate for CI/CD

1. **Export your certificate**:
   - Open **Keychain Access** on macOS
   - Find your "Developer ID Application" certificate
   - Right-click and select "Export..."
   - Save as `.p12` file with a password

2. **Configure GitHub Secrets**:
   
   Go to your repository Settings > Secrets and variables > Actions, and add:
   
   **For Code Signing (required):**
   
   - `MACOS_CERTIFICATE`: Base64-encoded certificate file
     ```bash
     base64 -i certificate.p12 | pbcopy
     ```
   
   - `MACOS_CERTIFICATE_PASSWORD`: Password used when exporting the certificate
   
   - `MACOS_KEYCHAIN_PASSWORD`: Any secure password for the temporary keychain
   
   - `MACOS_SIGNING_IDENTITY`: Your certificate identity (e.g., "Developer ID Application: Your Name (TEAM_ID)")
     ```bash
     # Find your identity:
     security find-identity -v -p codesigning
     ```

   **For Notarization (optional, recommended):**
   
   - `APPLE_ID`: Your Apple ID email address
   
   - `APPLE_TEAM_ID`: Your Team ID from developer.apple.com
   
   - `APPLE_APP_PASSWORD`: App-specific password
     - Create this at [appleid.apple.com](https://appleid.apple.com)
     - Go to **Security** > **App-Specific Passwords**
     - Generate a new password for "GitHub Actions"

3. **The build workflow will automatically**:
   - Use certificate-based signing if secrets are configured
   - Fall back to ad-hoc signing if no certificate is available
   - **Notarize the app** if Apple credentials are configured (APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD)
   - Skip notarization if credentials are not available
   - Ad-hoc signed apps work but require users to manually allow them in System Preferences

#### Notarization

Notarization is integrated into the build workflow and will be performed automatically when the required secrets are configured.

**Automatic Notarization (CI/CD)**

The GitHub Actions workflow automatically performs these steps:

1. **Sign** the app with your Developer ID Certificate
2. **Create** a ZIP archive
3. **Notarize** using `xcrun notarytool`:
   - Submit the ZIP archive to Apple
   - Wait for approval (typically 1-5 minutes)
4. **Staple** using `xcrun stapler`:
   - Attach the notarization ticket to the app
   - Re-package the ZIP with the notarized app

If notarization secrets (`APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD`) are not configured, this step will be skipped and only code signing will be performed.

**Manual Notarization (Local Builds)**

For local builds, you can manually notarize the app:

1. Create an App-Specific Password at [appleid.apple.com](https://appleid.apple.com)
2. Create ZIP archive:
   ```bash
   zip -r "Hytale-World-Exporter-macOS.zip" "Hytale World Exporter.app"
   ```
3. Submit for notarization using `xcrun notarytool`:
   ```bash
   xcrun notarytool submit "Hytale-World-Exporter-macOS.zip" \
     --apple-id "your@email.com" \
     --team-id "TEAM_ID" \
     --password "app-specific-password" \
     --wait
   ```
4. Staple the notarization ticket using `xcrun stapler`:
   ```bash
   unzip "Hytale-World-Exporter-macOS.zip"
   xcrun stapler staple "Hytale World Exporter.app"
   ```

For more information, see [Apple's Code Signing Guide](https://developer.apple.com/support/code-signing/) and [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution).

## License

MIT License - see [LICENSE](LICENSE) file for details.
