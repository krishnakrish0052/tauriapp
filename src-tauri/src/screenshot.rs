use screenshots::Screen;
use base64::Engine;
use anyhow::Result;
use log::{info, error};
use image::ImageFormat;
use std::io::Cursor;

/// Capture screenshot of the primary display and return as base64 encoded image
pub fn capture_screenshot() -> Result<String> {
    info!("Capturing screenshot...");
    
    // Try primary method first (screenshots crate)
    match capture_screenshot_primary() {
        Ok(result) => {
            info!("Screenshot captured successfully using primary method");
            return Ok(result);
        }
        Err(e) => {
            error!("Primary screenshot method failed: {}", e);
            info!("Trying Windows fallback method...");
        }
    }
    
    // Try Windows-specific fallback
    #[cfg(target_os = "windows")]
    {
        match capture_screenshot_windows() {
            Ok(result) => {
                info!("Screenshot captured successfully using Windows fallback");
                return Ok(result);
            }
            Err(e) => {
                error!("Windows fallback screenshot method failed: {}", e);
            }
        }
    }
    
    Err(anyhow::anyhow!("All screenshot capture methods failed"))
}

fn capture_screenshot_primary() -> Result<String> {
    // Get all screens
    let screens = Screen::all()
        .ok_or_else(|| {
            error!("Failed to get screens - no screens available");
            anyhow::anyhow!("Failed to get screens - no screens available")
        })?;
    
    // Use the primary screen (first one)
    let screen = screens.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("No screens found"))?;
    
    info!("Screen found: {}x{} (scale: {})", 
          screen.display_info.width, 
          screen.display_info.height,
          screen.display_info.scale_factor);
    
    // Capture the screenshot
    let image = screen.capture()
        .ok_or_else(|| {
            error!("Failed to capture screenshot - screen capture returned None");
            anyhow::anyhow!("Failed to capture screenshot - screen capture returned None")
        })?;
    
    info!("Screenshot captured successfully: {}x{}", image.width(), image.height());
    
    // Convert the screenshots::Image to image::RgbaImage then encode as PNG
    // The screenshots crate may return data in BGRA format on Windows, so we need to convert
    let buffer = image.buffer();
    let width = image.width();
    let height = image.height();
    
    info!("Screenshot buffer info: {}x{}, buffer size: {} bytes", width, height, buffer.len());
    
    // Validate buffer is not empty
    if buffer.is_empty() {
        error!("Screenshot buffer is empty");
        return Err(anyhow::anyhow!("Screenshot buffer is empty"));
    }
    
    // Calculate bytes per pixel based on actual buffer size
    let total_pixels = (width * height) as usize;
    
    // Prevent division by zero
    if total_pixels == 0 {
        error!("Invalid image dimensions: {}x{}", width, height);
        return Err(anyhow::anyhow!("Invalid image dimensions: {}x{}", width, height));
    }
    
    let bytes_per_pixel = buffer.len() / total_pixels;
    
    info!("Calculated bytes per pixel: {} (total pixels: {}, buffer size: {})", bytes_per_pixel, total_pixels, buffer.len());
    
    // Handle edge case where calculation results in 0 bytes per pixel
    if bytes_per_pixel == 0 {
        error!("Buffer too small for image dimensions: buffer={} bytes for {}x{} = {} pixels", 
               buffer.len(), width, height, total_pixels);
        return Err(anyhow::anyhow!("Buffer too small for image dimensions: {} bytes for {} pixels", 
                                  buffer.len(), total_pixels));
    }
    
    // Handle different pixel formats
    if bytes_per_pixel != 3 && bytes_per_pixel != 4 {
        error!("Unsupported pixel format: {} bytes per pixel (buffer: {} bytes, dimensions: {}x{})", 
               bytes_per_pixel, buffer.len(), width, height);
        return Err(anyhow::anyhow!("Unsupported pixel format: {} bytes per pixel", bytes_per_pixel));
    }
    
    // Convert to RGBA format based on the detected pixel format
    let rgba_buffer: Vec<u8> = match bytes_per_pixel {
        4 => {
            // 4 bytes per pixel - likely BGRA on Windows, RGBA on other platforms
            if cfg!(target_os = "windows") {
                // Convert BGRA to RGBA
                buffer.chunks(4)
                    .flat_map(|pixel| {
                        if pixel.len() == 4 {
                            vec![pixel[2], pixel[1], pixel[0], pixel[3]] // BGRA -> RGBA
                        } else {
                            vec![0, 0, 0, 255] // Fallback black pixel with full alpha
                        }
                    })
                    .collect()
            } else {
                buffer.to_vec() // On non-Windows, assume it's already RGBA
            }
        }
        3 => {
            // 3 bytes per pixel - RGB format, need to add alpha channel
            if cfg!(target_os = "windows") {
                // Convert BGR to RGBA
                buffer.chunks(3)
                    .flat_map(|pixel| {
                        if pixel.len() == 3 {
                            vec![pixel[2], pixel[1], pixel[0], 255] // BGR -> RGBA with full alpha
                        } else {
                            vec![0, 0, 0, 255] // Fallback black pixel with full alpha
                        }
                    })
                    .collect()
            } else {
                // Convert RGB to RGBA
                buffer.chunks(3)
                    .flat_map(|pixel| {
                        if pixel.len() == 3 {
                            vec![pixel[0], pixel[1], pixel[2], 255] // RGB -> RGBA with full alpha
                        } else {
                            vec![0, 0, 0, 255] // Fallback black pixel with full alpha
                        }
                    })
                    .collect()
            }
        }
        _ => {
            error!("Unexpected bytes per pixel: {}", bytes_per_pixel);
            return Err(anyhow::anyhow!("Unexpected bytes per pixel: {}", bytes_per_pixel));
        }
    };
    
    let rgba_image = image::RgbaImage::from_raw(
        width,
        height,
        rgba_buffer
    ).ok_or_else(|| {
        error!("Failed to create RGBA image from screenshot buffer after format conversion");
        anyhow::anyhow!("Failed to create RGBA image from screenshot buffer after format conversion")
    })?;
    
    // Convert to PNG format in memory
    let mut png_buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_buffer);
        rgba_image.write_to(&mut cursor, ImageFormat::Png).map_err(|e| {
            error!("Failed to encode screenshot as PNG: {}", e);
            anyhow::anyhow!("Failed to encode screenshot as PNG: {}", e)
        })?
    }
    
    // Encode as base64
    let base64_image = base64::prelude::BASE64_STANDARD.encode(&png_buffer);
    
    info!("Screenshot encoded as base64, size: {} bytes", base64_image.len());
    
    Ok(base64_image)
}

#[cfg(target_os = "windows")]
fn capture_screenshot_windows() -> Result<String> {
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    
    info!("Attempting Windows GDI screenshot capture...");
    
    unsafe {
        // Get desktop window and device context
        let hwnd_desktop = GetDesktopWindow();
        let hdc_desktop = GetDC(hwnd_desktop);
        if hdc_desktop == 0 {
            return Err(anyhow::anyhow!("Failed to get desktop device context"));
        }
        
        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        
        info!("Windows desktop dimensions: {}x{}", screen_width, screen_height);
        
        if screen_width <= 0 || screen_height <= 0 {
            ReleaseDC(hwnd_desktop, hdc_desktop);
            return Err(anyhow::anyhow!("Invalid screen dimensions: {}x{}", screen_width, screen_height));
        }
        
        // Create compatible device context and bitmap
        let hdc_mem = CreateCompatibleDC(hdc_desktop);
        if hdc_mem == 0 {
            ReleaseDC(hwnd_desktop, hdc_desktop);
            return Err(anyhow::anyhow!("Failed to create compatible device context"));
        }
        
        let hbitmap = CreateCompatibleBitmap(hdc_desktop, screen_width, screen_height);
        if hbitmap == 0 {
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd_desktop, hdc_desktop);
            return Err(anyhow::anyhow!("Failed to create compatible bitmap"));
        }
        
        // Select bitmap into memory device context
        let old_bitmap = SelectObject(hdc_mem, hbitmap);
        if old_bitmap == 0 {
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd_desktop, hdc_desktop);
            return Err(anyhow::anyhow!("Failed to select bitmap into memory DC"));
        }
        
        // Copy screen to bitmap
        let result = BitBlt(
            hdc_mem,
            0,
            0,
            screen_width,
            screen_height,
            hdc_desktop,
            0,
            0,
            SRCCOPY,
        );
        
        if result == 0 {
            SelectObject(hdc_mem, old_bitmap);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            ReleaseDC(hwnd_desktop, hdc_desktop);
            return Err(anyhow::anyhow!("Failed to copy screen to bitmap"));
        }
        
        // Get bitmap data
        let mut bmp_info = std::mem::zeroed::<BITMAPINFO>();
        bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmp_info.bmiHeader.biWidth = screen_width;
        bmp_info.bmiHeader.biHeight = -screen_height; // Top-down bitmap
        bmp_info.bmiHeader.biPlanes = 1;
        bmp_info.bmiHeader.biBitCount = 32; // RGBA
        bmp_info.bmiHeader.biCompression = BI_RGB;
        
        let buffer_size = (screen_width * screen_height * 4) as usize;
        let mut buffer = vec![0u8; buffer_size];
        
        let lines_copied = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            screen_height as u32,
            buffer.as_mut_ptr() as *mut _,
            &mut bmp_info,
            DIB_RGB_COLORS,
        );
        
        // Clean up GDI objects
        SelectObject(hdc_mem, old_bitmap);
        DeleteObject(hbitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd_desktop, hdc_desktop);
        
        if lines_copied == 0 {
            return Err(anyhow::anyhow!("Failed to get bitmap data"));
        }
        
        info!("Windows screenshot captured: {} lines, buffer size: {} bytes", lines_copied, buffer.len());
        
        // Convert BGRA to RGBA
        for chunk in buffer.chunks_mut(4) {
            if chunk.len() == 4 {
                chunk.swap(0, 2); // Swap B and R channels
            }
        }
        
        // Create image from buffer
        let rgba_image = image::RgbaImage::from_raw(
            screen_width as u32,
            screen_height as u32,
            buffer
        ).ok_or_else(|| anyhow::anyhow!("Failed to create RGBA image from Windows screenshot buffer"))?;
        
        // Convert to PNG format in memory
        let mut png_buffer = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_buffer);
            rgba_image.write_to(&mut cursor, ImageFormat::Png).map_err(|e| {
                error!("Failed to encode Windows screenshot as PNG: {}", e);
                anyhow::anyhow!("Failed to encode Windows screenshot as PNG: {}", e)
            })?
        }
        
        // Encode as base64
        let base64_image = base64::prelude::BASE64_STANDARD.encode(&png_buffer);
        
        info!("Windows screenshot encoded as base64, size: {} bytes", base64_image.len());
        
        Ok(base64_image)
    }
}

/// Capture screenshot and return as data URL for direct use in HTML/web contexts
pub fn capture_screenshot_as_data_url() -> Result<String> {
    let base64_data = capture_screenshot()?;
    Ok(format!("data:image/png;base64,{}", base64_data))
}
