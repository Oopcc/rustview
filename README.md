# RustView

Tiny, fast, cross-platform image viewer written in Rust.
Opens **GIF** and **WebP** (including animated), PNG, JPEG, BMP, ICO, TIFF, TGA, QOI, PNM.

[Русская версия](README.ru.md)

## Features

- Double-click an image in Explorer: RustView opens it and lets you flip through the other images in the same folder
- Arrow keys to navigate (natural sort: `img2` comes before `img10`)
- Zoom with mouse wheel (around the cursor), pan by dragging, double click toggles fit / 100%
- Animated GIF / WebP: play / pause, frame-by-frame stepping
- Rotate view, fullscreen, drag & drop a file onto the window
- Format is detected by file content, so a WebP renamed to `.jpg` still opens
- One small binary, no runtime dependencies

## Shortcuts

| Key | Action |
|---|---|
| `Left` / `Right`, `PgUp` / `PgDn` | previous / next image |
| `Home` / `End` | first / last image |
| Mouse wheel, `Up` / `Down`, `+` / `-` | zoom |
| Drag, double click | pan, toggle 100% / fit |
| `0` / `1` | fit to window / actual size |
| `R` / `Shift+R` | rotate clockwise / counter-clockwise |
| `Space` | play / pause animation (static image: next) |
| `,` / `.` | previous / next animation frame |
| `F`, `F11`, `Enter` | fullscreen |
| `I` | toggle info bar |
| `H`, `F1` | help |
| `Esc` | exit fullscreen / close |

## Install (Windows)

Download `RustView-Setup-x.y.z.exe` from [Releases](../../releases) and run it.
The installer registers RustView for all supported formats and puts the app icon on associated files.

> Windows 10/11 does not let installers silently change your default apps.
> After installing, either tick the last checkbox in the installer (opens *Settings > Default apps*)
> or right-click any image > *Open with* > *Choose another app* > RustView > *Always*.
> Do this once per format (e.g. `.gif`, `.webp`).

A portable `rustview.exe` (no installation) is also attached to each release.

## Build from source

The code is fully cross-platform (Windows, Linux, macOS, ChromeOS via its Linux container). You need [Rust](https://rustup.rs) (stable, 1.81+).

```bash
git clone https://github.com/<you>/rustview
cd rustview
cargo run --release -- path/to/image.webp
cargo build --release          # binary: target/release/rustview(.exe)
```

Platform notes:

- **Windows**: `cargo build --release`; the icon is embedded automatically. To build the installer, install [Inno Setup 6](https://jrsoftware.org/isinfo.php) and run
  `"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\rustview.iss` -> `dist\RustView-Setup-<version>.exe`.
- **Linux**: needs Vulkan (Mesa) or OpenGL at runtime. Optional integration: copy `packaging/rustview.desktop` to `~/.local/share/applications/` and `assets/icon.png` to `~/.local/share/icons/hicolor/512x512/apps/rustview.png`, then pick RustView in "Open with".
- **ChromeOS**: enable *Linux development environment*, then follow the Linux steps in its terminal.
- **macOS**: `cargo build --release`; wrap in a `.app` with [cargo-bundle](https://github.com/burtonageo/cargo-bundle) if you want Finder associations.

## Releasing

Push a version tag; GitHub Actions builds everything and attaches it to the release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow (`.github/workflows/release.yml`) produces the Windows installer + portable exe and Linux / macOS archives.

## Project layout

```
src/main.rs                 the whole app (egui/eframe + image crate)
build.rs                    embeds assets/icon.ico into the Windows exe
installer/rustview.iss      Inno Setup script (file associations, icon)
assets/                     icon.png, icon.ico
packaging/rustview.desktop  Linux desktop entry
.github/workflows/          CI: build + release
```

## Adding a format

Add the extension to `EXTENSIONS` in `src/main.rs` (and to `installer/rustview.iss`), enable the matching codec feature of the `image` crate in `Cargo.toml`.

## License

MIT
