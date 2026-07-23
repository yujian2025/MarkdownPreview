//! markdownpreview — A fast, minimal markdown preview reader with live reload.
//!
//! This library provides the core rendering and server logic.
//! The binary entry point is in `main.rs`.

pub mod renderer;
pub mod server;

// Re-export the main public API
pub use renderer::render_content;
pub use server::{FileEntry, FileMap, make_file_id, watch_file_id};
