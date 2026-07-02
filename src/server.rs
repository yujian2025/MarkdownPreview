use crate::renderer;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

/// Run the HTTP preview server.
///
/// When no file is loaded (`has_file` is false), the server shows a landing page
/// where the user can choose a markdown file (via drag-and-drop, browse, or path input).
/// When a file is loaded, it shows the rendered markdown preview with live reload.
pub fn run_server(
    port: u16,
    markdown: Arc<Mutex<String>>,
    version: Arc<AtomicU64>,
    current_path: Arc<Mutex<Option<PathBuf>>>,
    has_file: Arc<Mutex<bool>>,
    theme: &str,
) {
    let addr = format!("127.0.0.1:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: could not start server on {}: {}", addr, e);
            eprintln!("Try a different port with -p <port>");
            std::process::exit(1);
        }
    };

    let theme_owned = theme.to_string();
    let server_port = port;

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();
        let md = markdown.clone();
        let ver = version.clone();
        let path = current_path.clone();
        let has = has_file.clone();
        let th = theme_owned.clone();
        let p = server_port;

        // Handle each request in a thread so we don't block other requests
        std::thread::spawn(move || {
            match (method.as_str(), url.as_str()) {
                ("GET", "/") => {
                    if *has.lock().unwrap() {
                        let filename = path.lock().unwrap().clone()
                            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
                        handle_index(request, &md, &th, filename.as_deref(), p);
                    } else {
                        handle_landing(request);
                    }
                }
                ("GET", "/content") => handle_content(request, &md),
                ("GET", "/check") => handle_check(request, &ver),
                ("POST", "/close") => handle_close(request, &has, &path, &md),
                ("POST", "/open") => handle_open(request, &md, &ver, &path, &has),
                ("POST", "/upload") => handle_upload(request, &md, &ver, &path, &has),
                ("GET", "/info") => handle_info(request, &path, &has),
                _ => handle_404(request),
            }
        });
    }
}

/// Serve the full HTML preview page.
fn handle_index(
    request: tiny_http::Request,
    markdown: &Arc<Mutex<String>>,
    theme: &str,
    filename: Option<&str>,
    port: u16,
) {
    let content = markdown.lock().unwrap();
    let html = renderer::render_full_page(&content, theme, filename, port);
    respond_html(request, &html);
}

/// Serve just the rendered markdown content (for AJAX live-reload).
fn handle_content(request: tiny_http::Request, markdown: &Arc<Mutex<String>>) {
    let content = markdown.lock().unwrap();
    let html = renderer::render_content(&content);
    respond_html(request, &html);
}

/// Serve the current version number (for polling).
fn handle_check(request: tiny_http::Request, version: &Arc<AtomicU64>) {
    let v = version.load(Ordering::SeqCst);
    respond_text(request, &v.to_string());
}

/// Serve the landing page (file picker).
fn handle_landing(request: tiny_http::Request) {
    let html = renderer::render_landing_page();
    respond_html(request, &html);
}

/// Handle POST /close — reset state so the user can pick a new file.
fn handle_close(
    request: tiny_http::Request,
    has_file: &Arc<Mutex<bool>>,
    current_path: &Arc<Mutex<Option<PathBuf>>>,
    markdown: &Arc<Mutex<String>>,
) {
    *has_file.lock().unwrap() = false;
    current_path.lock().unwrap().take();
    markdown.lock().unwrap().clear();
    println!("  → Closed file, returning to file picker");
    respond_text(request, "OK");
}

/// Handle POST /open — receive JSON `{"path":"..."}`, read file, start watching.
fn handle_open(
    mut request: tiny_http::Request,
    markdown: &Arc<Mutex<String>>,
    version: &Arc<AtomicU64>,
    current_path: &Arc<Mutex<Option<PathBuf>>>,
    has_file: &Arc<Mutex<bool>>,
) {
    // Read body manually  
    use std::io::Read;
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
            // Try raw string as path (for backward compatibility)
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
            *markdown.lock().unwrap() = content;
            version.fetch_add(1, Ordering::SeqCst);
            let display_path = abs_path.clone();
            *current_path.lock().unwrap() = Some(abs_path.clone());
            *has_file.lock().unwrap() = true;

            let wm = markdown.clone();
            let wv = version.clone();
            std::thread::spawn(move || { watch_file(&abs_path, wm, wv); });

            println!("  → Opened: {}", display_path.display());
            respond_text(request, "OK");
        }
        Err(e) => {
            respond_text(request, &format!("ERROR: could not read file: {}", e));
        }
    }
}

/// Handle POST /upload — receive JSON `{"content":"...", "name":"..."}` or raw body.
fn handle_upload(
    mut request: tiny_http::Request,
    markdown: &Arc<Mutex<String>>,
    version: &Arc<AtomicU64>,
    current_path: &Arc<Mutex<Option<PathBuf>>>,
    has_file: &Arc<Mutex<bool>>,
) {
    // Read body manually to avoid read_to_string issues
    use std::io::Read;
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

    // Try JSON first, fall back to raw body
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

    *markdown.lock().unwrap() = content;
    version.fetch_add(1, Ordering::SeqCst);
    *has_file.lock().unwrap() = true;

    // Store filename for toolbar display
    if let Some(name) = filename {
        *current_path.lock().unwrap() = Some(PathBuf::from(&name));
    }

    respond_text(request, "OK");
}

/// Serve file info JSON.
fn handle_info(
    request: tiny_http::Request,
    current_path: &Arc<Mutex<Option<PathBuf>>>,
    has_file: &Arc<Mutex<bool>>,
) {
    let has = *has_file.lock().unwrap();
    let path = current_path.lock().unwrap().clone();
    let info = if has {
        if let Some(p) = path {
            format!(r#"{{"hasFile":true,"path":"{}","filename":"{}"}}"#,
                p.display().to_string().replace('\\', "\\\\"),
                p.file_name().unwrap().to_str().unwrap_or(""))
        } else {
            r#"{"hasFile":true,"path":"","filename":""}"#.to_string()
        }
    } else {
        r#"{"hasFile":false}"#.to_string()
    };
    respond_json(request, &info);
}

/// 404 handler.
fn handle_404(request: tiny_http::Request) {
    let response = Response::from_string("404 Not Found").with_status_code(404);
    let _ = request.respond(response);
}

// ── File watcher for live reload ───────────────────────────────

fn watch_file(
    file_path: &std::path::Path,
    content: Arc<Mutex<String>>,
    version: Arc<AtomicU64>,
) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let watch_dir = file_path.parent().unwrap_or(file_path).to_path_buf();
    let target_file = file_path.file_name().unwrap().to_str().unwrap().to_string();
    let dir = watch_dir.clone();
    let target = target_file.clone();

    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let is_target = event.paths.iter().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == target)
                    .unwrap_or(false)
            });
            if is_target {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let fp = dir.join(&target);
                    if let Ok(new_content) = std::fs::read_to_string(&fp) {
                        let mut md = content.lock().unwrap();
                        if *md != new_content {
                            *md = new_content;
                            version.fetch_add(1, Ordering::SeqCst);
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

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
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
