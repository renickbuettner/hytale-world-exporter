# macOS Code-Signierung - Anleitung

## Problem

macOS verhindert standardmäßig das Ausführen von Apps, die nicht von einem verifizierten Entwickler signiert sind. Dies führt zu Fehlermeldungen wie:

- "Die App kann nicht geöffnet werden, da der Entwickler nicht verifiziert werden kann"
- "macOS kann die App nicht auf Schadsoftware überprüfen"

## Kurzfristige Lösung für Benutzer

Wenn Sie die heruntergeladene App ausführen möchten:

### Methode 1: Rechtsklick-Menü (empfohlen)

1. **Rechtsklick** (oder Control-Klick) auf die App
2. Wählen Sie **"Öffnen"** aus dem Menü
3. Klicken Sie im Dialog auf **"Öffnen"**
4. Die App wird gestartet und kann danach normal geöffnet werden

### Methode 2: Systemeinstellungen

1. Öffnen Sie **Systemeinstellungen** > **Sicherheit** > **Allgemein**
2. Klicken Sie auf **"Dennoch öffnen"** neben der Meldung über die blockierte App
3. Bestätigen Sie mit **"Öffnen"**

## Langfristige Lösung für Entwickler

### Ist ein Zertifikat notwendig?

**Ja**, für eine professionelle Verteilung ohne Sicherheitswarnungen benötigen Sie:

1. **Apple Developer Account** (99 USD/Jahr)
2. **Developer ID Application Certificate**

### So erhalten Sie ein Zertifikat

#### Schritt 1: Apple Developer Account erstellen

1. Registrieren Sie sich bei [developer.apple.com](https://developer.apple.com)
2. Schließen Sie die Registrierung ab und bezahlen Sie die Jahresgebühr (99 USD)
3. Warten Sie auf die Freischaltung (kann 24-48 Stunden dauern)

#### Schritt 2: Zertifikat erstellen

1. Melden Sie sich im [Apple Developer Portal](https://developer.apple.com/account) an
2. Gehen Sie zu **Certificates, Identifiers & Profiles**
3. Klicken Sie auf das **+** Symbol bei Certificates
4. Wählen Sie **Developer ID Application** unter "Software"
5. Folgen Sie den Anweisungen zum Erstellen einer Certificate Signing Request (CSR):
   - Öffnen Sie **Keychain Access** (Schlüsselbundverwaltung)
   - Menü: **Keychain Access** > **Certificate Assistant** > **Request a Certificate from a Certificate Authority**
   - Geben Sie Ihre E-Mail-Adresse ein
   - Wählen Sie "Saved to disk"
6. Laden Sie die CSR-Datei hoch
7. Laden Sie das erstellte Zertifikat herunter und installieren Sie es per Doppelklick

#### Schritt 3: Zertifikat exportieren

1. Öffnen Sie **Keychain Access**
2. Finden Sie Ihr "Developer ID Application" Zertifikat
3. Rechtsklick > **Exportieren...**
4. Speichern Sie als `.p12` Datei mit einem sicheren Passwort
5. Bewahren Sie diese Datei und das Passwort sicher auf

### Was muss am Build-Prozess angepasst werden?

Der Build-Prozess wurde bereits aktualisiert und unterstützt:

1. **Automatische Ad-hoc-Signierung**: Funktioniert ohne Zertifikat
   - Benutzer müssen die App manuell in den Systemeinstellungen erlauben
   
2. **Zertifikat-basierte Signierung**: Wenn GitHub Secrets konfiguriert sind
   - Keine Sicherheitswarnungen für Benutzer

#### GitHub Secrets konfigurieren

Fügen Sie folgende Secrets in Ihren GitHub Repository-Einstellungen hinzu:

1. **MACOS_CERTIFICATE**: Base64-kodiertes Zertifikat
   ```bash
   base64 -i certificate.p12 | pbcopy
   ```

2. **MACOS_CERTIFICATE_PASSWORD**: Passwort der .p12-Datei

3. **MACOS_KEYCHAIN_PASSWORD**: Ein beliebiges sicheres Passwort

4. **MACOS_SIGNING_IDENTITY**: Ihre Zertifikat-Identität
   ```bash
   # Identität herausfinden:
   security find-identity -v -p codesigning
   # Beispiel: "Developer ID Application: Max Mustermann (ABC123XYZ)"
   ```

#### Lokale Builds signieren

Für lokale Entwicklung können Sie die App mit einer Ad-hoc-Signatur versehen:

```bash
# App-Bundle erstellen und signieren
codesign --force --deep --sign - "Hytale World Exporter.app"

# Signatur überprüfen
codesign --verify --verbose "Hytale World Exporter.app"
```

## Optional: Notarisierung

Für die beste Benutzererfahrung können Sie die App auch notarisieren lassen:

1. App-spezifisches Passwort erstellen bei [appleid.apple.com](https://appleid.apple.com)
2. Nach dem Signieren die App zur Notarisierung einreichen:
   ```bash
   xcrun notarytool submit "Hytale-World-Exporter-macOS.zip" \
     --apple-id "ihre@email.com" \
     --team-id "TEAM_ID" \
     --password "app-spezifisches-passwort" \
     --wait
   ```
3. Notarisierungs-Ticket an die App heften:
   ```bash
   xcrun stapler staple "Hytale World Exporter.app"
   ```

## Zusammenfassung

| Methode | Kosten | Benutzerfreundlichkeit | Aufwand |
|---------|--------|------------------------|---------|
| Keine Signierung | Kostenlos | ⚠️ Warnung + manuelle Freigabe | Minimal |
| Ad-hoc Signierung | Kostenlos | ⚠️ Warnung + manuelle Freigabe | Sehr gering |
| Entwickler-Zertifikat | 99 USD/Jahr | ⚠️ Warnung beim ersten Start | Mittel |
| Zertifikat + Notarisierung | 99 USD/Jahr | ✅ Keine Warnung | Hoch |

## Weitere Ressourcen

- [Apple Code Signing Guide](https://developer.apple.com/support/code-signing/)
- [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Creating Distribution-Signed Code](https://developer.apple.com/documentation/xcode/distributing-your-app-for-beta-testing-and-releases)
