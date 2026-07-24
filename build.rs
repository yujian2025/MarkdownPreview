fn main() {
    // Only embed icon on Windows
    #[cfg(target_os = "windows")]
    {
        // Generate icon.ico and resource.rc
        generate_icon();
        embed_resource::compile("resource.rc", embed_resource::NONE);
    }
}

#[cfg(target_os = "windows")]
fn generate_icon() {
    use std::fs::File;
    use std::io::Write;

    const W: u32 = 32;
    const H: u32 = 32;

    // Build ICO file from pixel data
    let mut ico_data: Vec<u8> = Vec::new();

    // ICONDIR (6 bytes)
    ico_data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico_data.extend_from_slice(&1u16.to_le_bytes()); // type = icon
    ico_data.extend_from_slice(&1u16.to_le_bytes()); // count

    // ICONDIRENTRY (16 bytes)
    ico_data.push(W as u8);
    ico_data.push(H as u8);
    ico_data.push(0);
    ico_data.push(0);
    ico_data.extend_from_slice(&1u16.to_le_bytes());  // planes
    ico_data.extend_from_slice(&32u16.to_le_bytes()); // bpp

    let bih_size: u32 = 40;
    let and_row_bytes = ((W + 31) / 32) * 4;
    let image_size = bih_size + (W * H * 4) + (and_row_bytes * H);
    ico_data.extend_from_slice(&image_size.to_le_bytes());
    ico_data.extend_from_slice(&22u32.to_le_bytes()); // offset = 6 + 16

    // BITMAPINFOHEADER
    ico_data.extend_from_slice(&bih_size.to_le_bytes());
    ico_data.extend_from_slice(&W.to_le_bytes());
    ico_data.extend_from_slice(&(H * 2).to_le_bytes()); // double height for ICO
    ico_data.extend_from_slice(&1u16.to_le_bytes());
    ico_data.extend_from_slice(&32u16.to_le_bytes());
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // compression
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // image size
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // x ppm
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // y ppm
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // colors used
    ico_data.extend_from_slice(&0u32.to_le_bytes()); // important colors

    // Pixel data (BGRA, bottom-up)
    let bg: u32 = 0xFF4A6AE0; // Blue
    let fg: u32 = 0xFFFFFFFF; // White
    for y in 0..H {
        for x in 0..W {
            let row = H - 1 - y; // bottom-up
            let px = row * W + x;
            let color = if is_m(px, W, H) { fg } else { bg };
            ico_data.push((color & 0xFF) as u8);        // B
            ico_data.push(((color >> 8) & 0xFF) as u8); // G
            ico_data.push(((color >> 16) & 0xFF) as u8);// R
            ico_data.push(((color >> 24) & 0xFF) as u8);// A
        }
    }

    // AND mask (1 = transparent, all opaque for icon)
    for _ in 0..(and_row_bytes * H) {
        ico_data.push(0);
    }

    // Write to file
    let mut f = File::create("icon.ico").expect("Failed to create icon.ico");
    f.write_all(&ico_data).expect("Failed to write icon.ico");

    // Create resource.rc
    let rc_content = "IDI_APP ICON \"icon.ico\"\n";
    let mut rc = File::create("resource.rc").expect("Failed to create resource.rc");
    rc.write_all(rc_content.as_bytes()).expect("Failed to write resource.rc");
}

#[cfg(target_os = "windows")]
fn is_m(px: u32, w: u32, _h: u32) -> bool {
    let x = (px % w) as i32;
    let y = (px / w) as i32;
    // Left vertical bar
    if x >= 5 && x <= 8 && y >= 6 && y <= 27 { return true; }
    // Right vertical bar
    if x >= 23 && x <= 26 && y >= 6 && y <= 27 { return true; }
    // Left diagonal
    if x >= 8 && x <= 15 && y >= 6 && y <= 27 {
        if (y - (3 * (x - 8) + 6)).abs() <= 2 { return true; }
    }
    // Right diagonal
    if x >= 15 && x <= 23 && y >= 6 && y <= 27 {
        let expected = ((-21.0 / 8.0) * (x - 15) as f64 + 27.0) as i32;
        if (y - expected).abs() <= 2 { return true; }
    }
    // Top horizontal bar
    if x >= 9 && x <= 22 && y >= 6 && y <= 8 { return true; }
    false
}
