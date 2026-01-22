# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Saten is a Rust-based image viewer for archive files (RAR4, RAR5, ZIP, CBR, CBZ) built with the Iced GUI framework (v0.13.1). It supports single and double-page display modes, image rotation, flip mode, and single-instance functionality via IPC.

## Build Commands

```bash
# Syntax check
cargo check

# Run all tests
cargo test

# Build debug version
cargo build

# Build release version
cargo build --release

# Run application
cargo run -- path/to/archive.rar

# Full build pipeline (check → test → release build)
./build_test.sh
```

## Architecture

The codebase follows a clean **MVC (Model-View-Controller)** pattern:

### Model Layer (`src/model/`)
- **app_state.rs**: Core `AppState` struct - all application state and business logic
- **archive_manager.rs**: Archive file handling and extraction
- **image_manager.rs**: Image data processing and format validation
- **page_manager.rs**: Page navigation and display mode logic

### View Layer (`src/view/`)
- **app_view.rs**: Main application view composition
- **image_view.rs**: Image rendering components
- **layout.rs**: Reusable layout helpers

### Controller Layer (`src/controller/`)
- **app_controller.rs**: Central event dispatcher
- **keyboard_handler.rs**: Keyboard input processing
- **file_handler.rs**: File drop and loading

### Archive Readers
- **archive_reader.rs**: `ArcReader` trait defining the reader interface
- **reader_zip.rs**: ZIP format implementation
- **reader_rar4.rs**: RAR4 format implementation
- **reader_rar5.rs**: RAR5 format implementation
- **file_checker.rs**: Format detection via magic byte signatures

### Other Key Files
- **main.rs**: Application entry point, Iced GUI loop, message handling
- **ipc.rs**: Single-instance IPC via Unix Domain Socket (`~/.saten.sock`)
- **sort_filename.rs**: Natural sorting for filenames

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Arrow Left/Right | Previous/Next page |
| Arrow Up/Down | Previous/Next file |
| 1 / 2 | Single/Double page mode |
| R | Rotate image |
| F | Flip mode (swap left/right) |
| Z | Fullsize mode |
| I | Toggle overlay display |
| Home/End | First/Last page |
| Space/Backspace | Next/Previous page |

## Supported Formats

- ZIP: `.zip`, `.cbz`
- RAR4/RAR5: `.rar`, `.cbr`
- Single images: `.jpg`, `.png`, `.gif`, `.bmp`, `.webp`, `.tiff`

## Adding New Features

1. **State changes**: Add to `AppState` in `src/model/app_state.rs`
2. **User input**: Handle in `src/controller/keyboard_handler.rs`
3. **UI rendering**: Update views in `src/view/`
4. **New messages**: Add variants to the `Message` enum in `src/main.rs`

## Error Handling

Uses `thiserror` for error types with a central `ArchiveError` enum. Propagate errors via `Result<T, ArchiveError>`.

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run
```

Use `log::debug!()`, `log::info!()`, etc. for diagnostic output.
