<div align="center">

# 📝 Markdown Preview

**A fast, minimal markdown preview reader with live reload**

*Lightweight alternative to VS Code / Typora markdown plugins — just save and see.*

![Rust](https://img.shields.io/badge/Rust-1.96+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)

[Quick Start](#quick-start) •
[Features](#features) •
[Installation](#installation) •
[Usage](#usage) •
[Build](#build-from-source)

---

</div>

## Quick Start

```bash
# 网页模式：运行后浏览器打开文件选择页，拖拽/选择/输入路径均可
markdownpreview

# 直接预览模式
markdownpreview README.md

# 暗色主题
markdownpreview README.md -t dark
```

## Features

| Feature | Description |
|---------|-------------|
| 🖱️ **File Picker Mode** | Run without args — drag-and-drop, browse, or paste path in browser |
| ⚡ **Live Reload** | Instant preview updates on file save via AJAX polling — no page reload |
| 🌙 **Dark / Light Themes** | `-t dark` or `-t light` for comfortable reading |
| 🪶 **Minimal & Fast** | ~0.8 MB binary, <100ms startup, ~5 MB RAM |
| 📋 **Full Markdown** | Tables, code blocks, task lists, blockquotes, strikethrough |
| 🌐 **Browser Preview** | Works in any modern browser |
| 🖨️ **Print / PDF Export** | Built-in print button, supports "Save as PDF" |
| 🌏 **中文 / English** | Built-in bilingual UI, toggle with one click |
| 🖥️ **System Tray** | Minimizes to Windows tray, right-click for menu |
| 🔄 **Auto-Start** | `--install` to register on boot (Windows) |
| 🔧 **No Dependencies** | Standalone binary — no Node.js, no Python, no IDE |

## Installation

### Download (Windows)

Download the latest `markdownpreview.exe` from [Releases](https://github.com/yujian2025/markdownpreview/releases).

### Build from Source

```bash
git clone https://github.com/yujian2025/markdownpreview.git
cd markdownpreview
cargo build --release
# Binary: ./target/release/markdownpreview.exe
```

**Prerequisites:** [Rust](https://rustup.rs/) 1.96+

## Usage

### 网页模式（推荐）

```bash
markdownpreview
```
浏览器自动打开文件选择页，支持：
- **拖拽** .md 文件到虚线区域
- **点击 Browse Files** 选择文件
- **输入完整路径** 后点 Open（支持自动刷新）

### 直接预览模式

```bash
markdownpreview 文档.md
```

### CLI Options

```
Usage: markdownpreview [OPTIONS] [FILE]

Arguments:
  [FILE]  Markdown file to preview (optional — uses file picker if omitted)

Options:
  -p, --port <PORT>    Port for preview server [default: 8080]
  -t, --theme <THEME>  Theme: light or dark [default: light]
  -n, --no-open        Do not open browser automatically
      --install        Register auto-start on boot (Windows)
      --uninstall      Remove auto-start registration
  -h, --help           Print help
  -V, --version        Print version
```

### Examples

```bash
markdownpreview                          # 网页文件选择器
markdownpreview README.md -t dark        # 暗色预览
markdownpreview doc.md -p 9000           # 自定义端口
markdownpreview --install                # 安装开机自启
markdownpreview --uninstall              # 卸载开机自启
```

## System Tray

When running, the program creates a **system tray icon** (Windows taskbar notification area):

| Action | Result |
|--------|--------|
| **Right-click → Open Browser** | Opens the preview page in browser |
| **Right-click → Exit** | Stops the server and exits |
| **Program runs in background** | Server keeps running until you exit via tray |

## Language

The UI supports **中文** (default) and **English**:

| Page | How to switch |
|------|---------------|
| **File picker page** | Click `中 / EN` button (top-right) |
| **Preview page** | Double-click `← 打开文件` button |

Language preference is saved in browser `localStorage`.

## Toolbar Buttons

| Button | Action |
|--------|--------|
| 🟢 | Green dot — indicates server is running |
| ← Open File | Return to file picker to open another file |
| 🖨️ Print | Open browser print dialog (supports "Save as PDF") |
| ✕ | Close current document, return to file picker |

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
│   ├── main.rs             # CLI entry, system tray, auto-start
│   ├── renderer.rs         # Markdown → HTML renderer + landing page
│   ├── server.rs           # tiny_http server + file watching
│   └── lib.rs              # Public API exports
├── templates/
│   └── preview.html        # Standalone HTML template for preview page
├── tests/
│   └── integration.rs      # 21 integration tests
├── examples/
│   └── sample.md           # Demo markdown file
├── docs/
│   └── manual.md           # Full user manual
├── README.md               # This file
├── LICENSE                 # MIT License
└── .gitignore
```

## Performance

| Metric | Value |
|--------|-------|
| Binary size (release, stripped) | ~820 KB |
| Startup time | < 100 ms |
| Memory usage | ~5 MB |
| Poll interval | 1.5 seconds |
| Max file size | No limit |

## Development

```bash
# Check
cargo check

# Run tests (21 tests)
cargo test

# Build release
cargo build --release

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
| [winapi](https://crates.io/crates/winapi) | 0.3 | Windows system tray API |
| [open](https://crates.io/crates/open) | 5 | Open browser URL |

## Why Markdown Preview?

**VS Code extensions / Typora / Obsidian** are great, but:

| Tool | Startup | RAM | Standalone |
|------|---------|-----|------------|
| VS Code + plugin | 3-10s | 200-500 MB | ❌ Editor required |
| Typora | 1-2s | 80-150 MB | ⚠️ Paid |
| Obsidian | 2-5s | 150-300 MB | ❌ Vault required |
| **Markdown Preview** | **< 0.1s** | **~5 MB** | **✅ Just the .exe** |

## License

MIT — see [LICENSE](./LICENSE).

---

<div align="center">
  <sub>Built with ❤️ and 🦀 Rust</sub>
  <br>
  <sub>📝 <a href="https://yujian2025.blog.csdn.net">Blog</a> • 
  🐙 <a href="https://github.com/yujian2025">GitHub</a></sub>
</div>
