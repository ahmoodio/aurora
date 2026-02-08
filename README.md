# Aurora

Aurora is a Wayland-first GTK4/libadwaita GUI for Arch Linux package management. It combines official repository packages via pacman and AUR packages via yay/paru, while keeping the GUI unprivileged and delegating root actions to a strict pkexec helper.

## Features
- Repo and AUR search
- App details with AppStream metadata and screenshot carousel
- Transaction queue with mandatory review
- Streaming logs (stdout/stderr) with copy/save
- Installed packages (searchable)
- Updates page (check + apply)
- Settings for AUR helper and confirmation mode
- About dialog with project branding

## Architecture
- `src/core/` contains providers, transaction planning, appstream integration, caching, and command runner.
- `src/ui/` implements the UI pages, widgets, and app interactions.
- `src/bin/aurora-helper.rs` is the pkexec helper for privileged pacman operations.
- `assets/` includes the project logo and icon theme structure for installation.
- `resources/` provides the desktop file and AppStream metainfo.

## Security Model
- The GUI runs unprivileged.
- All privileged operations are executed via `pkexec` and `aurora-helper`.
- The helper enforces a strict whitelist of pacman operations and flags.
- No shell execution is used; all commands are executed with `std::process::Command` and explicit args.
- `--noconfirm` is only used if explicitly enabled in Settings.

## AppStream
Aurora uses `appstreamcli` to retrieve summaries, icons, and screenshots. Screenshots are cached at:

```
~/.cache/aurora/screenshots/
```

Missing metadata is handled gracefully with fallbacks.

## Branding
The logo is pulled from the `yay-gui-manager` GitHub repository and installed under the hicolor icon theme:

- `assets/icons/hicolor/scalable/apps/io.github.ahmoodio.aurora.png`
- `assets/icons/hicolor/256x256/apps/io.github.ahmoodio.aurora.png`

The logo is shown on the Home page, used as the application icon, and displayed in the About dialog.

## Build

```
cargo build --release
```

## Run (Development)

```
cargo run --bin aurora
```

For `pkexec` to find the helper during development, add the binary to PATH or install it system-wide.

## Install (Local)

```
sudo install -Dm755 target/release/aurora /usr/bin/aurora
sudo install -Dm755 target/release/aurora-helper /usr/bin/aurora-helper
sudo install -Dm644 resources/io.github.ahmoodio.aurora.desktop /usr/share/applications/io.github.ahmoodio.aurora.desktop
sudo install -Dm644 resources/io.github.ahmoodio.aurora.metainfo.xml /usr/share/metainfo/io.github.ahmoodio.aurora.metainfo.xml
sudo install -Dm644 assets/icons/hicolor/256x256/apps/io.github.ahmoodio.aurora.png /usr/share/icons/hicolor/256x256/apps/io.github.ahmoodio.aurora.png
sudo install -Dm644 assets/icons/hicolor/scalable/apps/io.github.ahmoodio.aurora.png /usr/share/icons/hicolor/scalable/apps/io.github.ahmoodio.aurora.png
```

Launch:

```
aurora
```
