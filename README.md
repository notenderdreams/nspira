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

- View all projects with sizes and last cleaned dates
- Multi-select projects with spacebar
- Clean selected projects with progress bar
- Remove projects from tracking
- View cache directory details
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

- Scans from home directory
- Multi-select interface to add projects
- Smart pattern matching
- Skip already tracked projects

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
┌───────────────────────────────────────────────────────────────────┐
│ ID  Name               Path                 Size    Last Cleaned  │
│ 1   my-react-app       /home/user/projects  245 MB  2024-01-15    │
│ 2   rust-project       /home/user/dev       189 MB  2024-01-14    │
│ ✓ 3 java-app           /home/user/work      567 MB  2024-01-10    │
└───────────────────────────────────────────────────────────────────┘
```

**Controls:**

- `↑/↓` or `j/k` - Navigate projects
- `Space` - Select/deselect project
- `a` - Select/deselect all projects
- `Enter` - Clean selected projects
- `d` - Remove project from tracking
- `Tab` - Toggle between stats and cache view
- `q` - Quit

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
