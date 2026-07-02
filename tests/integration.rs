//! Integration tests for markdownpreview
//!
//! These test the core components directly (no background process needed).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Import our crate
extern crate markdownpreview;

#[test]
fn test_render_content_basic() {
    let md = "# Hello\n\nThis is **bold** and *italic*.";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<h1>"), "Should have h1 tag");
    assert!(html.contains("Hello"), "Should contain heading text");
    assert!(html.contains("<strong>"), "Should have bold tag");
    assert!(html.contains("<em>"), "Should have italic tag");
}

#[test]
fn test_render_content_code_block() {
    let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
    let html = markdownpreview::render_content(md);
    // pulldown-cmark renders fenced code inside <pre><code>
    assert!(html.contains("<pre>"), "Should have pre tag");
    assert!(html.contains("fn main"), "Should contain code content");
    assert!(html.contains("println!"), "Should contain code content");
}

#[test]
fn test_render_content_table() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<table>"), "Should have table tag");
    assert!(html.contains("<th>"), "Should have header cell");
    assert!(html.contains("<td>"), "Should have data cell");
}

#[test]
fn test_render_content_task_list() {
    let md = "- [x] Done\n- [ ] Todo";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("checkbox"), "Should have checkbox input");
    assert!(html.contains("checked"), "Done item should be checked");
}

#[test]
fn test_render_content_blockquote() {
    let md = "> This is a quote";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<blockquote>"), "Should have blockquote tag");
    assert!(html.contains("This is a quote"), "Should contain quote text");
}

#[test]
fn test_render_content_strikethrough() {
    let md = "~~deleted~~";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<del>"), "Should have del tag");
    assert!(html.contains("deleted"), "Should contain deleted text");
}

#[test]
fn test_render_content_unordered_list() {
    let md = "- Item 1\n- Item 2";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<ul>"), "Should have ul tag");
    assert!(html.contains("<li>"), "Should have li tag");
}

#[test]
fn test_render_content_ordered_list() {
    let md = "1. First\n2. Second";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<ol>"), "Should have ol tag");
}

#[test]
fn test_render_content_link() {
    let md = "[GitHub](https://github.com)";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<a href"), "Should have anchor tag");
    assert!(html.contains("GitHub"), "Should contain link text");
}

#[test]
fn test_render_content_horizontal_rule() {
    let md = "---";
    let html = markdownpreview::render_content(md);
    assert!(html.contains("<hr"), "Should have hr tag");
}

#[test]
fn test_render_full_page_light() {
    let md = "# Test";
    let html = markdownpreview::render_full_page(md, "light", None, 8080);
    assert!(html.contains("<!DOCTYPE html>"), "Should be a full HTML page");
    assert!(html.contains("#ffffff"), "Light background color");
    assert!(html.contains("#1f2328"), "Light text color");
    assert!(html.contains("fetch("), "Should have live-reload JS");
    assert!(html.contains("</html>"), "Should have closing HTML tag");
    assert!(html.contains("btnBack"), "Should have back button");
    assert!(html.contains("toolbar"), "Should have toolbar");
}

#[test]
fn test_render_full_page_dark() {
    let md = "# Test";
    let html = markdownpreview::render_full_page(md, "dark", None, 8080);
    assert!(html.contains("#0d1117"), "Dark theme should have dark background");
    assert!(html.contains("#e6edf3"), "Dark theme should have light text");
}

#[test]
fn test_render_full_page_content_injection() {
    let md = "# Hello World\n\nSome paragraph.";
    let html = markdownpreview::render_full_page(md, "light", None, 8080);
    assert!(html.contains("Hello World"), "Page should contain rendered content");
    assert!(html.contains("Some paragraph"), "Page should contain rendered content");
}

#[test]
fn test_render_full_page_empty_content() {
    let md = "";
    let html = markdownpreview::render_full_page(md, "light", None, 8080);
    assert!(html.contains("<!DOCTYPE html>"), "Should still render full page");
    assert!(html.contains("id=\"content\""), "Should have content div");
}

#[test]
fn test_render_full_page_with_filename() {
    let md = "# Test";
    let html = markdownpreview::render_full_page(md, "light", Some("my_doc.md"), 8080);
    assert!(html.contains("my_doc.md"), "Filename should appear in toolbar");
}

#[test]
fn test_render_full_page_toolbar_buttons() {
    let md = "# Test";
    let html = markdownpreview::render_full_page(md, "dark", Some("readme.md"), 8080);
    assert!(html.contains("/close"), "Should have close endpoint reference");
    assert!(html.contains("打开文件"), "Should have back button with Chinese text");
    assert!(html.contains("btnPrint"), "Should have print button");
    assert!(html.contains("btnClose"), "Should have close button");
    assert!(html.contains("@media print"), "Should have print CSS");
}

#[test]
fn test_version_increment() {
    let version = Arc::new(AtomicU64::new(100));
    assert_eq!(version.load(Ordering::SeqCst), 100);

    version.fetch_add(1, Ordering::SeqCst);
    assert_eq!(version.load(Ordering::SeqCst), 101);

    version.fetch_add(1, Ordering::SeqCst);
    assert_eq!(version.load(Ordering::SeqCst), 102);
}

// ── Landing page tests ────────────────────────────────────────

#[test]
fn test_landing_page_structure() {
    let html = markdownpreview::render_landing_page();
    assert!(html.contains("<!DOCTYPE html>"), "Should be a full HTML page");
    assert!(html.contains("Markdown Preview"), "Should have title");
    assert!(html.contains("drop-zone"), "Should have drop zone");
    assert!(html.contains("pathInput"), "Should have path input");
    assert!(html.contains("Browse Files"), "Should have browse button");
    assert!(html.contains("Open"), "Should have open button");
    assert!(html.contains("</html>"), "Should have closing tag");
}

#[test]
fn test_landing_page_drag_and_drop_js() {
    let html = markdownpreview::render_landing_page();
    assert!(html.contains("dragover"), "Should handle dragover event");
    assert!(html.contains("FileReader"), "Should use FileReader API");
    assert!(html.contains("/upload"), "Should POST to /upload");
    assert!(html.contains("/open"), "Should POST to /open");
}

#[test]
fn test_landing_page_no_markdown_content() {
    // The landing page should NOT contain markdown-rendered content
    let html = markdownpreview::render_landing_page();
    assert!(!html.contains("#content"), "Landing page should not have content div");
    assert!(!html.contains("checkVersion"), "Landing page should not have version checker");
}

#[test]
fn test_landing_page_recent_or_tips() {
    let html = markdownpreview::render_landing_page();
    assert!(html.contains("Tips") || html.contains("reload"), "Should show tips or instructions");
}
