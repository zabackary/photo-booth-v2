# photo-booth-v2

> A poorly named photo booth application

![photo_booth_demo](https://github.com/user-attachments/assets/92156f5b-1916-43cf-b1c3-65c7680ba3b2)

photo-booth-v2 is a configuration photo booth application supporting multiple backends for each option. It supports configurable theming, printers, email servers, etc.

I hereby license this repo under the GPLv3. Exceptions will be granted if you ask.

Recommended development run command:

```bash
RUST_LOG=photo_booth_v2=debug cargo run --release --features "fast_animations mock" -- --config config.example.json
```

### Backends

The default build (available on GitHub releases) includes the following backends:

- **email_gapps_script_webhook**: send email via a Google Apps Script webhook

- **filter_skin_softening**: a simple skin softening filter

- **printer_cups**: print photos using CUPS (requires CUPS installed on the system)

- **camera_nokhwa**: capture webcam images using the nokhwa crate

- **camera_gphoto2**: capture from cameras using gphoto2 (requires gphoto2 installed on the system)

- **storage_google_drive**: upload photos to Google Drive using the Google Drive API

- **storage_local_filesystem**: save photos to the local filesystem
