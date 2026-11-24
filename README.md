<p align="center">
<img src="./docs/nspira.png" align="center">
</p>
<p align="center">
<b>nspira</b> is a lightweight CLI tool that helps you manage and clean cache directories for both development projects and desktop applications.
</p>

---

## Features

- **Smart Project Detection** - Automatically detects projects and their cache directories
- **Visual Interface** - Beautiful TUI for managing projects and viewing statistics
- **One-Click Cleaning** - Clean individual projects or multiple projects at once
- **Health Monitoring** - Doctor command to check project health and fix issues
- **Discovery Mode** - Scan your system to find untracked projects with cache directories

---

## Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/notenderdreams/nspira
cargo install --path nspira
```

### Basic Usage

```bash
# Initialize a project in current directory (auto-detects cache dirs)
nspira init

# Add a project manually
nspira add my-project target node_modules dist

# List all projects (interactive TUI)
nspira list

# Clean all caches
nspira clean

# Clean specific project
nspira clean 1

# Check project health
nspira doctor

# Scan for new projects
nspira scan
```

---

## Commands

### `nspira init`

Initialize a new project in the current directory. Automatically detects common cache directories based on project type.

**Supported project types:**

- **Node.js** - `node_modules`, `.next`, `dist`, `build`
- **Rust** - `target`
- **Java/Maven** - `target`
- **Gradle** - `build`, `.gradle`
- **Go** - `bin`, `pkg`
- **Python** - `__pycache__`, `.venv`, `venv`

### `nspira add <name> <cache_dirs...>`

Manually add a project with specified cache directories.

```bash
nspira add my-react-app node_modules .next dist
nspira add rust-project target
nspira add java-app target build
```

### `nspira list`

Launch interactive TUI to view and manage all tracked projects.

**TUI Features:**

- Expand a project to see each cache directory and its size
- Select whole projects or individual cache directories
- Clean the selection with a progress bar
- Stop tracking a project or a single cache directory
- Sort and filter projects

### `nspira clean [id]`

Clean cache directories. Without ID, cleans all projects.

```bash
nspira clean      # Clean all projects
nspira clean 1    # Clean project with ID 1
```

### `nspira doctor`

Check health of all tracked projects and identify issues.

**Checks:**

- Project paths exist
- Cache directories exist
- Provides fixes for missing paths
- Remove broken projects directly from TUI

### `nspira scan`

Scan your filesystem for projects with cache directories that aren't being tracked.

**Features:**

- Scans from home directory (or `nspira scan <path>`)
- A folder matching several patterns (e.g. Node.js + Rust) gets all their caches
- Nested projects (monorepo packages, `src-tauri/`) merge into the outermost project
- Track all or some of a project's cache directories
- `x` ignores a project or a single cache directory in future scans
- Tracked and ignored paths are skipped with their whole subtree

### `nspira ignore`

Manage paths that scans skip.

```bash
nspira ignore list            # Show ignored paths
nspira ignore add ~/archive   # Skip a folder and everything under it
nspira ignore remove ~/archive
```

### `nspira stats`

Show cache statistics across all projects.

### `nspira remove <id>`

Remove a project from tracking.

```bash
nspira remove 1
```

### `nspira flush`

Delete the entire database (use with caution!).

### `nspira config`

Manage nspira configuration.

**Subcommands:**
```bash
nspira config show    # Display current configuration
nspira config path    # Show configuration file location
nspira config reset   # Reset configuration to defaults
```

---

## Interactive TUI Guide

### List Command (`nspira list`)

```
╭ Projects ─────────────────────────────────────────────────────────────╮
│      Name         Path                Last cleaned        Size  Caches│
│> [-] - tauri-app  ~/dev/tauri-app     2026-11-20 10:02  1.2 GB       2│
│  [x]                node_modules                       310 MB         │
│  [ ]                src-tauri/target                   920 MB         │
│  [ ] + api        ~/dev/api           Never             88 MB       1│
╰───────────────────────────────────────────────────────────────────────╯
```

**Controls:**

- `Up/Down` or `j/k` - Move, `g/G` top/bottom, `PgUp/PgDn` page
- `Right/l` or `Enter` - Expand a project, `Left/h` - collapse
- `Space` - Select a project (all its caches) or one cache dir
- `a` - Select all / none
- `c` - Clean selected caches (or the one under the cursor)
- `d` - Stop tracking (whole project if fully selected, else just those dirs)
- `/` - Filter, `s` - sort, `i` - details, `Tab` - stats
- `?` - Help, `q` - quit

The TUI uses your terminal's 16 ANSI colors, so it follows your terminal theme.

### Doctor Command (`nspira doctor`)

```
┌─────────────────────────────────────┐
│         Project Health Report       │
├─────────────────────────────────────┤
│ Total Projects:           5         │
│ Healthy:                  3         │
│ Issues:                   2         │
└─────────────────────────────────────┘
```

**Features:**

- Color-coded health status
- Detailed issue reporting
- Direct removal of problematic projects
- Actionable recommendations

---

## ⚙️ Configuration

### Configuration File

Nspira uses a TOML configuration file that is automatically created on first run.

**Location:**
- **macOS**: `~/Library/Application Support/nspira/config.toml`
- **Linux**: `~/.config/nspira/config.toml`
- **Windows**: `%APPDATA%\nspira\config.toml`

**Commands:**
```bash
nspira config show    # View current configuration
nspira config path    # Get config file location
nspira config reset   # Reset to defaults
```

### Configuration Options

```toml
[scan]
# Maximum depth for filesystem scanning
max_depth = 4

# Directories to skip during scanning
skip_directories = [
    "Library",
    "System",
    "Applications",
    ".Trash",
    "node_modules",
]

[clean]
# Ask for confirmation before cleaning
confirm_before_clean = true

# Track cleaning history in database
enable_history = true
```

### Customizing Configuration

Edit the config file directly:

```bash
# Get config path
nspira config path

# Edit with your editor
vim "$(nspira config path)"
```

### Custom Project Patterns

For custom project detection, edit `~/.config/nspira/patterns.json`:

```json
{
  "patterns": [
    {
      "name": "My Framework",
      "identifier": "myframework.json",
      "cache_dirs": ["cache", "temp", "build"]
    }
  ],
  "skip_dirs": [".git", ".svn"]
}
```

### Database Location

- **Development**: `./nspira.db`
- **Production**: `~/.local/share/nspira/nspira.db` (Linux/macOS)
- **Production**: `%APPDATA%\nspira\nspira.db` (Windows)

---

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/notenderdreams/nspira
cd nspira
cargo build
cargo test
```

---

## Contributing

We welcome contributions! Please feel free to submit issues, feature requests, or pull requests.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🆘 Troubleshooting

### Common Issues

**"Project not found" errors:**

- Ensure project paths are correct
- Use `nspira doctor` to check project health

**Permission errors:**

- Run with appropriate permissions for cache directories
- Check database file permissions

**Database issues:**

- Use `nspira flush` to reset (⚠️ deletes all data)
- Check disk space

### Getting Help

- 📖 Check this README
- 🐛 [Open an Issue](https://github.com/notenderdreams/nspira/issues)
- 💬 [Discussions](https://github.com/notenderdreams/nspira/discussions)

---

<p align="center">
Made with ❤️ for developers who care about clean systems
</p>
