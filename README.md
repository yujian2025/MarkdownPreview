<div align="center">

# 📝 Markdown Preview

**A fast, minimal markdown preview reader with live reload**

*Lightweight alternative to VS Code / Typora markdown plugins — just save and see.*

![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.2.0-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey.svg)

[Quick Start](#quick-start) •
[Features](#features) •
[Installation](#installation) •
[Usage](#usage) •
[Build](#build-from-source)

---

</div>

## Quick Start

```bash
# One command (cross-platform)
cargo install markdownpreview

# Then run:
markdownpreview                        # Browser file picker mode
markdownpreview README.md              # Direct preview
markdownpreview README.md -t dark      # Dark theme
```

Or download from [Releases](https://github.com/yujian2025/MarkdownPreview/releases).

## Features

| Feature | Description |
|---------|-------------|
| 🖱️ **File Picker Mode** | Run without args — drag & drop, or browse files in browser |
| ⚡ **Live Reload** | Instant preview updates on file save via AJAX polling |
| 🌙 **Dark / Light Themes** | `-t dark` or `-t light` for comfortable reading |
| 📑 **Multi-Tab UI** | Open multiple files simultaneously, switch via tabs (WPS Office style) |
| 🪶 **Minimal & Fast** | ~0.8 MB binary, <100ms startup, ~5 MB RAM |
| 📋 **Full Markdown** | Tables, code blocks, task lists, blockquotes, strikethrough |
| 🌐 **Browser Preview** | Works in any modern browser |
| 🖨️ **Print / PDF Export** | Built-in print button, supports "Save as PDF" |
| 🌏 **中文 / English** | Built-in bilingual UI, toggle with one click |
| 🖥️ **System Tray** | Minimizes to tray (Windows only, `tray` feature) |
| 🔄 **Auto-Start** | `--install` to register on boot (Windows only) |
| 🐧 **Cross-Platform** | Works on Windows, Linux, macOS |
| 🔧 **No Dependencies** | Standalone binary — no Node.js, no Python, no IDE |

## Installation

### Option 1: Install via Cargo (all platforms)

```bash
cargo install markdownpreview
```

- **Windows**: Full features including system tray icon
- **Linux / macOS**: Server-only mode (no tray, runs until Ctrl+C)

### Option 2: Download Binary

Download from [Releases](https://github.com/yujian2025/MarkdownPreview/releases).

### Option 3: Build from Source

```bash
git clone https://github.com/yujian2025/markdownpreview.git
cd markdownpreview

# Windows (with system tray)
cargo build --release

# Linux / macOS (server only, no tray)
cargo build --release --no-default-features
```

**Prerequisites:** [Rust](https://rustup.rs/) 1.80+

## Cross-Platform Notes

| Platform | Tray Icon | Auto-Start | Ctrl+C Handling |
|----------|-----------|------------|------------------|
| Windows ✅ | ✅ System tray icon | ✅ `--install` | ✅ Via tray Exit |
| Linux ✅ | ❌ (server only) | ❌ | ✅ Via Ctrl+C |
| macOS ✅ | ❌ (server only) | ❌ | ✅ Via Ctrl+C |

On Linux and macOS, the program runs as a terminal server. Press **Ctrl+C** to stop.

## Usage

### Browser File Picker Mode (recommended)

```bash
markdownpreview
```
Browser opens automatically. Drag & drop `.md` files anywhere into the window, or click the Open button in the toolbar.

### Direct Preview Mode

```bash
markdownpreview 文档.md
markdownpreview doc1.md doc2.md    # Open multiple files
```

### CLI Options

```
Usage: markdownpreview [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Markdown file(s) to preview (optional — uses file picker if omitted)

Options:
  -p, --port <PORT>    Port for preview server [default: 8080]
  -t, --theme <THEME>  Theme: light or dark [default: light]
  -n, --no-open        Do not open browser automatically
      --install        Register auto-start on boot (Windows only, tray feature)
      --uninstall      Remove auto-start registration
  -h, --help           Print help
  -V, --version        Print version
```

### Examples

```bash
markdownpreview                          # Browser file picker
markdownpreview README.md -t dark        # Dark theme preview
markdownpreview doc1.md doc2.md          # Multiple files in tabs
markdownpreview -p 9000                  # Custom port
```

## Multi-Tab Interface

The v0.2.0 UI features a WPS Office-style tabbed interface:

```
┌──────────────────────────────────────────────────────┐
│  [M] Markdown Preview  [📂 打开] [🖨 打印]    中/EN  │
├──────────────────────────────────────────────────────┤
│  [📄 doc1.md ✕]  [📄 doc2.md ✕]  [+]                │  ← Click tabs to switch
├──────────────────────────────────────────────────────┤
│                     Preview Content                    │  ← Live reload on save
└──────────────────────────────────────────────────────┘
```

- **Click a tab** → switch to that file
- **Click ✕** → close the file and tab
- **Click +** → open a new file (browse or drag & drop)
- **🖨️ Print** → prints the currently active tab's content

## System Tray (Windows Only)

When running on Windows with the default `tray` feature, a **system tray icon** is shown:

| Action | Result |
|--------|--------|
| **Right-click → Open Browser** | Opens the preview page in browser |
| **Right-click → Exit** | Stops the server and exits |

On Linux/macOS, the server runs in the terminal. Press **Ctrl+C** to exit.

## Language

The UI supports **中文** (default) and **English**:

| How to switch |
|---------------|
| Click `中 / EN` button (top-right of toolbar) |
| Preference saved in browser `localStorage` |

## Supported Markdown

- ✅ **Headings** `# h1` ~ `###### h6`
- ✅ **Bold**, *Italic*, ~~Strikethrough~~
- ✅ **Inline code** `` `code` `` and **fenced code blocks** ``` ```rust ```
- ✅ **Tables** with header and cell borders
- ✅ **Ordered** `1.`, **unordered** `-`, and **task lists** `[x]`
- ✅ **Blockquotes** with nesting
- ✅ **Links** and **images**
- ✅ **Horizontal rules**
- ✅ **Smart punctuation** (curly quotes, em-dashes)

## How It Works

```
┌─────────────┐     ┌──────────────┐     ┌───────────┐
│  Markdown   │────▶│  tiny_http   │────▶│  Browser  │
│  File (.md) │     │   Server     │     │  Preview  │
└─────────────┘     └──────┬───────┘     └───────────┘
       │                    │                      ▲
       │ file change        │ version check        │ AJAX poll
       ▼                    ▼                      │
┌─────────────┐     ┌──────────────┐     ┌───────────┘
│   notify    │────▶│   Version    │────▶
│  (watcher)  │     │   Counter    │
└─────────────┘     └──────────────┘
```

1. **pulldown-cmark** parses markdown → HTML
2. **tiny_http** serves the rendered page
3. **notify** watches the file for changes
4. **Browser JS** polls `/check` every 1.5s
5. On change, HTML is fetched via `/content` and injected — **no page reload**

## Project Structure

```
markdownpreview/
├── Cargo.toml              # Rust project config & dependencies
├── src/
│   ├── main.rs             # CLI entry, tray (Windows), signal handling (cross-platform)
│   ├── renderer.rs         # Markdown → HTML renderer + app page
│   ├── server.rs           # tiny_http server + file watching
│   └── lib.rs              # Public API exports
├── templates/
│   └── app.html            # Tabbed multi-file UI (SPA)
├── tests/
│   └── integration.rs      # 19 integration tests
├── examples/
│   └── sample.md           # Demo markdown file
├── README.md               # This file
├── LICENSE                 # MIT License
└── .gitignore
```

## Performance

| Metric | Value |
|--------|-------|
| Binary size (release, stripped) | ~880 KB |
| Startup time | < 100 ms |
| Memory usage | ~5 MB |
| Poll interval | 1.5 seconds |
| Max file size | No limit |

## Development

```bash
# Check
cargo check

# Run tests (19 tests)
cargo test

# Build release (Windows with tray)
cargo build --release

# Build release (Linux/macOS, server only)
cargo build --release --no-default-features

# Run with sample
cargo run -- examples/sample.md

# Run file picker mode
cargo run --
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [pulldown-cmark](https://crates.io/crates/pulldown-cmark) | 0.11 | Markdown → HTML parser |
| [tiny_http](https://crates.io/crates/tiny_http) | 0.12 | Lightweight HTTP server |
| [notify](https://crates.io/crates/notify) | 6 | File system watcher (live reload) |
| [clap](https://crates.io/crates/clap) | 4 | CLI argument parsing |
| [serde_json](https://crates.io/crates/serde_json) | 1 | JSON parsing for file uploads |
| [ctrlc](https://crates.io/crates/ctrlc) | 3 | Cross-platform Ctrl+C handler |
| [winapi](https://crates.io/crates/winapi) | 0.3 | Windows system tray API (optional) |
| [open](https://crates.io/crates/open) | 5 | Open browser URL |

## Why Markdown Preview?

**VS Code extensions / Typora / Obsidian** are great, but:

| Tool | Startup | RAM | Standalone | Cross-Platform |
|------|---------|-----|------------|----------------|
| VS Code + plugin | 3-10s | 200-500 MB | ❌ Editor required | ✅ |
| Typora | 1-2s | 80-150 MB | ⚠️ Paid | ✅ |
| Obsidian | 2-5s | 150-300 MB | ❌ Vault required | ✅ |
| **Markdown Preview** | **< 0.1s** | **~5 MB** | **✅ Just the binary** | **✅ Windows/Linux/macOS** |

## License

MIT — see [LICENSE](./LICENSE).

---

<div align="center">
  <sub>Built with ❤️ and 🦀 Rust</sub>
  <br>
  <sub>📝 <a href="https://yujian2025.blog.csdn.net">Blog</a> • 
  🐙 <a href="https://github.com/yujian2025">GitHub</a></sub>
</div>