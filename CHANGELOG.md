# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## 0.16.1 - 2025-07-28

### Changed

- Remove dependency for Native TLS and OpenSSL on Linux
- Update Dependencies

## 0.16.0 - 2025-07-12

### Changed

- Use CLI arg --config for configuration directory and themes
- Remove write configuration CLI argument
- Update Dependencies

## 0.15.0 - 2025-03-28

### Added

- Filter and cycle through untagged items

### Changed

- Update rust edition to 2024 + Adjust CI Checks

## 0.14.0 - 2025-01-27

### Added

- Configuration to set the extension of the temporary file used with the external editor.
- Custom themes on user level & Styles improvements

## 0.13.1 - 2024-12-08

### Added

- A new FreeBSD port was published for tui-journal.
- Config path via CLI & State Path configurable & Fix default state path

## 0.13.0 - 2024-11-25

### Added

- Smart Case for Filter & Fuzzy Finder

### Changed

- Update dependencies & Simplified versions

### Fixed

- Fix Homebrew GitHub Action
- Increase Min Rust Version & Include minimal supported rust version in CI

## 0.12.1 - 2024-11-10

### Added

- GitHub Action to Release on Homebrew Tap

## 0.12.0 - 2024-09-27

### Added

- Provide Configuration for entries datum visibility
- Support Cycling Through Tags via Keybinding

## 0.11.0 - 2024-09-08

### Added

- Colored Tags to Journals, generated and assigned automatically

## 0.10.0 - 2024-09-01

### Added

- History Management: Provide Undo & Redo Functionalities.

### Changed

- Update Dependencies & Fix Crossterm and Raratui Breaking Changes

## 0.9.1 - 2024-07-20

### Added

- Basic Emacs keybindings to Editor Help Popup
- Show the selected index and entry count.

## 0.9.0 - 2024-05-30

### Added

- Optionally Sync Clipboard between Built-in Editor and the Operating System

## 0.8.3 - 2024-04-28

### Changed

- Added `--locked` to Install Instruction & Update dependencies

### Fixed

- Remove forgotten unused sort calls from app.
- Specify arm target on mac build and release

## 0.8.2 - 2024-02-01

### Fixed

- Change Help popup shortcut on Windows

## 0.8.1 - 2024-01-19

### Added

- Rust minimum supported version to cargo manifest

### Fixed

- Migrate changelog tooling from deprecated `-T` option

## 0.8.0 - 2024-01-16

### Added

- Add Default suggested priority while creating journals
- Add an optional priority field to the journals
- Add sort Functionality for journals
- App State (Sorting and Full-Screen Options) will be persisted
- CLI Sub-Command to assign priority to journals
- Go to Top/Bottom Journal & Page Up/Down Commands
- Sort key-map to app main footer

### Changed

- Skip Rendering the UI on Key-Release & Key-Repeat Events

## 0.7.0 - 2024-01-05

### Added

- Add full screen mode to main app view

### Changed

- Improve changelog creation logic (#279)
- Main Footer Height is calculated dynamically
- Use native rust support for async traits

### Fixed

- App will be redrawn on non-handled input
- Fix changelog creation logic
- Fix changelog tooling invocation
- Fix changelog assembly fetch depth

## 0.6.1 - 2024-01-01

### Changed

- Update Ratatui to 0.25 + Update dependencies - Ratatui has been updated in the app and in textarea to version 0.25. - Small code changes has been done due to breaking changes. - Cargo lock has been updated so we don't have references to old versions there.

### Fixed

- Ignore key events of types other than press

## 0.6.0 - 2023-12-16

### Added

- Support to Copy, Cut and Paste text between Built-in Editor and OS Clipboard

## 0.5.1 - 2023-11-28

### Added

- Navigation between journals popup inputs via arrow keys

### Fixed

- Letter V can't be typed in built-in editor insert mode

## 0.5.0 - 2023-11-26

### Added

- Horizontal Scrollbar to the Editor
- Visual Mode to Built-in Editor

### Changed

- Upgrade Crates Ratatui & Tui-Textarea with breaking changes

### Fixed

- Release notes will be always generated in release GitHub action

## 0.4.0 - 2023-10-14

### Added

- Autosave Option for External Editor

### Fixed

- - Optimizations for size calculation for the popup size to fit the footer on almost each terminal size - Correcting the keybindings in the footer text
- Write configurations CLI argument creates needed directories

## 0.3.3 - 2023-10-08

### Added

- Scrollbar for keybindings Overview Popup
- Scrollbar to Journals Editor
- Scrollbar to Journals' list

### Fixed

- Release GitHub Action creates binaries for Linux, MacOs and Windows

## 0.3.2 - 2023-10-02

### Added

- Add installation instructions for Alpine Linux
- Unit Tests for UI Functions
- Unit Tests for the main App Logic

### Changed

- Migrate to Ratatui & Update Crossterm to 27

### Fixed

- Attach built binaries to the release
- Input Boxes Border Colors

## 0.3.1 - 2023-08-16

### Changed

- Optimization for the app main loop
- Release GitHub Action
- Replace panic with compile_error in build script

### Fixed

- Fix SQLite connection string path on Windows

## 0.3.0 - 2023-07-13

### Added

- Add package name to changelog assembly workflow
- Fuzzy Finder Demo
- Fuzzy Finder for Journals

### Changed

- Call Docker images of linters directly

### Fixed

- Fix Release action using MakeFile
- Prevent release generation in forks

## 0.2.0 - 2023-06-24

### Added

- Create Cleanup Cache GitHub Action
- Search Functions in Filter
- Select-Tags Popup in Create/Edit Journals Prompt
- Tags (Categories) for journal + Filter Function
- Wiki Documentation
- automatic version increment and CFF self-maintenance

### Changed

- Add Filter Keybindings To Main Footer
- Improvements On Release Action Pipeline
- Keep journals list in focus after closing the external editor
- Update app Preview in README

## 0.1.4 - 2023-06-11

### Added

- CITATION.cff
- Edit current journal content in external editor
- Export Import functions for multiple journals
- Export current journal's content
- Multi-selection for journals
- Release CD action
- Tabs and scrolling to help popup
- Configure changelog maintenance utilities

### Changed

- Add NetBSD
- Add section to install the dependency OpenSSL development package on Linux
- CI Improvements
- Enhance render loop
- Help popup improvements
- Make creating new entry more user-friendly postponing the validation at initialisation
- Option to Use Git Configured Editor as External editor
- [Documentation] Create README Badges
- bump baptiste0928/cargo-install to v2.1.0
- improve GHA linting speed dramatically
- settings Fixes & improvements

### Fixed

- Editor will be cleared after deleting entries if there are no entries left
- Export journal file extension
- Setting back-end path from CLI
- Synchronizing problems with sqlite back-end
- Fix changelog creation bugs
