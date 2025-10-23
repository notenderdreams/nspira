use std::fs;

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
    if fs::metadata(path).is_ok() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
