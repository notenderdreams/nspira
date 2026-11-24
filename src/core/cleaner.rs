use crate::utils::{clean_dir, get_dir_size};
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// Empty one cache dir. Returns bytes freed; missing dirs free nothing.
pub fn clean_cache(path: &str) -> Result<u64> {
    if !Path::new(path).is_dir() {
        return Ok(0);
    }
    let size = get_dir_size(path);
    clean_dir(path)?;
    Ok(size)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CleanTotals {
    /// Dirs that actually freed space.
    pub dirs: usize,
    pub bytes: u64,
    pub failed: usize,
}

impl CleanTotals {
    pub fn add(&mut self, other: CleanTotals) {
        self.dirs += other.dirs;
        self.bytes += other.bytes;
        self.failed += other.failed;
    }
}

/// Clean `paths` of one project, call `on_each` per dir, and record history.
pub fn clean_project(
    conn: &Connection,
    project_id: i32,
    paths: &[String],
    mut on_each: impl FnMut(&str, &Result<u64>),
) -> Result<CleanTotals> {
    let mut totals = CleanTotals::default();
    for path in paths {
        let result = clean_cache(path);
        on_each(path, &result);
        match result {
            Ok(0) => {}
            Ok(bytes) => {
                totals.dirs += 1;
                totals.bytes += bytes;
            }
            Err(_) => totals.failed += 1,
        }
    }
    if totals.dirs > 0 {
        crate::db::record_clean(conn, project_id, totals.bytes, totals.dirs)?;
    }
    Ok(totals)
}
