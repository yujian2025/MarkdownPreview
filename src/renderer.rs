use pulldown_cmark::{Options, Parser};

/// Render markdown text to HTML content (no wrapper).
pub fn render_content(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(input, options);
    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, parser);
    output
}

/// Render markdown text to a full HTML page with CSS styling, toolbar, and live-reload JS.
///
/// Uses a standalone HTML template (`templates/preview.html`) embedded via `include_str!`,
/// with simple string replacement for dynamic values. No `format!` escaping issues.
pub fn render_full_page(input: &str, theme: &str, filename: Option<&str>, port: u16) -> String {
    let content_html = render_content(input);
    let file_display = filename.filter(|s| !s.is_empty()).unwrap_or("未命名文档");
    let title = format!("Markdown Preview — {file_display}");
    let is_dark = theme == "dark";

    static TEMPLATE: &str = include_str!("../templates/preview.html");

    TEMPLATE
        .replace("__TITLE__", &title)
        .replace("__FILENAME__", file_display)
        .replace("__THEME__", theme)
        .replace("__VERSION__", env!("CARGO_PKG_VERSION"))
        .replace("__PORT__", &port.to_string())
        .replace("__CONTENT__", &content_html)
        // Theme colors
        .replace("__BG__", if is_dark { "#0d1117" } else { "#ffffff" })
        .replace("__FG__", if is_dark { "#e6edf3" } else { "#1f2328" })
        .replace("__MUTED__", if is_dark { "#8b949e" } else { "#656d76" })
        .replace("__BORDER__", if is_dark { "#30363d" } else { "#d0d7de" })
        .replace("__ACCENT__", if is_dark { "#58a6ff" } else { "#0969da" })
        .replace("__CODE_BG__", if is_dark { "rgba(110,118,129,0.4)" } else { "rgba(175,184,193,0.2)" })
        .replace("__PRE_BG__", if is_dark { "#161b22" } else { "#f6f8fa" })
        .replace("__TABLE_HDR__", if is_dark { "rgba(110,118,129,0.15)" } else { "rgba(208,215,222,0.3)" })
        .replace("__TABLE_ALT__", if is_dark { "rgba(110,118,129,0.06)" } else { "#f6f8fa" })
}

/// Render the landing page file picker (shown when no file is loaded).
/// Supports Chinese (default) and English language switching.
pub fn render_landing_page() -> String {
    r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Markdown Preview</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans SC", Helvetica, Arial, sans-serif;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
}

.card {
    background: #ffffff;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.1);
    padding: 48px 40px;
    width: 100%;
    max-width: 600px;
    text-align: center;
    position: relative;
}

/* Language toggle */
#langToggle {
    position: absolute;
    top: 12px; right: 12px;
    font-size: 12px;
    font-family: inherit;
    padding: 3px 10px;
    border: 1px solid #d0d7de;
    border-radius: 5px;
    background: transparent;
    color: #656d76;
    cursor: pointer;
    transition: all .15s;
}
#langToggle:hover { background: #eaeef2; }

.logo { font-size: 48px; margin-bottom: 8px; }
h1 { font-size: 28px; font-weight: 700; color: #1a2332; margin-bottom: 4px; }

.subtitle {
    font-size: 15px;
    color: #6b7a8d;
    margin-bottom: 32px;
}

/* Drop zone */
.drop-zone {
    border: 2px dashed #c3cfe2;
    border-radius: 12px;
    padding: 40px 20px;
    cursor: pointer;
    transition: all 0.25s ease;
    background: #f8faff;
    margin-bottom: 20px;
}
.drop-zone:hover, .drop-zone.dragover {
    border-color: #5b7cfa;
    background: #eef4ff;
}
.drop-zone-icon { font-size: 36px; margin-bottom: 8px; }
.drop-zone-text { font-size: 16px; color: #1a2332; font-weight: 500; }
.drop-zone-hint { font-size: 13px; color: #8a9baa; margin-top: 4px; }

.btn-browse {
    display: inline-block;
    padding: 10px 28px;
    background: #5b7cfa;
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
    margin-top: 8px;
    font-family: inherit;
}
.btn-browse:hover { background: #4a6ae0; }

.divider {
    display: flex;
    align-items: center;
    gap: 16px;
    margin: 24px 0;
    color: #8a9baa;
    font-size: 13px;
}
.divider::before, .divider::after {
    content: "";
    flex: 1;
    height: 1px;
    background: #e0e6ef;
}

.path-input-group {
    display: flex;
    gap: 8px;
    margin-top: 4px;
}
.path-input-group input {
    flex: 1;
    padding: 10px 16px;
    border: 2px solid #e0e6ef;
    border-radius: 8px;
    font-size: 14px;
    font-family: "SF Mono", "Fira Code", Consolas, monospace;
    color: #1a2332;
    outline: none;
    transition: border-color 0.2s;
}
.path-input-group input:focus { border-color: #5b7cfa; }
.path-input-group button {
    padding: 10px 24px;
    background: #1a2332;
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
    white-space: nowrap;
    font-family: inherit;
}
.path-input-group button:hover { background: #2a3a4a; }

/* Tips */
.examples {
    margin-top: 28px;
    padding: 16px;
    background: #f8faff;
    border-radius: 8px;
    text-align: left;
}
.examples-title {
    font-size: 12px;
    font-weight: 600;
    color: #8a9baa;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 8px;
}
.examples-list { list-style: none; }
.examples-list li {
    padding: 4px 0;
    font-size: 13px;
    color: #5b7cfa;
    cursor: pointer;
    transition: color 0.2s;
}
.examples-list li:hover { color: #4a6ae0; text-decoration: underline; }

.footer { margin-top: 28px; font-size: 12px; color: #a0b0c0; }

#status {
    display: none;
    margin-top: 12px;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 14px;
}
#status.success { display: block; background: #dff0d8; color: #3c763d; }
#status.error { display: block; background: #f2dede; color: #a94442; }
#status.loading { display: block; background: #d9edf7; color: #31708f; }

#file-input { display: none; }
</style>
</head>
<body>

<div class="card">
    <button id="langToggle">中 / EN</button>
    <div class="logo">📝</div>
    <h1>Markdown Preview</h1>
    <p class="subtitle" data-zh="打开 .md 文件，实时预览" data-en="Open a .md file to preview with live reload">打开 .md 文件，实时预览</p>

    <div class="drop-zone" id="dropZone">
        <div class="drop-zone-icon">📄</div>
        <div class="drop-zone-text" data-zh="拖拽 .md 文件到此处" data-en="Drag &amp; drop .md file here">拖拽 .md 文件到此处</div>
        <div class="drop-zone-hint" data-zh="或点击下方按钮浏览" data-en="or click the button below to browse">或点击下方按钮浏览</div>
        <button class="btn-browse" id="browseBtn" type="button" data-zh="浏览文件" data-en="Browse Files">浏览文件</button>
        <input type="file" id="file-input" accept=".md,.markdown,.txt">
    </div>

    <div class="divider" data-zh="或输入文件路径" data-en="or enter file path">或输入文件路径</div>

    <div class="path-input-group">
        <input type="text" id="pathInput" placeholder="C:\path\to\document.md" spellcheck="false">
        <button id="openBtn" type="button" data-zh="打开" data-en="Open">打开</button>
    </div>

    <div class="examples">
        <div class="examples-title" data-zh="💡 提示" data-en="💡 Tips">💡 提示</div>
        <ul class="examples-list">
            <li data-zh="输入完整路径可启用自动刷新" data-en="Enter full path to enable auto-reload on save">输入完整路径可启用自动刷新</li>
            <li data-zh="拖拽上传后不可自动刷新（可重新拖拽）" data-en="Drag &amp; drop: no auto-reload (re-drop to refresh)">拖拽上传后不可自动刷新（可重新拖拽）</li>
            <li data-zh="CLI 选项：-t dark / -p 3000" data-en="CLI options: -t dark / -p 3000">CLI 选项：-t dark / -p 3000</li>
        </ul>
    </div>

    <div id="status"></div>
    <div class="footer">
        markdownpreview v__VERSION__ ·
        <a href="https://yujian2025.blog.csdn.net" target="_blank" style="color:#5b7cfa;text-decoration:none;">博客</a> ·
        <a href="https://github.com/yujian2025" target="_blank" style="color:#5b7cfa;text-decoration:none;">GitHub</a> ·
        MIT
    </div>
</div>

<script>
(function() {
    var dropZone = document.getElementById('dropZone');
    var fileInput = document.getElementById('file-input');
    var browseBtn = document.getElementById('browseBtn');
    var pathInput = document.getElementById('pathInput');
    var openBtn = document.getElementById('openBtn');
    var status = document.getElementById('status');
    var langToggle = document.getElementById('langToggle');

    // ── Language ──
    var lang = localStorage.getItem('mdpreview_lang') || 'zh';

    function applyLang(l) {
        lang = l;
        localStorage.setItem('mdpreview_lang', l);
        document.querySelectorAll('[data-zh]').forEach(function(el) {
            el.textContent = el.getAttribute('data-' + l) || el.textContent;
        });
        // Update placeholder if applicable
        if (l === 'en') pathInput.placeholder = 'C:\\path\\to\\document.md';
        else pathInput.placeholder = 'C:\\path\\to\\document.md';
        document.documentElement.lang = l === 'zh' ? 'zh-CN' : 'en';
    }

    applyLang(lang);

    langToggle.onclick = function() {
        applyLang(lang === 'zh' ? 'en' : 'zh');
    };

    function setStatus(msg, type) {
        status.textContent = msg;
        status.className = type || '';
    }

    // ── Browse ──
    browseBtn.addEventListener('click', function(e) {
        e.stopPropagation();
        fileInput.click();
    });
    dropZone.addEventListener('click', function() { fileInput.click(); });

    fileInput.addEventListener('change', function() {
        if (this.files && this.files[0]) uploadFile(this.files[0]);
    });

    // ── Drag & Drop ──
    dropZone.addEventListener('dragover', function(e) {
        e.preventDefault(); e.stopPropagation();
        this.classList.add('dragover');
    });
    dropZone.addEventListener('dragleave', function(e) {
        e.preventDefault(); e.stopPropagation();
        this.classList.remove('dragover');
    });
    dropZone.addEventListener('drop', function(e) {
        e.preventDefault(); e.stopPropagation();
        this.classList.remove('dragover');
        if (e.dataTransfer.files && e.dataTransfer.files[0]) uploadFile(e.dataTransfer.files[0]);
    });

    function uploadFile(file) {
        if (!file.name.match(/\.(md|markdown|txt)$/i)) {
            setStatus(lang === 'zh' ? '请选择 .md 文件' : 'Please select a .md file', 'error');
            return;
        }
        setStatus((lang === 'zh' ? '正在读取 ' : 'Reading ') + file.name + '...', 'loading');
        var reader = new FileReader();
        reader.onload = function(e) {
            fetch('/upload', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content: e.target.result, name: file.name })
            })
            .then(function(r) { return r.text(); })
            .then(function(result) {
                if (result === 'OK') { window.location.href = '/'; }
                else { setStatus(result, 'error'); }
            })
            .catch(function() { setStatus(lang === 'zh' ? '连接失败' : 'Connection lost', 'error'); });
        };
        reader.readAsText(file);
    }

    // ── Path input ──
    function openPath() {
        var path = pathInput.value.trim();
        if (!path) {
            setStatus(lang === 'zh' ? '请输入文件路径' : 'Please enter a file path', 'error');
            return;
        }
        setStatus((lang === 'zh' ? '正在打开 ' : 'Opening ') + path + '...', 'loading');
        fetch('/open', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path: path })
        })
        .then(function(r) { return r.text(); })
        .then(function(result) {
            if (result === 'OK') { window.location.href = '/'; }
            else { setStatus(result, 'error'); }
        })
        .catch(function() { setStatus(lang === 'zh' ? '连接失败' : 'Connection lost', 'error'); });
    }

    openBtn.addEventListener('click', openPath);
    pathInput.addEventListener('keydown', function(e) {
        if (e.key === 'Enter') openPath();
    });
})();
</script>
</body>
</html>"#.to_string().replace("__VERSION__", env!("CARGO_PKG_VERSION"))
}
