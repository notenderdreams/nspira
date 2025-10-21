// Table: projects
// ---------------
// id            INTEGER PRIMARY KEY AUTOINCREMENT
// name          TEXT            -- project/folder name
// path          TEXT            -- absolute path to project root
// cache_dir     TEXT            -- relative cache folder (e.g. "node_modules" or "target")
// last_cleaned  TEXT            -- ISO timestamp (UTC), set on creation or after cleaning


// Location:
// ---------
// ~/.local/share/nspira/nspira.db   (Linux/macOS)
// %AppData%\nspira\nspira.db        (Windows)
// */