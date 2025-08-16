use std::path::{ PathBuf};

pub fn get_system_font_dirs() -> Vec<PathBuf> {
    let mut font_dirs = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        use winapi::um::sysinfoapi::GetWindowsDirectoryW;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        
        let mut buf = [0u16; 260];
        unsafe {
            GetWindowsDirectoryW(buf.as_mut_ptr(), buf.len() as u32);
            // FreeConsole();
        }
        let path = PathBuf::from(
            String::from_utf16_lossy(&buf)
                .trim_end_matches('\0')
        );
        font_dirs.push(path.join("Fonts"));
    }
    
    #[cfg(target_os = "macos")]
    {
        font_dirs.push(PathBuf::from("/Library/Fonts"));
        font_dirs.push(PathBuf::from("/System/Library/Fonts"));
        if let Ok(home) = std::env::var("HOME") {
            font_dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        font_dirs.push(PathBuf::from("/usr/share/fonts"));
        font_dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Ok(home) = std::env::var("HOME") {
            font_dirs.push(PathBuf::from(home).join(".fonts"));
        }
    }
    
    font_dirs
}

/// 递归扫描目录中的字体文件
pub  fn scan_font_files(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut font_files = Vec::new();
    let extensions = ["ttf", "otf", "ttc", "woff", "woff2"];
    
    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    font_files.extend(scan_font_files(vec![path]));
                } else if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        font_files.push(path);
                    }
                }
            }
        }
    }
    
    font_files
}


