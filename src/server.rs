use crate::renderer;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

/// A single markdown file entry in the shared state.
#[derive(Clone)]
pub struct FileEntry {
    pub content: String,
    pub version: u64,
    pub path: Option<PathBuf>,
    pub display_name: String,
}

/// Shared state: map of file_id -> FileEntry
pub type FileMap = Arc<Mutex<HashMap<String, FileEntry>>>;

/// Generate a URL-safe file ID from a path.
pub fn make_file_id(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| {
            n.chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect::<String>()
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Ensure a unique file ID within the current map.
fn unique_file_id(base: &str, map: &HashMap<String, FileEntry>) -> String {
    if !map.contains_key(base) {
        return base.to_string();
    }
    for i in 1..100 {
        let candidate = format!("{}_{}", base, i);
        if !map.contains_key(&candidate) {
            return candidate;
        }
    }
    format!("{}_{}", base, rand_id())
}

fn rand_id() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Run the HTTP preview server with tabbed multi-file UI.
///
/// Routes:
///   GET   /                    → main app page (tabbed UI)
///   GET   /content?id=xxx      → rendered markdown content for a file
///   GET   /check?id=xxx        → version number for a file (polling)
///   POST  /close?id=xxx        → close a specific file
///   POST  /open                → open a new file from path
///   POST  /upload              → upload content as a new file
///   GET   /info                → list of open files (JSON)
pub fn run_server(port: u16, files: FileMap, theme: &str) {
    let addr = format!("127.0.0.1:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  Error: could not start server on {}: {}", addr, e);
            eprintln!("  Try a different port with -p <port>");
            std::process::exit(1);
        }
    };

    let theme_owned = theme.to_string();

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();
        let f = files.clone();
        let th = theme_owned.clone();

        std::thread::spawn(move || {
            let (path, query) = parse_url(&url);
            let file_id = query.get("id").cloned();

            match (method.as_str(), path.as_str()) {
                // ── Main app page (tabbed UI) ──
                ("GET", "/") => {
                    let map = f.lock().unwrap();
                    let active_id = query.get("active").map(|s| s.as_str());
                    let html = renderer::render_app_page(&map, active_id, &th, port);
                    drop(map);
                    respond_html(request, &html);
                }

                // ── API: get rendered content for a file ──
                ("GET", "/content") => {
                    match file_id {
                        Some(id) => {
                            let map = f.lock().unwrap();
                            if let Some(entry) = map.get(&id) {
                                let html = renderer::render_content(&entry.content);
                                drop(map);
                                respond_html(request, &html);
                            } else {
                                drop(map);
                                respond_text(request, "ERROR: file not found");
                            }
                        }
                        None => respond_text(request, "ERROR: missing id parameter"),
                    }
                }

                // ── API: check version for polling ──
                ("GET", "/check") => {
                    match file_id {
                        Some(id) => {
                            let map = f.lock().unwrap();
                            if let Some(entry) = map.get(&id) {
                                let v = entry.version;
                                drop(map);
                                respond_text(request, &v.to_string());
                            } else {
                                drop(map);
                                respond_text(request, "0");
                            }
                        }
                        None => respond_text(request, "0"),
                    }
                }

                // ── API: close a file ──
                ("POST", "/close") => {
                    if let Some(id) = file_id {
                        let mut map = f.lock().unwrap();
                        if map.remove(&id).is_some() {
                            println!("  → Closed file: {}", id);
                        }
                        drop(map);
                        respond_text(request, "OK");
                    } else {
                        respond_text(request, "OK");
                    }
                }

                // ── API: open a file by path ──
                ("POST", "/open") => handle_open(request, &f),

                // ── API: upload content as a new file ──
                ("POST", "/upload") => handle_upload(request, &f),

                // ── API: get file info (JSON) ──
                ("GET", "/info") => handle_info(request, &f),

                // ── API: clear all files ──
                ("POST", "/clear-all") => {
                    let mut map = f.lock().unwrap();
                    map.clear();
                    drop(map);
                    println!("  → Cleared all files, reset to initial state");
                    respond_text(request, "OK");
                }

                // ── 404 ──
                _ => handle_404(request),
            }
        });
    }
}

/// Parse URL path and query string. Query values are URL-decoded.
fn parse_url(url: &str) -> (String, HashMap<String, String>) {
    let mut parts = url.splitn(2, '?');
    let path = parts.next().unwrap_or("").to_string();
    let mut query = HashMap::new();
    if let Some(qs) = parts.next() {
        for pair in qs.split('&') {
            let mut kv = pair.splitn(2, '=');
            let key = kv.next().unwrap_or("").to_string();
            let val = url_decode(kv.next().unwrap_or(""));
            query.insert(key, val);
        }
    }
    (path, query)
}

/// Simple URL percent-decode. Decodes %XX sequences as UTF-8 bytes, + → space.
fn url_decode(input: &str) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '+' {
            bytes.push(b' ');
        } else if c == '%' {
            let hi = chars.next().and_then(|c| c.to_digit(16));
            let lo = chars.next().and_then(|c| c.to_digit(16));
            match (hi, lo) {
                (Some(h), Some(l)) => {
                    bytes.push((h as u8) << 4 | l as u8);
                }
                _ => bytes.push(b'?'),
            }
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Handle POST /open — receive JSON `{"path":"..."}`, read file, start watching.
fn handle_open(
    mut request: tiny_http::Request,
    files: &FileMap,
) {
    let mut buf = [0u8; 4096];
    let mut raw = Vec::new();
    loop {
        match request.as_reader().read(&mut buf) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
    }
    let body = String::from_utf8_lossy(&raw).to_string();

    #[derive(serde::Deserialize)]
    struct OpenReq { path: String }

    let file_path: PathBuf = match serde_json::from_str::<OpenReq>(&body) {
        Ok(r) => PathBuf::from(r.path),
        Err(_) => {
            let p = body.trim().trim_matches('"').to_string();
            if p.is_empty() {
                respond_text(request, "ERROR: missing path");
                return;
            }
            PathBuf::from(p)
        }
    };

    if !file_path.exists() {
        respond_text(request, &format!("ERROR: file not found: {}", file_path.display()));
        return;
    }

    let abs_path = std::fs::canonicalize(&file_path).unwrap_or(file_path);

    match std::fs::read_to_string(&abs_path) {
        Ok(content) => {
            let display_name = abs_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let mut map = files.lock().unwrap();

            // Check if this file path is already open — return existing ID
            let existing_id = {
                let mut found = None;
                for (existing_id, entry) in map.iter() {
                    if let Some(ref existing_path) = entry.path {
                        if existing_path == &abs_path {
                            found = Some(existing_id.clone());
                            break;
                        }
                    }
                }
                found
            };
            if let Some(eid) = existing_id {
                drop(map);
                println!("  → File already open: {} (id: {})", display_name, eid);
                respond_text(request, &eid);
                return;
            }

            let base_id = make_file_id(&abs_path);
            let file_id = unique_file_id(&base_id, &map);

            let entry = FileEntry {
                content,
                path: Some(abs_path.clone()),
                display_name: display_name.clone(),
                version: current_timestamp(),
            };

            map.insert(file_id.clone(), entry);

            // Start file watcher
            let w_path = abs_path.clone();
            let w_id = file_id.clone();
            let w_files = files.clone();
            std::thread::spawn(move || watch_file_id(&w_path, &w_id, w_files));

            drop(map);
            println!("  → Opened: {} (id: {})", display_name, file_id);
            respond_text(request, &file_id);
        }
        Err(e) => {
            respond_text(request, &format!("ERROR: could not read file: {}", e));
        }
    }
}

/// Handle POST /upload — receive JSON `{"content":"...", "name":"..."}` or raw body.
fn handle_upload(
    mut request: tiny_http::Request,
    files: &FileMap,
) {
    let mut buf = [0u8; 4096];
    let mut raw = Vec::new();
    loop {
        match request.as_reader().read(&mut buf) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
    }
    let body = String::from_utf8_lossy(&raw).to_string();

    if body.is_empty() {
        respond_text(request, "ERROR: empty body");
        return;
    }

    #[derive(serde::Deserialize)]
    struct UploadReq { content: String, name: Option<String> }

    let (content, filename) = match serde_json::from_str::<UploadReq>(&body) {
        Ok(r) => (r.content, r.name),
        Err(_) => (body.trim().to_string(), None),
    };

    if content.is_empty() {
        respond_text(request, "ERROR: empty content");
        return;
    }

    let display_name = filename.unwrap_or_else(|| "untitled.md".to_string());
    let base_id = display_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    let mut map = files.lock().unwrap();

    // Check if a file with the same display name already exists
    let replace_id = {
        let mut found: Option<(String, u64)> = None;
        for (existing_id, entry) in map.iter() {
            if entry.display_name == display_name {
                found = Some((existing_id.clone(), current_timestamp()));
                break;
            }
        }
        found
    };
    if let Some((rid, version)) = replace_id {
        let entry = map.get_mut(&rid).unwrap();
        entry.content = content;
        entry.version = version;
        drop(map);
        println!("  → Replaced: {} (id: {})", display_name, rid);
        respond_text(request, &rid);
        return;
    }

    let file_id = unique_file_id(&base_id, &map);

    let entry = FileEntry {
        content,
        path: None,
        display_name: display_name.clone(),
        version: current_timestamp(),
    };

    map.insert(file_id.clone(), entry);
    drop(map);

    println!("  → Uploaded: {} (id: {})", display_name, file_id);
    respond_text(request, &file_id);
}

/// Serve file info JSON — list of open files.
fn handle_info(request: tiny_http::Request, files: &FileMap) {
    let map = files.lock().unwrap();
    let mut entries = Vec::new();
    for (id, entry) in map.iter() {
        let path_str = entry
            .path
            .as_ref()
            .map(|p| p.display().to_string().replace('\\', "\\\\"))
            .unwrap_or_default();
        entries.push(format!(
            r#"{{"id":"{}","name":"{}","path":"{}"}}"#,
            id,
            entry.display_name.replace('\\', "\\\\").replace('"', "\\\""),
            path_str
        ));
    }
    let json = format!(r#"{{"files":[{}]}}"#, entries.join(","));
    drop(map);
    respond_json(request, &json);
}

/// 404 handler.
fn handle_404(request: tiny_http::Request) {
    let response = Response::from_string("404 Not Found").with_status_code(404);
    let _ = request.respond(response);
}

// ── File watcher for live reload ───────────────────────────────

/// Watch a file for changes and update the shared state.
pub fn watch_file_id(
    file_path: &std::path::Path,
    file_id: &str,
    files: FileMap,
) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let watch_dir = file_path.parent().unwrap_or(file_path).to_path_buf();
    let target_file = file_path.file_name().unwrap().to_str().unwrap().to_string();
    let dir = watch_dir.clone();
    let target = target_file.clone();
    let fid = file_id.to_string();
    let fid_for_closure = fid.clone();
    let files_for_closure = files.clone();
    let dir_for_closure = dir.clone();
    let target_for_closure = target.clone();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let is_target = event.paths.iter().any(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == target_for_closure)
                        .unwrap_or(false)
                });
                if is_target {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        let fp = dir_for_closure.join(&target_for_closure);
                        if let Ok(new_content) = std::fs::read_to_string(&fp) {
                            let mut map = files_for_closure.lock().unwrap();
                            if let Some(entry) = map.get_mut(&fid_for_closure) {
                                if entry.content != new_content {
                                    entry.content = new_content;
                                    entry.version = current_timestamp();
                                }
                            }
                        }
                    }
                }
            }
        })
        .expect("Failed to create file watcher");

    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .expect("Failed to start watching");

    println!("  Watching: {} (auto-reload on save)", dir.join(&target).display());

    // Keep the watcher alive and check periodically if the file was closed
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let map = files.lock().unwrap();
        if !map.contains_key(&fid) {
            println!("  → Watcher stopped: {} was closed", target);
            break;
        }
    }
}

pub fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

// ── Helpers ────────────────────────────────────────────────────

fn respond_html(request: tiny_http::Request, body: &str) {
    let response = Response::from_string(body)
        .with_header("Content-Type: text/html; charset=utf-8".parse::<Header>().unwrap());
    let _ = request.respond(response);
}

fn respond_text(request: tiny_http::Request, body: &str) {
    let response = Response::from_string(body)
        .with_header("Content-Type: text/plain; charset=utf-8".parse::<Header>().unwrap())
        .with_header("Cache-Control: no-cache".parse::<Header>().unwrap());
    let _ = request.respond(response);
}

fn respond_json(request: tiny_http::Request, body: &str) {
    let response = Response::from_string(body)
        .with_header("Content-Type: application/json; charset=utf-8".parse::<Header>().unwrap());
    let _ = request.respond(response);
}