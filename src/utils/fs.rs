use std::fs;
use std::path::Path;

pub fn human_readable_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Absolute path with `.` and `..` resolved lexically. Symlinks are kept as-is
/// so the result matches paths produced by walking the filesystem.
pub fn absolute(path: impl AsRef<Path>) -> std::path::PathBuf {
    use std::path::Component;
    let joined = std::env::current_dir().unwrap_or_default().join(path);
    let mut out = std::path::PathBuf::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// Total size of regular files under `path`. Symlinks are not followed.
pub fn get_dir_size(path: impl AsRef<Path>) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Delete everything inside `path`, keeping the directory itself.
/// Symlinks are removed, never followed. Missing directories are a no-op.
pub fn clean_dir(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if !path.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        } else {
            fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

/// Pure function to compute scan paths for Windows given a home directory and available drives.
/// Ensures the user's home directory is checked, as well as the roots of other drives (e.g. D:\, E:\).
pub fn compute_windows_scan_paths(
    home: Option<&Path>,
    drives: &[&Path],
) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let mut paths = Vec::new();
    let mut home_drive_letter: Option<char> = None;

    if let Some(h) = home {
        let s = h.to_string_lossy();
        let bytes = s.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            home_drive_letter = Some((bytes[0] as char).to_ascii_uppercase());
        }
        paths.push(h.to_path_buf());

        // Also check if common project folders exist on the home drive root (e.g. C:\projects, C:\dev)
        if let Some(letter) = home_drive_letter {
            for folder in &[
                "projects",
                "dev",
                "code",
                "workspace",
                "src",
                "repos",
                "git",
            ] {
                let dev_dir = PathBuf::from(format!(r"{}:\{}", letter, folder));
                if dev_dir.is_dir() && !paths.contains(&dev_dir) {
                    paths.push(dev_dir);
                }
            }
        }
    }

    for &drive in drives {
        let s = drive.to_string_lossy();
        let bytes = s.as_bytes();
        let drive_letter = if bytes.len() >= 2 && bytes[1] == b':' {
            Some((bytes[0] as char).to_ascii_uppercase())
        } else {
            None
        };

        // If this drive is the one containing the home directory, we've already handled it
        if let (Some(hl), Some(dl)) = (home_drive_letter, drive_letter)
            && hl == dl
        {
            continue;
        }

        let drive_buf = drive.to_path_buf();
        if !paths.contains(&drive_buf) {
            paths.push(drive_buf);
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    paths
}

#[cfg(windows)]
pub fn get_available_windows_drives() -> Vec<std::path::PathBuf> {
    use std::collections::HashSet;
    use std::path::PathBuf;

    let mut drives = Vec::new();
    let mut seen = HashSet::new();

    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        let mount = disk.mount_point();
        let s = mount.to_string_lossy().to_uppercase();
        let clean = s.trim_end_matches('\\').to_string() + "\\";
        if mount.exists() && seen.insert(clean) {
            drives.push(mount.to_path_buf());
        }
    }

    // Fallback: check drive letters C..=Z in case any drives are not reported by sysinfo
    for b in b'C'..=b'Z' {
        let drive_str = format!(r"{}:\", b as char);
        if !seen.contains(&drive_str) {
            let p = PathBuf::from(&drive_str);
            if p.exists() && seen.insert(drive_str) {
                drives.push(p);
            }
        }
    }

    drives
}

/// Returns default paths to scan based on current OS.
/// On Windows: checks the home directory on C: (plus common root dev folders) and all other available drives (D:\, E:\, etc.).
/// On macOS/Linux: checks the home directory.
pub fn get_default_scan_paths() -> Vec<std::path::PathBuf> {
    #[cfg(windows)]
    {
        let home = dirs::home_dir();
        let available_drives = get_available_windows_drives();
        let drive_refs: Vec<&Path> = available_drives.iter().map(|p| p.as_path()).collect();
        compute_windows_scan_paths(home.as_deref(), &drive_refs)
    }

    #[cfg(not(windows))]
    {
        vec![dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compute_windows_scan_paths_multiple_drives() {
        let home = Path::new(r"C:\Users\testuser");
        let drives = [Path::new(r"C:\"), Path::new(r"D:\"), Path::new(r"E:\")];
        let result = compute_windows_scan_paths(Some(home), &drives);

        assert_eq!(result[0], PathBuf::from(r"C:\Users\testuser"));
        assert!(result.contains(&PathBuf::from(r"D:\")));
        assert!(result.contains(&PathBuf::from(r"E:\")));
        // Should not have C:\ since home is on C:
        assert!(!result.contains(&PathBuf::from(r"C:\")));
    }

    #[test]
    fn test_compute_windows_scan_paths_home_on_d() {
        let home = Path::new(r"D:\Users\testuser");
        let drives = [Path::new(r"C:\"), Path::new(r"D:\")];
        let result = compute_windows_scan_paths(Some(home), &drives);

        assert_eq!(result[0], PathBuf::from(r"D:\Users\testuser"));
        assert!(result.contains(&PathBuf::from(r"C:\")));
        assert!(!result.contains(&PathBuf::from(r"D:\")));
    }

    #[test]
    fn test_compute_windows_scan_paths_no_home() {
        let drives = [Path::new(r"C:\"), Path::new(r"D:\")];
        let result = compute_windows_scan_paths(None, &drives);

        assert_eq!(result, vec![PathBuf::from(r"C:\"), PathBuf::from(r"D:\")]);
    }

    #[test]
    fn test_get_default_scan_paths_not_empty() {
        let paths = get_default_scan_paths();
        assert!(!paths.is_empty());
    }
}
