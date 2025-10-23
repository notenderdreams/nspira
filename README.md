<p align="center">
<img src="./docs/logo.svg" align="center">
</p>
<h1 align="center"> 🌿 nspira </h1>
<p align="center">
<b>nspira</b> is a lightweight CLI tool that helps you manage and clean cache directories for both development projects and desktop applications.
</p>


## ✨ Features

* Track cache directories for development projects and desktop apps
* One-command safe cleaning
* Automatic cache detection (currently supports JS & Rust projects)

---

## 📋 Todo

* [ ] `doctor` – Check for missing or invalid entries
* [ ] `search` – Quickly find projects
* [ ] Lightweight GUI or interactive TUI
* [ ] Progress bar during cleaning
* [ ] Pattern based file deletion

---

## ⚙️ Installation

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (Edition 2024 or newer)

### Build from Source

```bash
git clone https://github.com/notenderdreams/nspira.git
cargo install --path nspira
```

You can delete the folder afterward — done!

---

## 🧰 Usage

```bash
nspira <COMMAND>
```

### Commands Overview

| Command                   | Description                                                                             |
| ------------------------- | --------------------------------------------------------------------------------------- |
| `init [path]`             | Initialize a new project (auto-detects cache folders like `target/` or `node_modules/`) |
| `add <name> <cache_path>` | Manually add a project and its cache directory                                          |
| `list`                    | Display all tracked projects in a clean table                                           |
| `clean [id]`              | Clean the cache for a specific project, or all projects if no ID is provided            |
| `remove <id>`             | Remove a project from tracking                                                          |
| `stats`                   | Show overall cache usage statistics                                                     |
| `doctor`                  | Scan for missing or broken projects (WIP)                                               |

---

## 🌾 Examples

Auto-detect and add your current Rust or Node.js project:

```bash
cd ~/Projects/myapp
nspira init
```

If your project contains a **package.json** or **Cargo.toml**, nspira automatically adds its cache directory.
If not, it prompts you to add one manually , so you can use it for other apps as well.

Manually add a cache directory:

```bash
nspira add "VoidCrate" "/home/user/voidcrate/target"
```

List all tracked Caches:

```bash
nspira list
```

```
╭────┬──────────────────┬────────────┬────────┬──────────────╮
│ id │ name             │ cache_path │  size  │ last_cleaned │
├────┼──────────────────┼────────────┼────────┼──────────────┤
│ 3  │ Telegram Desktop │ cache      │ 6.7 GB │ 2025-10-22   │
╰────┴──────────────────┴────────────┴────────┴──────────────╯
```

Clean all caches:

```bash
nspira clean
```

View storage statistics:

```bash
nspira stats
```
```
╭─────────────────────────────────────╮
│  Cache Statistics                   │
├─────────────────────────────────────┤
│  Projects tracked  │              3 │
│  Total cache size  │      986.36 MB │
╰─────────────────────────────────────╯
```
---

## 📂 Database Location

By default, **nspira** stores its database in:

* **Development:** `./nspira.db`
* **Production:**

    * **Linux/macOS:** `~/.local/share/nspira/nspira.db`
    * **Windows:** `%APPDATA%\nspira\nspira.db`

---

## 🧪 Development

Clone and run locally:

```bash
git clone https://github.com/notenderdreams/nspira.git
cd nspira
cargo run -- <command>
```

Run tests:

```bash
cargo test
```

