//! markdownpreview — A fast, minimal markdown preview reader
//! Runs with system tray icon support.

use clap::Parser;
use markdownpreview::server;
use std::collections::HashMap;
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

    // Shared state: map of file_id -> FileEntry
    let files: server::FileMap = Arc::new(Mutex::new(HashMap::new()));

    // If files were provided via CLI, read them and start watching
    if !args.file.is_empty() {
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
                version: server::current_timestamp(),
            };

            // Spawn file watcher
            let w_path = file_path.clone();
            let w_id = file_id.clone();
            let w_files = files.clone();
            std::thread::spawn(move || server::watch_file_id(&w_path, &w_id, w_files));

            files.lock().unwrap().insert(file_id, entry);
        }
    }

    // Print banner
    let file_count = files.lock().unwrap().len();
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

    // ── System tray icon ──
    run_tray(port);
}

// ── System tray (Windows API) ──

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

    // Store port in a static so the window procedure can access it
    static mut TRAY_PORT: u16 = 8080;
    unsafe { TRAY_PORT = port; }

    // ── Create custom "M" icon ──
    let m_icon = create_m_icon();

    // Window procedure
    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    return 0;
                }
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
                    if lparam as u32 == 0x205 {
                        // WM_CONTEXTMENU (right click)
                        let mut pos = std::mem::zeroed();
                        GetCursorPos(&mut pos);
                        let hmenu = CreatePopupMenu();
                        AppendMenuW(
                            hmenu,
                            MF_STRING,
                            ID_TRAY_OPEN,
                            to_wstr("打开浏览器 / Open Browser").as_ptr(),
                        );
                        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                        AppendMenuW(
                            hmenu,
                            MF_STRING,
                            ID_TRAY_EXIT,
                            to_wstr("退出 / Exit").as_ptr(),
                        );
                        SetForegroundWindow(hwnd);
                        TrackPopupMenu(
                            hmenu,
                            TPM_LEFTALIGN,
                            pos.x,
                            pos.y,
                            0,
                            hwnd,
                            std::ptr::null_mut(),
                        );
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
            0,
            class_name.as_ptr(),
            to_wstr("MarkdownPreview").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinst,
            std::ptr::null_mut(),
        );

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_CALLBACK;
        nid.hIcon = m_icon;

        // Tooltip
        let tip = to_wstr("Markdown Preview v0.2.0");
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
        if !m_icon.is_null() {
            winapi::um::winuser::DestroyIcon(m_icon);
        }
    }
}

/// Create a custom "M" icon for the program using Windows GDI.
fn create_m_icon() -> winapi::shared::windef::HICON {
    unsafe {
        use winapi::ctypes::c_void;
        use winapi::shared::windef::HICON;
        use winapi::um::wingdi::*;
        use winapi::um::winuser::{CreateIconIndirect, GetDC, ICONINFO, ReleaseDC};

        const W: i32 = 32;
        const H: i32 = 32;

        let hdc_screen = GetDC(std::ptr::null_mut());
        let hdc = CreateCompatibleDC(hdc_screen);

        // ── Color bitmap (32bpp BGRA) ──
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = W;
        bmi.bmiHeader.biHeight = -H; // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut color_bits: *mut c_void = std::ptr::null_mut();
        let hbmp_color = CreateDIBSection(
            hdc,
            &mut bmi,
            DIB_RGB_COLORS,
            &mut color_bits,
            std::ptr::null_mut(),
            0,
        );

        if hbmp_color.is_null() {
            DeleteDC(hdc);
            ReleaseDC(std::ptr::null_mut(), hdc_screen);
            return std::ptr::null_mut();
        }

        // Fill pixel data: blue background + white "M" letter
        let pixels = std::slice::from_raw_parts_mut(color_bits as *mut u32, (W * H) as usize);
        let bg = 0xFF4A6AE0u32; // Blue (#4A6AE0)
        let fg = 0xFFFFFFFFu32; // White

        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let is_m = is_m_pixel_32(x, y);
                let is_border = is_rounded_corner(x, y, W, H);
                pixels[i] = if is_border {
                    0x00000000 // transparent
                } else if is_m {
                    fg
                } else {
                    bg
                };
            }
        }

        // ── Mask bitmap (1bpp, 1 = transparent, 0 = opaque) ──
        let mask_row_bytes = ((W + 31) / 32) * 4;
        let mut mask_data = vec![0u8; (mask_row_bytes * H) as usize];
        for y in 0..H {
            for x in 0..W {
                if is_rounded_corner(x, y, W, H) {
                    let byte_idx = (y * mask_row_bytes + x / 8) as usize;
                    mask_data[byte_idx] |= 1 << (7 - (x % 8));
                }
            }
        }
        let hbmp_mask = CreateBitmap(
            W,
            H,
            1,
            1,
            mask_data.as_ptr() as *const c_void,
        );

        // ── Create icon ──
        let mut icon_info: ICONINFO = std::mem::zeroed();
        icon_info.fIcon = 1;
        icon_info.hbmColor = hbmp_color;
        icon_info.hbmMask = hbmp_mask;

        let hicon: HICON = CreateIconIndirect(&mut icon_info);

        // Cleanup GDI objects
        DeleteObject(hbmp_color as *mut _);
        DeleteObject(hbmp_mask as *mut _);
        DeleteDC(hdc);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);

        hicon
    }
}

/// Check if a pixel is part of the "M" letter shape (32x32 icon).
fn is_m_pixel_32(x: i32, y: i32) -> bool {
    // Left vertical bar (x: 5-8, y: 6-27)
    if x >= 5 && x <= 8 && y >= 6 && y <= 27 {
        return true;
    }
    // Right vertical bar (x: 23-26, y: 6-27)
    if x >= 23 && x <= 26 && y >= 6 && y <= 27 {
        return true;
    }
    // Left diagonal: from (8, 6) to (15, 27)
    if x >= 8 && x <= 15 && y >= 6 && y <= 27 {
        let expected_y = 3 * (x - 8) + 6; // slope = 21/7 = 3
        if (y - expected_y).abs() <= 2 {
            return true;
        }
    }
    // Right diagonal: from (15, 27) to (23, 6)
    if x >= 15 && x <= 23 && y >= 6 && y <= 27 {
        let expected_y = ((-21.0 / 8.0) * (x - 15) as f64 + 27.0) as i32;
        if (y - expected_y).abs() <= 2 {
            return true;
        }
    }
    // Top horizontal bar connecting the two verticals (x: 8-23, y: 6-8)
    if x >= 9 && x <= 22 && y >= 6 && y <= 8 {
        return true;
    }
    false
}

/// Check if a pixel is outside the rounded corners (for transparency).
fn is_rounded_corner(x: i32, y: i32, w: i32, h: i32) -> bool {
    let r = 4; // corner radius
    // Top-left
    if x < r && y < r {
        let dx = x - r + 1;
        let dy = y - r + 1;
        return dx * dx + dy * dy > r * r;
    }
    // Top-right
    if x >= w - r && y < r {
        let dx = x - (w - r) - 1;
        let dy = y - r + 1;
        return dx * dx + dy * dy > r * r;
    }
    // Bottom-left
    if x < r && y >= h - r {
        let dx = x - r + 1;
        let dy = y - (h - r) - 1;
        return dx * dx + dy * dy > r * r;
    }
    // Bottom-right
    if x >= w - r && y >= h - r {
        let dx = x - (w - r) - 1;
        let dy = y - (h - r) - 1;
        return dx * dx + dy * dy > r * r;
    }
    false
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
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "MarkdownPreview",
            "/t",
            "REG_SZ",
            "/d",
            &exe,
            "/f",
        ])
        .status()
    {
        Ok(s) if s.success() => {
            println!(
                "✅ Auto-start installed: {}\n   → {}",
                "MarkdownPreview will start on boot", exe
            );
        }
        _ => {
            eprintln!("❌ Failed to install auto-start (try running as admin)");
        }
    }
}

fn uninstall_startup() {
    match std::process::Command::new("reg")
        .args(&[
            "delete",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "MarkdownPreview",
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

fn print_banner(file_count: usize, port: u16, theme: &str) {
    let mode = if file_count > 0 {
        format!("{} file(s) loaded + auto-reload", file_count)
    } else {
        "file picker (open in browser)".to_string()
    };
    let v = env!("CARGO_PKG_VERSION");

    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║         _ __ ___ ___ _ __ ___            ║");
    println!("  ║        | '_ \\ __/ __| '__/ _ \\           ║");
    println!("  ║        | |_) |_|\\__ \\ | | (_) |          ║");
    println!("  ║        | .__/   |___/_|  \\___/           ║");
    println!("  ║        |_|    Markdown Preview           ║");
    println!("  ╠══════════════════════════════════════════╣");
    println!("  ║  Version : {}                                 ║", v);
    println!("  ║  Mode    : {}      ║", mode);
    println!("  ║  Port    : {}                                 ║", port);
    println!("  ║  Theme   : {}                                 ║", theme);
    println!("  ║  Tray    : right-click icon → Exit            ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
}