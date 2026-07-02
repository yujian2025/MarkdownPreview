# Markdown Preview User Manual

## Overview

`markdownpreview` is a fast, minimal markdown preview reader with live reload. It watches a markdown file and instantly updates the browser preview when you save changes — no plugin, no IDE, no heavy runtime required.

## Quick Start

```bash
# Preview a markdown file
markdownpreview README.md

# Use a different port
markdownpreview README.md -p 3000

# Dark theme
markdownpreview README.md -t dark

# Don't open browser automatically
markdownpreview README.md -n
```

## Installation

### From Source

```bash
git clone https://github.com/atomcode/markdownpreview.git
cd markdownpreview
cargo build --release
# Binary at ./target/release/markdownpreview.exe
```

### Add to PATH

Copy the binary to a directory in your `PATH`, or add this alias:

```bash
alias markdownpreview="/path/to/markdownpreview"
```

## Features

### Live Reload

When you edit and save the markdown file, the preview updates automatically **without page reload** — the content area smoothly refreshes via AJAX polling, preserving scroll position.

### Themes

Two built-in themes:

| Flag | Description |
|------|-------------|
| `-t light` (default) | Light background, GitHub-like styling |
| `-t dark` | Dark background, low-light friendly |

### Custom Port

If port 8080 is in use, specify a different one:

```bash
markdownpreview file.md -p 9000
```

### No-Browser Mode

Useful for headless servers or when you want to manually open the URL:

```bash
markdownpreview file.md -n
# Preview at http://127.0.0.1:8080
```

## Usage Examples

### Preview a README while writing

```bash
markdownpreview README.md -t dark
```

Opens your browser with a dark-themed preview. Save `README.md` — the preview updates.

### Include in a build pipeline

```bash
markdownpreview docs/manual.md -p 5000 -n
# Then open http://127.0.0.1:5000 manually
```

### Compare light vs dark

```bash
# Terminal 1
markdownpreview doc.md -p 8080 -t light

# Terminal 2  
markdownpreview doc.md -p 8081 -t dark
```

## CLI Reference

```
Usage: markdownpreview [OPTIONS] <FILE>

Arguments:
  <FILE>  Markdown file to preview

Options:
  -p, --port <PORT>    Port for the preview server [default: 8080]
  -t, --theme <THEME>  Theme: light or dark [default: light]
  -n, --no-open        Do not open browser automatically
  -h, --help           Print help
  -V, --version        Print version
```

## Supported Markdown

- Headings (h1–h6)
- Bold, italic, strikethrough
- Inline code & fenced code blocks (no language, or any language)
- Tables (GitHub-flavored)
- Ordered & unordered lists
- Task lists (checkboxes)
- Blockquotes (nested supported)
- Links & images
- Horizontal rules
- Smart punctuation (curly quotes, em-dashes)

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  Markdown   │────▶│  File       │────▶│  HTTP Server │
│  File (.md) │     │  Watcher    │     │  (tiny_http) │
└─────────────┘     └─────────────┘     └──────┬───────┘
                                                │
                                          ┌─────▼───────┐
                                          │  Browser    │
                                          │  (Preview)  │
                                          └─────────────┘
```

1. **File Watcher** (on a background thread) monitors the markdown file for changes
2. **HTTP Server** (on another thread) serves the rendered HTML
3. **Browser** polls the server every 1.5s for content updates
4. On file save, the watcher increments a version counter → next poll fetches new HTML

## Performance

| Metric | Value |
|--------|-------|
| Binary size (release) | ~2–3 MB |
| Startup time | < 100ms |
| Memory usage | ~5–10 MB |
| Poll interval | 1.5 seconds |
| Max file size | No limit (renders in chunks) |

## Troubleshooting

### Port already in use

```bash
# Find the process
netstat -ano | findstr :8080

# Kill it
taskkill /PID <PID> /F

# Or use a different port
markdownpreview file.md -p 8081
```

### Browser doesn't open

```bash
markdownpreview file.md -n
# Manually open http://127.0.0.1:8080
```

### File not found

Ensure the path is correct. Use an absolute path if needed:

```bash
markdownpreview "C:\Users\me\docs\readme.md"
```

## License

MIT License — see [LICENSE](../LICENSE).
