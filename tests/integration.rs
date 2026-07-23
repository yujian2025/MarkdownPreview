//! Integration tests for markdownpreview
//!
//! These test the core components directly (no background process needed).

use std::collections::HashMap;
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

// ── App page tests ────────────────────────────────────────────

#[test]
fn test_app_page_empty() {
    let map = HashMap::new();
    let html = markdownpreview::renderer::render_app_page(&map, None, "light", 8080);
    assert!(html.contains("<!DOCTYPE html>"), "Should be a full HTML page");
    assert!(html.contains("Markdown Preview"), "Should have title");
    assert!(html.contains("id=\"welcome\""), "Should have welcome section");
    assert!(html.contains("</html>"), "Should have closing tag");
}

#[test]
fn test_app_page_with_files() {
    let mut map = HashMap::new();
    map.insert(
        "test_md".to_string(),
        markdownpreview::FileEntry {
            content: "# Hello\n\nWorld".to_string(),
            version: 1,
            path: Some(std::path::PathBuf::from("test.md")),
            display_name: "test.md".to_string(),
        },
    );
    let html = markdownpreview::renderer::render_app_page(&map, Some("test_md"), "light", 8080);
    assert!(html.contains("<!DOCTYPE html>"), "Should be a full HTML page");
    assert!(html.contains("test.md"), "Should show filename in tab bar");
    assert!(html.contains("data-id=\"test_md\""), "Tab should have data-id attribute");
    assert!(html.contains("active"), "Active tab should have active class");
    assert!(html.contains("fetch("), "Should have live-reload JS");
}

#[test]
fn test_app_page_dark_theme() {
    let mut map = HashMap::new();
    map.insert(
        "test_md".to_string(),
        markdownpreview::FileEntry {
            content: "# Test".to_string(),
            version: 1,
            path: None,
            display_name: "test.md".to_string(),
        },
    );
    let html = markdownpreview::renderer::render_app_page(&map, Some("test_md"), "dark", 8080);
    assert!(html.contains("#0d1117"), "Dark theme should have dark background");
    assert!(html.contains("#e6edf3"), "Dark theme should have light text");
}

#[test]
fn test_app_page_file_picker_modal() {
    let map = HashMap::new();
    let html = markdownpreview::renderer::render_app_page(&map, None, "light", 8080);
    assert!(html.contains("id=\"filePicker\""), "Should have file picker modal");
    assert!(html.contains("modalBrowseBtn"), "Should have browse button in modal");
    assert!(html.contains("modalFileInput"), "Should have file input in modal");
}

#[test]
fn test_app_page_tab_bar() {
    let mut map = HashMap::new();
    map.insert(
        "a_md".to_string(),
        markdownpreview::FileEntry {
            content: "# A".to_string(),
            version: 1,
            path: None,
            display_name: "a.md".to_string(),
        },
    );
    map.insert(
        "b_md".to_string(),
        markdownpreview::FileEntry {
            content: "# B".to_string(),
            version: 2,
            path: None,
            display_name: "b.md".to_string(),
        },
    );
    let html = markdownpreview::renderer::render_app_page(&map, Some("a_md"), "light", 8080);
    assert!(html.contains("a.md"), "Should show first filename");
    assert!(html.contains("b.md"), "Should show second filename");
    assert!(html.contains("tab active"), "Should have active tab");
    assert!(html.contains("tab-add"), "Should have add tab button");
}

#[test]
fn test_app_page_print_button() {
    let map = HashMap::new();
    let html = markdownpreview::renderer::render_app_page(&map, None, "light", 8080);
    assert!(html.contains("btnPrint"), "Should have print button");
    assert!(html.contains("@media print"), "Should have print styles");
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

#[test]
fn test_make_file_id() {
    let path = std::path::PathBuf::from("hello world.md");
    let id = markdownpreview::make_file_id(&path);
    assert_eq!(id, "hello_world_md", "Should create URL-safe file ID");

    let path2 = std::path::PathBuf::from("simple.md");
    let id2 = markdownpreview::make_file_id(&path2);
    assert_eq!(id2, "simple_md", "Should handle simple filenames");

    // Chinese characters are is_alphanumeric() in Rust
    let path3 = std::path::PathBuf::from("产品说明书.md");
    let id3 = markdownpreview::make_file_id(&path3);
    assert_eq!(id3, "产品说明书_md", "Should preserve Chinese characters");
}

#[test]
fn test_app_page_js_files_json() {
    let mut map = HashMap::new();
    map.insert(
        "file1_md".to_string(),
        markdownpreview::FileEntry {
            content: "test".to_string(),
            version: 1,
            path: None,
            display_name: "file1.md".to_string(),
        },
    );
    let html = markdownpreview::renderer::render_app_page(&map, Some("file1_md"), "light", 8080);
    // The files JSON should be embedded in the JS for tab initialization
    assert!(html.contains("file1_md"), "Should contain file ID in page");
    assert!(html.contains("file1.md"), "Should contain filename in page");
}