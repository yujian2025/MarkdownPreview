use pulldown_cmark::{Options, Parser};
use std::collections::HashMap;

use crate::server::FileEntry;

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

/// Render the main application page with tabbed interface.
///
/// This is the single-page app that shows all open files as tabs
/// (like WPS Office), with a content area showing the active tab's
/// rendered markdown and live-reload polling.
pub fn render_app_page(
    files: &HashMap<String, FileEntry>,
    active_id: Option<&str>,
    theme: &str,
    port: u16,
) -> String {
    let is_dark = theme == "dark";

    // Build tab bar HTML
    let mut tabs_html = String::new();
    for (id, entry) in files.iter() {
        let active = active_id.map_or(false, |a| a == id.as_str());
        let name = &entry.display_name;
        tabs_html.push_str(&format!(
            r#"<div class="tab {}" data-id="{}"><span class="tab-icon">📄</span><span class="tab-name">{}</span><span class="tab-close" data-id="{}">✕</span></div>"#,
            if active { "active" } else { "" },
            id,
            name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;"),
            id,
        ));
    }

    // Build files JSON for JavaScript initialization
    let files_json: String = {
        let mut arr = Vec::new();
        for (id, entry) in files.iter() {
            let name = entry.display_name.replace('\\', "\\\\").replace('"', "\\\"");
            arr.push(format!(r#"{{"id":"{}","name":"{}"}}"#, id, name));
        }
        format!("[{}]", arr.join(","))
    };

    // Determine active tab and its content
    let resolved_active = active_id
        .and_then(|id| {
            if files.contains_key(id) {
                Some(id.to_string())
            } else {
                files.keys().next().cloned()
            }
        })
        .or_else(|| files.keys().next().cloned());

    let active_content = resolved_active
        .as_ref()
        .and_then(|id| files.get(id))
        .map(|entry| render_content(&entry.content))
        .unwrap_or_default();

    let active_id_str = resolved_active.as_deref().unwrap_or("");

    static TEMPLATE: &str = include_str!("../templates/app.html");

    // Theme CSS variables (including tab-specific colors)
    let (bg, fg, muted, border, accent, code_bg, pre_bg, table_hdr, table_alt);
    let (tab_bg, tab_active_bg, tab_hover_bg, toolbar_bg);

    if is_dark {
        bg = "#0d1117"; fg = "#e6edf3"; muted = "#8b949e"; border = "#30363d";
        accent = "#58a6ff"; code_bg = "rgba(110,118,129,0.4)"; pre_bg = "#161b22";
        table_hdr = "rgba(110,118,129,0.15)"; table_alt = "rgba(110,118,129,0.06)";
        tab_bg = "#161b22"; tab_active_bg = "#0d1117";
        tab_hover_bg = "rgba(110,118,129,0.15)"; toolbar_bg = "#161b22";
    } else {
        bg = "#ffffff"; fg = "#1f2328"; muted = "#656d76"; border = "#d0d7de";
        accent = "#0969da"; code_bg = "rgba(175,184,193,0.2)"; pre_bg = "#f6f8fa";
        table_hdr = "rgba(208,215,222,0.3)"; table_alt = "#f6f8fa";
        tab_bg = "#f6f8fa"; tab_active_bg = "#ffffff";
        tab_hover_bg = "rgba(208,215,222,0.3)"; toolbar_bg = "#f6f8fa";
    }

    TEMPLATE
        .replace("__TABS__", &tabs_html)
        .replace("__CONTENT__", &active_content)
        .replace("__FILES_JSON__", &files_json)
        .replace("__ACTIVE_ID__", active_id_str)
        .replace("__THEME__", theme)
        .replace("__VERSION__", env!("CARGO_PKG_VERSION"))
        .replace("__PORT__", &port.to_string())
        .replace("__BG__", bg)
        .replace("__FG__", fg)
        .replace("__MUTED__", muted)
        .replace("__BORDER__", border)
        .replace("__ACCENT__", accent)
        .replace("__CODE_BG__", code_bg)
        .replace("__PRE_BG__", pre_bg)
        .replace("__TABLE_HDR__", table_hdr)
        .replace("__TABLE_ALT__", table_alt)
        .replace("__TAB_BG__", tab_bg)
        .replace("__TAB_ACTIVE_BG__", tab_active_bg)
        .replace("__TAB_HOVER_BG__", tab_hover_bg)
        .replace("__TOOLBAR_BG__", toolbar_bg)
}