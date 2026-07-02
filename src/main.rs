//! markdownpreview — A fast, minimal markdown preview reader
//! Runs with system tray icon support.

use clap::Parser;
use markdownpreview::server;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// A fast, minimal markdown preview reader with live reload
#[derive(Parser, Debug)]
#[command(name = "markdownpreview")]
#[command(version, about, long_about = None)]
struct Args {
    /// Markdown file to preview (optional — use file picker page if omitted)
    file: Option<PathBuf>,

    /// Port for the preview server (default: 8080)
    #[arg(short = 'p', long, default_value_t = 8080)]
    port: u16,

    /// Do not open browser automatically
    #[arg(short = 'n', long)]
    no_open: bool,

    /// Theme: light or dark (default: light)
    #[arg(short = 't', long, default_value = "light")]
    theme: String,

    /// Install auto-start on boot (Windows only)
    #[arg(long)]
    install: bool,

    /// Uninstall auto-start on boot
    #[arg(long)]
    uninstall: bool,
}

fn main() {
    let args = Args::parse();

    // Handle --install / --uninstall
    if args.install {
        install_startup();
        return;
    }
    if args.uninstall {
        uninstall_startup();
        return;
    }

    let theme = validate_theme(&args.theme);
    let port = args.port;

    // Shared state
    let markdown: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let version: Arc<AtomicU64> = Arc::new(AtomicU64::new(current_timestamp_nanos()));
    let current_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    let has_file: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // If a file was provided via CLI, read it and start watching
    if let Some(file) = &args.file {
        if !file.exists() {
            eprintln!("Error: file '{}' not found", file.display());
            std::process::exit(1);
        }
        let file_path = std::fs::canonicalize(file).unwrap_or_else(|e| {
            eprintln!("Error resolving path: {}", e);
            std::process::exit(1);
        });
        let initial_content = std::fs::read_to_string(&file_path).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        });
        *markdown.lock().unwrap() = initial_content;
        *current_path.lock().unwrap() = Some(file_path.clone());
        *has_file.lock().unwrap() = true;

        let wm = markdown.clone();
        let wv = version.clone();
        let fp = file_path.clone();
        std::thread::spawn(move || start_watcher(&fp, wm, wv));

        print_banner(Some(&file_path), port, &theme);
    } else {
        print_banner(None, port, &theme);
    }

    // Start HTTP server
    let sm = markdown.clone();
    let sv = version.clone();
    let sp = current_path.clone();
    let sh = has_file.clone();
    let st = theme.clone();
    std::thread::spawn(move || server::run_server(port, sm, sv, sp, sh, &st));

    // Give server a moment to start, then open browser
    if !args.no_open {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let url = format!("http://127.0.0.1:{}", port);
        println!("  Opening browser: {}", url);
        if let Err(e) = open::that(&url) {
            eprintln!("  Warning: could not open browser: {}", e);
            println!("  Manually open: {}", url);
        }
    } else {
        println!("\n  Open: http://127.0.0.1:{}", port);
    }

    // ── System tray icon ──
    run_tray(port);
}

// ── System tray (Windows API) ──

fn run_tray(port: u16) {
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winuser::{
        CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, 
        DispatchMessageW, GetMessageW, PostQuitMessage, RegisterClassW,
        SetForegroundWindow, TrackPopupMenu, TranslateMessage, 
        AppendMenuW, LoadCursorW, LoadIconW, MSG, WNDCLASSW,
        CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDI_APPLICATION, 
        IDC_ARROW, WS_OVERLAPPEDWINDOW, WM_DESTROY, WM_COMMAND,
        WM_USER, TPM_LEFTALIGN, MF_STRING, MF_SEPARATOR, DestroyWindow,
        GetCursorPos,
    };
    use winapi::um::shellapi::{Shell_NotifyIconW, NOTIFYICONDATAW,
        NIM_ADD, NIM_DELETE, NIF_MESSAGE, NIF_ICON, NIF_TIP};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const WM_TRAY_CALLBACK: u32 = WM_USER + 100;
    const ID_TRAY_OPEN: usize = 1001;
    const ID_TRAY_EXIT: usize = 1002;

    // Store port in a static so the window procedure can access it
    static mut TRAY_PORT: u16 = 8080;
    unsafe { TRAY_PORT = port; }

    // Window procedure
    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
        unsafe {
            match msg {
                WM_DESTROY => { PostQuitMessage(0); return 0; }
                WM_COMMAND => {
                    let id = wparam & 0xFFFF;
                    match id as usize {
                        ID_TRAY_OPEN => {
                            let p = TRAY_PORT;
                            std::thread::spawn(move || {
                                let url = format!("http://127.0.0.1:{}", p);
                                let _ = open::that(&url);
                            });
                            return 0;
                        }
                        ID_TRAY_EXIT => {
                            DestroyWindow(hwnd);
                            return 0;
                        }
                        _ => {}
                    }
                }
                WM_TRAY_CALLBACK => {
                    if lparam as u32 == 0x205 { // WM_CONTEXTMENU (right click)
                        let mut pos = std::mem::zeroed();
                        GetCursorPos(&mut pos);
                        let hmenu = CreatePopupMenu();
                        AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN,
                            to_wstr("打开浏览器 / Open Browser").as_ptr());
                        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                        AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT,
                            to_wstr("退出 / Exit").as_ptr());
                        SetForegroundWindow(hwnd);
                        TrackPopupMenu(hmenu, TPM_LEFTALIGN, pos.x, pos.y, 0, hwnd, std::ptr::null_mut());
                        DestroyMenu(hmenu);
                        return 0;
                    }
                }
                _ => {}
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        let class_name = to_wstr("MarkdownPreviewTrayClass");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0, class_name.as_ptr(), to_wstr("MarkdownPreview").as_ptr(),
            WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
            0, 0, std::ptr::null_mut(), std::ptr::null_mut(), hinst, std::ptr::null_mut(),
        );

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_CALLBACK;
        nid.hIcon = LoadIconW(std::ptr::null_mut(), IDI_APPLICATION);

        // Tooltip
        let tip = to_wstr("Markdown Preview");
        let tip_len = tip.len().min(128);
        let tip_slice = &tip[..tip_len];
        for (i, &ch) in tip_slice.iter().enumerate() {
            nid.szTip[i] = ch;
        }

        Shell_NotifyIconW(NIM_ADD, &mut nid);
        println!("  [tray] Icon created. Right-click to open/exit.");
        println!("  Open: http://127.0.0.1:{}", port);

        // Windows message loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        Shell_NotifyIconW(NIM_DELETE, &mut nid);
    }
}

fn to_wstr(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

// ── Auto-start (Windows registry) ──

fn exe_path() -> String {
    std::env::current_exe()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn install_startup() {
    let exe = exe_path();
    match std::process::Command::new("reg")
        .args(&[
            "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "MarkdownPreview",
            "/t", "REG_SZ",
            "/d", &exe,
            "/f",
        ])
        .status()
    {
        Ok(s) if s.success() => {
            println!("✅ Auto-start installed: {}\n   → {}", 
                "MarkdownPreview will start on boot", exe);
        }
        _ => {
            eprintln!("❌ Failed to install auto-start (try running as admin)");
        }
    }
}

fn uninstall_startup() {
    match std::process::Command::new("reg")
        .args(&[
            "delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "MarkdownPreview",
            "/f",
        ])
        .status()
    {
        Ok(s) if s.success() => {
            println!("✅ Auto-start removed");
        }
        _ => {
            eprintln!("❌ Failed to remove auto-start (may not be installed)");
        }
    }
}

// ── Helpers ──

fn validate_theme(theme: &str) -> String {
    let t = theme.to_lowercase();
    if t != "light" && t != "dark" {
        eprintln!("Error: theme must be 'light' or 'dark', got '{}'", theme);
        std::process::exit(1);
    }
    t
}

fn print_banner(file: Option<&std::path::Path>, port: u16, theme: &str) {
    println!("┌─────────────────────────────────────────┐");
    println!("│  📝 Markdown Preview v{}","    │");
    println!("│─────────────────────────────────────────│");
    if let Some(f) = file {
        println!("│  File  : {}", f.display());
        println!("│  Mode  : file preview + auto-reload   │");
    } else {
        println!("│  Mode  : file picker (open in browser)│");
    }
    println!("│  Port  : {}", port);
    println!("│  Theme : {}", theme);
    println!("│  Tray  : right-click icon → Exit        │");
    println!("└─────────────────────────────────────────┘");
}

fn start_watcher(
    file_path: &std::path::Path,
    content: Arc<Mutex<String>>,
    version: Arc<AtomicU64>,
) {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let watch_dir = file_path.parent().unwrap_or(file_path).to_path_buf();
    let target_file = file_path.file_name().unwrap().to_str().unwrap().to_string();
    let wd = watch_dir.clone();
    let tf = target_file.clone();

    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let is_target = event.paths.iter().any(|p| {
                p.file_name().and_then(|n| n.to_str()).map(|n| n == tf).unwrap_or(false)
            });
            if is_target {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let fp = wd.join(&tf);
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
    }).expect("Failed to create file watcher");

    watcher.watch(&watch_dir, RecursiveMode::NonRecursive).expect("Failed to start watching");
    println!("  Watching: {} (auto-reload on save)", watch_dir.join(&target_file).display());

    loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}

fn current_timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
