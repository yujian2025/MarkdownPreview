//! markdownpreview — A fast, minimal markdown preview reader
//! Cross-platform (Windows/Linux/macOS).
//! On Windows with `tray` feature (default), shows a system tray icon.
//! On other platforms, runs as a terminal server until Ctrl+C.

use clap::Parser;
use markdownpreview::server;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

/// A fast, minimal markdown preview reader with live reload
#[derive(Parser, Debug)]
#[command(name = "markdownpreview")]
#[command(version, about, long_about = None)]
struct Args {
    /// Markdown file(s) to preview (optional — use file picker page if omitted)
    file: Vec<PathBuf>,

    /// Port for the preview server (default: 8090)
    #[arg(short = 'p', long, default_value_t = 8090)]
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

    // Handle --install / --uninstall (Windows only)
    #[cfg(feature = "tray")]
    {
        if args.install {
            install_startup();
            return;
        }
        if args.uninstall {
            uninstall_startup();
            return;
        }
    }
    #[cfg(not(feature = "tray"))]
    {
        if args.install || args.uninstall {
            eprintln!("  --install / --uninstall is only supported on Windows with the `tray` feature.");
            std::process::exit(1);
        }
    }

    let theme = validate_theme(&args.theme);
    let port = args.port;
    let _ = port; // suppress unused warning if tray disabled

    // Shared state: map of file_id -> FileEntry
    let files: server::FileMap = Arc::new(Mutex::new(std::collections::HashMap::new()));

    // If files were provided via CLI, read them and start watching
    let file_count = if !args.file.is_empty() {
        let mut count = 0;
        for file in &args.file {
            if !file.exists() {
                eprintln!("  Error: file '{}' not found", file.display());
                continue;
            }
            let file_path = std::fs::canonicalize(file).unwrap_or_else(|e| {
                eprintln!("  Error resolving path: {}", e);
                std::process::exit(1);
            });
            let content = std::fs::read_to_string(&file_path).unwrap_or_else(|e| {
                eprintln!("  Error reading file: {}", e);
                std::process::exit(1);
            });

            let file_id = server::make_file_id(&file_path);
            let display_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let entry = server::FileEntry {
                content,
                path: Some(file_path.clone()),
                display_name: display_name.clone(),
                version: current_timestamp(),
            };

            // Spawn file watcher
            let w_path = file_path.clone();
            let w_id = file_id.clone();
            let w_files = files.clone();
            std::thread::spawn(move || server::watch_file_id(&w_path, &w_id, w_files));

            files.lock().unwrap().insert(file_id, entry);
            count += 1;
        }
        count
    } else {
        0
    };

    // Print banner
    print_banner(file_count, port, &theme);

    // Start HTTP server
    let sf = files.clone();
    let st = theme.clone();
    std::thread::spawn(move || server::run_server(port, sf, &st));

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

    // ── Platform-specific main loop ──
    #[cfg(feature = "tray")]
    run_tray(port);

    #[cfg(not(feature = "tray"))]
    wait_for_exit();
}

/// Wait for Ctrl+C or Enter to exit (non-Windows / no-tray mode).
#[cfg(not(feature = "tray"))]
fn wait_for_exit() {
    println!("  Press Ctrl+C to stop the server.");
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl+C handler");
    let _ = rx.recv();
    println!("\n  Server stopped.");
}

// ── Tray icon (Windows only, behind `tray` feature) ──

#[cfg(feature = "tray")]
fn run_tray(port: u16) {
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winuser::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos,
        GetMessageW, LoadCursorW, PostQuitMessage, RegisterClassW,
        SetForegroundWindow, TrackPopupMenu, TranslateMessage,
        CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW,
        MF_SEPARATOR, MF_STRING, MSG, TPM_LEFTALIGN, WNDCLASSW,
        WS_OVERLAPPEDWINDOW, WM_COMMAND, WM_DESTROY, WM_USER,
    };
    use winapi::um::shellapi::{
        Shell_NotifyIconW, NOTIFYICONDATAW, NIM_ADD, NIM_DELETE,
        NIF_ICON, NIF_MESSAGE, NIF_TIP,
    };

    const WM_TRAY_CALLBACK: u32 = WM_USER + 100;
    const ID_TRAY_OPEN: usize = 1001;
    const ID_TRAY_EXIT: usize = 1002;

    static mut TRAY_PORT: u16 = 8080;
    unsafe { TRAY_PORT = port; }

    let m_icon = create_m_icon();

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
                                let _ = open::that(&format!("http://127.0.0.1:{}", p));
                            });
                            return 0;
                        }
                        ID_TRAY_EXIT => { DestroyWindow(hwnd); return 0; }
                        _ => {}
                    }
                }
                WM_TRAY_CALLBACK => {
                    if lparam as u32 == 0x205 {
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
            hIcon: m_icon,
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
        nid.hIcon = m_icon;

        let tip = to_wstr("Markdown Preview");
        let tip_len = tip.len().min(128);
        for (i, &ch) in tip[..tip_len].iter().enumerate() {
            nid.szTip[i] = ch;
        }

        Shell_NotifyIconW(NIM_ADD, &mut nid);
        println!("  [tray] Icon created. Right-click to open/exit.");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &mut nid);
        if !m_icon.is_null() {
            winapi::um::winuser::DestroyIcon(m_icon);
        }
    }
}

#[cfg(feature = "tray")]
fn to_wstr(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Create a custom "M" icon using ICO format bytes.
#[cfg(feature = "tray")]
fn create_m_icon() -> winapi::shared::windef::HICON {
    const W: u32 = 32;
    const H: u32 = 32;

    // Generate pixel data (BGRA format, bottom-up for ICO)
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    let bg: u32 = 0xFF4A6AE0; // Blue
    let fg: u32 = 0xFFFFFFFF; // White
    // ICO stores pixels bottom-up (last row first)
    for y in 0..H {
        for x in 0..W {
            let row = H - 1 - y; // bottom-up
            let i = (row * W + x) as usize;
            let color = if is_rounded_corner(x as i32, y as i32, W as i32, H as i32) {
                0x00000000u32
            } else if is_m_pixel_32(x as i32, y as i32) {
                fg
            } else {
                bg
            };
            // BGRA: byte order is B, G, R, A
            pixels[i * 4 + 0] = (color & 0xFF) as u8;
            pixels[i * 4 + 1] = ((color >> 8) & 0xFF) as u8;
            pixels[i * 4 + 2] = ((color >> 16) & 0xFF) as u8;
            pixels[i * 4 + 3] = ((color >> 24) & 0xFF) as u8;
        }
    }

    // AND mask (1 = transparent, bottom-up)
    let and_row_bytes = ((W + 31) / 32) * 4;
    let mut and_mask = vec![0u8; (and_row_bytes * H) as usize];
    for y in 0..H {
        for x in 0..W {
            if is_rounded_corner(x as i32, y as i32, W as i32, H as i32) {
                let row = H - 1 - y;
                let byte_idx = (row * and_row_bytes + x / 8) as usize;
                and_mask[byte_idx] |= 1 << (7 - (x % 8));
            }
        }
    }

    // Build ICO file data
    // ── BITMAPINFOHEADER (40 bytes) ──
    let bih_size: u32 = 40;
    let mut ico_data: Vec<u8> = Vec::new();

    // ICONDIR (6 bytes)
    ico_data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico_data.extend_from_slice(&1u16.to_le_bytes()); // type = icon
    ico_data.extend_from_slice(&1u16.to_le_bytes()); // count

    // ICONDIRENTRY (16 bytes)
    ico_data.push(W as u8);           // width
    ico_data.push(H as u8);           // height
    ico_data.push(0);                 // colors
    ico_data.push(0);                 // reserved
    ico_data.extend_from_slice(&1u16.to_le_bytes());  // planes
    ico_data.extend_from_slice(&32u16.to_le_bytes()); // bit count

    let image_size = bih_size + (W * H * 4) + (and_row_bytes * H);
    ico_data.extend_from_slice(&(image_size as u32).to_le_bytes());
    let image_offset: u32 = 6 + 16; // header + directory entry
    ico_data.extend_from_slice(&image_offset.to_le_bytes());

    // ── Image data: BITMAPINFOHEADER ──
    ico_data.extend_from_slice(&bih_size.to_le_bytes());
    ico_data.extend_from_slice(&W.to_le_bytes());
    ico_data.extend_from_slice(&(H * 2).to_le_bytes()); // ICO: height must be doubled
    ico_data.extend_from_slice(&1u16.to_le_bytes());    // planes
    ico_data.extend_from_slice(&32u16.to_le_bytes());   // bpp
    ico_data.extend_from_slice(&0u32.to_le_bytes());    // compression
    ico_data.extend_from_slice(&image_size.to_le_bytes()); // image size
    ico_data.extend_from_slice(&0u32.to_le_bytes());    // x pixels per meter
    ico_data.extend_from_slice(&0u32.to_le_bytes());    // y pixels per meter
    ico_data.extend_from_slice(&0u32.to_le_bytes());    // colors used
    ico_data.extend_from_slice(&0u32.to_le_bytes());    // important colors

    // XOR mask (pixel data)
    ico_data.extend_from_slice(&pixels);
    // AND mask
    ico_data.extend_from_slice(&and_mask);

    unsafe {
        use winapi::um::winuser::CreateIconFromResource;
        CreateIconFromResource(
            ico_data.as_ptr() as *mut u8,
            ico_data.len() as u32,
            1,  // fIcon = TRUE
            0x00030000, // dwReserved = Windows 3.0 format
        )
    }
}

#[cfg(feature = "tray")]
fn is_m_pixel_32(x: i32, y: i32) -> bool {
    if x >= 5 && x <= 8 && y >= 6 && y <= 27 { return true; }
    if x >= 23 && x <= 26 && y >= 6 && y <= 27 { return true; }
    if x >= 8 && x <= 15 && y >= 6 && y <= 27 {
        if (y - (3 * (x - 8) + 6)).abs() <= 2 { return true; }
    }
    if x >= 15 && x <= 23 && y >= 6 && y <= 27 {
        let expected = ((-21.0 / 8.0) * (x - 15) as f64 + 27.0) as i32;
        if (y - expected).abs() <= 2 { return true; }
    }
    if x >= 9 && x <= 22 && y >= 6 && y <= 8 { return true; }
    false
}

#[cfg(feature = "tray")]
fn is_rounded_corner(x: i32, y: i32, w: i32, h: i32) -> bool {
    let r = 4;
    if x < r && y < r { return (x - r + 1).pow(2) + (y - r + 1).pow(2) > r * r; }
    if x >= w - r && y < r { return (x - (w - r) - 1).pow(2) + (y - r + 1).pow(2) > r * r; }
    if x < r && y >= h - r { return (x - r + 1).pow(2) + (y - (h - r) - 1).pow(2) > r * r; }
    if x >= w - r && y >= h - r { return (x - (w - r) - 1).pow(2) + (y - (h - r) - 1).pow(2) > r * r; }
    false
}

// ── Auto-start (Windows registry, Windows only) ──

#[cfg(feature = "tray")]
fn exe_path() -> String {
    std::env::current_exe()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

#[cfg(feature = "tray")]
fn install_startup() {
    let exe = exe_path();
    match std::process::Command::new("reg")
        .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "MarkdownPreview", "/t", "REG_SZ", "/d", &exe, "/f"])
        .status()
    {
        Ok(s) if s.success() => {
            println!("✅ Auto-start installed: MarkdownPreview will start on boot\n   → {}", exe);
        }
        _ => eprintln!("❌ Failed to install auto-start (try running as admin)"),
    }
}

#[cfg(feature = "tray")]
fn uninstall_startup() {
    match std::process::Command::new("reg")
        .args(&["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "MarkdownPreview", "/f"])
        .status()
    {
        Ok(s) if s.success() => println!("✅ Auto-start removed"),
        _ => eprintln!("❌ Failed to remove auto-start (may not be installed)"),
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

fn print_banner(file_count: usize, port: u16, theme: &str) {
    let mode = if file_count > 0 {
        format!("{} file(s) loaded + auto-reload", file_count)
    } else {
        "file picker (open in browser)".to_string()
    };
    let v = env!("CARGO_PKG_VERSION");

    println!();
    println!("  ╔══════════════════════════════════════════════════╗");
    println!("  ║     __  ___           _                        ║");
    println!("  ║    /  |/  /___ _   __(_)___  ___  _____        ║");
    println!("  ║   / /|_/ / __ \\ | / / / __|/ _ \\/ ___/        ║");
    println!("  ║  / /  / / /_/ / |/ / / /_/ /  __/ /            ║");
    println!("  ║ /_/  /_/\\____/|___/_/\\__/ \\___/_/             ║");
    println!("  ║           Markdown Preview  v{}              ║", v);
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║                                                  ║");
    println!("  ║  ⚡ Lightweight  ~880 KB binary                  ║");
    println!("  ║  🚀 Fast         <100 ms startup                 ║");
    println!("  ║  💾 Low memory   ~5 MB RAM                       ║");
    println!("  ║  🔧 Standalone   No Node.js, No Python, No IDE  ║");
    println!("  ║                                                  ║");
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║  Mode    : {:<42}║", mode);
    println!("  ║  Port    : {:<42}║", port);
    println!("  ║  Theme   : {:<42}║", theme);
    #[cfg(feature = "tray")]
    println!("  ║  Tray    : right-click icon → Exit              ║");
    #[cfg(not(feature = "tray"))]
    println!("  ║  Tray    : not available (headless mode)         ║");
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║                                                  ║");
    println!("  ║  📌  Use Cases                                  ║");
    println!("  ║  • Quick preview .md files while coding          ║");
    println!("  ║  • Read docs/README side-by-side with editor     ║");
    println!("  ║  • Lightweight alternative to Typora/Obsidian    ║");
    println!("  ║  • Print / export .md to PDF via browser         ║");
    println!("  ║  • Team-share preview via URL (no install)       ║");
    println!("  ║                                                  ║");
    println!("  ╚══════════════════════════════════════════════════╝");
    println!();
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}