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

pub fn get_dir_size(path: &str) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    size += metadata.len();
                }
            } else if path.is_dir() {
                size += get_dir_size(path.to_str().unwrap());
            }
        }
    }
    size
}





pub fn clean_dir(path: &str) -> anyhow::Result<()> {
    let path = Path::new(path);

    if !path.exists() || !path.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            fs::remove_file(&entry_path)?;
        } else if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        }
    }
    Ok(())
}
