use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectPattern {
    pub name: String,
    /// File that marks a project root. `*.ext` matches any file with that extension.
    pub identifier: String,
    /// Cache dirs relative to the project root.
    pub cache_dirs: Vec<String>,
}

impl ProjectPattern {
    fn matches(&self, dir: &Path) -> bool {
        match self.identifier.strip_prefix("*.") {
            Some(ext) => fs::read_dir(dir).is_ok_and(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path().extension().is_some_and(|x| x == ext)
                        && e.file_type().is_ok_and(|t| t.is_file())
                })
            }),
            None => dir.join(&self.identifier).exists(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanConfig {
    pub patterns: Vec<ProjectPattern>,
    pub skip_dirs: Vec<String>,
}

impl ScanConfig {
    /// User override at `<config>/nspira/patterns.json`, else the built-in defaults.
    pub fn load() -> Result<Self> {
        if let Some(path) = dirs::config_dir().map(|d| d.join("nspira").join("patterns.json"))
            && path.exists()
        {
            return Ok(serde_json::from_str(&fs::read_to_string(path)?)?);
        }
        Ok(serde_json::from_str(crate::DEFAULT_PATTERNS)?)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedProject {
    pub name: String,
    pub path: PathBuf,
    /// Every pattern that matched, e.g. `["Node.js", "Rust"]`.
    pub kinds: Vec<String>,
    pub cache_dirs: Vec<PathBuf>,
}

impl DetectedProject {
    pub fn kind_label(&self) -> String {
        self.kinds.join(" + ")
    }
}

/// Detect a project at `dir` using every matching pattern. `None` if no pattern
/// matches or none of the matched patterns' cache dirs exist.
pub fn detect_at(dir: &Path, patterns: &[ProjectPattern]) -> Option<DetectedProject> {
    let mut kinds = Vec::new();
    let mut cache_dirs = Vec::new();

    for pattern in patterns.iter().filter(|p| p.matches(dir)) {
        let mut found = false;
        for cache in &pattern.cache_dirs {
            let cache_path = dir.join(cache);
            if cache_path.is_dir() {
                found = true;
                if !cache_dirs.contains(&cache_path) {
                    cache_dirs.push(cache_path);
                }
            }
        }
        if found && !kinds.contains(&pattern.name) {
            kinds.push(pattern.name.clone());
        }
    }

    if cache_dirs.is_empty() {
        return None;
    }
    Some(DetectedProject {
        name: dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        path: dir.to_path_buf(),
        kinds,
        cache_dirs,
    })
}

/// Fold projects nested inside another detected project into the outermost one,
/// then drop cache dirs that sit inside another cache dir of the same project.
pub fn merge_nested(mut projects: Vec<DetectedProject>) -> Vec<DetectedProject> {
    // Component-wise ordering keeps every descendant directly after its ancestor.
    projects.sort_by(|a, b| a.path.cmp(&b.path));

    let mut roots: Vec<DetectedProject> = Vec::new();
    for p in projects {
        match roots.last_mut() {
            Some(root) if p.path.starts_with(&root.path) => {
                for c in p.cache_dirs {
                    if !root.cache_dirs.contains(&c) {
                        root.cache_dirs.push(c);
                    }
                }
                for k in p.kinds {
                    if !root.kinds.contains(&k) {
                        root.kinds.push(k);
                    }
                }
            }
            _ => roots.push(p),
        }
    }

    for root in &mut roots {
        let all = root.cache_dirs.clone();
        root.cache_dirs
            .retain(|c| !all.iter().any(|d| d != c && c.starts_with(d)));
        root.cache_dirs.sort();
    }
    roots
}

/// True for the home directory and its ancestors (incl. filesystem roots).
/// A stray `~/package.json` must not turn home into one giant project that
/// swallows every real project under it.
pub fn is_container_dir(dir: &Path, home: Option<&Path>) -> bool {
    dir.parent().is_none() || home.is_some_and(|h| h.starts_with(dir))
}

pub struct Scanner {
    config: ScanConfig,
    /// Subtrees never walked: tracked projects and ignored paths.
    excluded: Vec<PathBuf>,
    home: Option<PathBuf>,
}

impl Scanner {
    pub fn new(config: ScanConfig, excluded: Vec<PathBuf>) -> Self {
        Self {
            config,
            excluded,
            home: dirs::home_dir(),
        }
    }

    pub fn scan(&self, roots: &[PathBuf], max_depth: usize) -> Vec<DetectedProject> {
        let dirs = self.collect_dirs(roots, max_depth);

        let progress = ProgressBar::new(dirs.len() as u64);
        progress.set_style(
            ProgressStyle::with_template("  {spinner:.cyan} {bar:30.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=> "),
        );
        progress.set_message("detecting projects");

        let detected: Vec<DetectedProject> = dirs
            .par_iter()
            .filter_map(|dir| {
                let found = (!is_container_dir(dir, self.home.as_deref()))
                    .then(|| detect_at(dir, &self.config.patterns))
                    .flatten();
                progress.inc(1);
                found
            })
            .collect();

        progress.finish_and_clear();

        // Ignored cache dirs can sit inside an otherwise wanted project.
        let mut projects = merge_nested(detected);
        for p in &mut projects {
            p.cache_dirs
                .retain(|c| !self.excluded.iter().any(|x| c.starts_with(x)));
        }
        projects.retain(|p| !p.cache_dirs.is_empty());
        projects
    }

    fn collect_dirs(&self, roots: &[PathBuf], max_depth: usize) -> Vec<PathBuf> {
        let norm = |s: &str| {
            if cfg!(windows) {
                s.to_lowercase()
            } else {
                s.to_string()
            }
        };

        // Single names match any path component; entries with a separator
        // (e.g. ".local/share") match as a path suffix.
        let (multi, single): (Vec<&String>, Vec<&String>) = self
            .config
            .skip_dirs
            .iter()
            .partition(|s| s.contains('/') || s.contains('\\'));
        let single: HashSet<String> = single.into_iter().map(|s| norm(s)).collect();
        let multi: Vec<PathBuf> = multi.into_iter().map(PathBuf::from).collect();

        let mut seen = HashSet::new();
        let mut dirs = Vec::new();

        for root in roots.iter().filter(|r| r.exists()) {
            let walker = WalkDir::new(root)
                .max_depth(max_depth)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let path = e.path();
                    if e.depth() > 0 && single.contains(&norm(&e.file_name().to_string_lossy())) {
                        return false;
                    }
                    !multi.iter().any(|m| path.ends_with(m))
                        && !self.excluded.iter().any(|x| path.starts_with(x))
                });

            for entry in walker.filter_map(|e| e.ok()) {
                if entry.file_type().is_dir() && seen.insert(entry.path().to_path_buf()) {
                    dirs.push(entry.into_path());
                }
            }
        }
        dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proj(path: &str, kinds: &[&str], caches: &[&str]) -> DetectedProject {
        DetectedProject {
            name: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into(),
            path: path.into(),
            kinds: kinds.iter().map(|s| s.to_string()).collect(),
            cache_dirs: caches.iter().map(PathBuf::from).collect(),
        }
    }

    #[test]
    fn nested_projects_merge_into_root() {
        let merged = merge_nested(vec![
            proj("/w/app/src-tauri", &["Rust"], &["/w/app/src-tauri/target"]),
            proj("/w/app", &["Node.js"], &["/w/app/node_modules"]),
            proj("/w/app-two", &["Go"], &["/w/app-two/bin"]),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].path, PathBuf::from("/w/app"));
        assert_eq!(merged[0].kinds, vec!["Node.js", "Rust"]);
        assert_eq!(
            merged[0].cache_dirs,
            vec![
                PathBuf::from("/w/app/node_modules"),
                PathBuf::from("/w/app/src-tauri/target")
            ]
        );
        assert_eq!(merged[1].path, PathBuf::from("/w/app-two"));
    }

    #[test]
    fn caches_inside_other_caches_are_dropped() {
        let merged = merge_nested(vec![
            proj("/w/a", &["Node.js"], &["/w/a/build"]),
            proj("/w/a/build/x", &["Rust"], &["/w/a/build/x/target"]),
        ]);
        assert_eq!(merged[0].cache_dirs, vec![PathBuf::from("/w/a/build")]);
    }

    #[test]
    fn home_and_its_ancestors_are_never_projects() {
        let home = Path::new("/Users/me");
        assert!(is_container_dir(home, Some(home)));
        assert!(is_container_dir(Path::new("/Users"), Some(home)));
        assert!(is_container_dir(Path::new("/"), Some(home)));
        assert!(!is_container_dir(
            Path::new("/Users/me/dev/app"),
            Some(home)
        ));
        assert!(!is_container_dir(Path::new("/Users/other"), Some(home)));
    }

    #[test]
    fn stray_marker_in_home_does_not_swallow_projects() {
        let d = tmp("home");
        fs::write(d.join("package.json"), "{}").unwrap();
        fs::create_dir_all(d.join("node_modules")).unwrap();
        fs::create_dir_all(d.join("dev/app/target")).unwrap();
        fs::write(d.join("dev/app/Cargo.toml"), "").unwrap();

        let config = ScanConfig {
            patterns: vec![
                ProjectPattern {
                    name: "Node.js".into(),
                    identifier: "package.json".into(),
                    cache_dirs: vec!["node_modules".into()],
                },
                ProjectPattern {
                    name: "Rust".into(),
                    identifier: "Cargo.toml".into(),
                    cache_dirs: vec!["target".into()],
                },
            ],
            skip_dirs: vec![],
        };
        let mut scanner = Scanner::new(config, vec![]);
        scanner.home = Some(d.clone());
        let found = scanner.scan(std::slice::from_ref(&d), 5);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, d.join("dev/app"));
        fs::remove_dir_all(&d).unwrap();
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nspira-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn detect_unions_all_matching_patterns_and_globs() {
        let d = tmp("detect");
        fs::write(d.join("package.json"), "{}").unwrap();
        fs::write(d.join("App.csproj"), "").unwrap();
        fs::create_dir_all(d.join("node_modules")).unwrap();
        fs::create_dir_all(d.join("obj")).unwrap();

        let patterns = vec![
            ProjectPattern {
                name: "Node.js".into(),
                identifier: "package.json".into(),
                cache_dirs: vec!["node_modules".into()],
            },
            ProjectPattern {
                name: ".NET".into(),
                identifier: "*.csproj".into(),
                cache_dirs: vec!["bin".into(), "obj".into()],
            },
            ProjectPattern {
                name: "Rust".into(),
                identifier: "Cargo.toml".into(),
                cache_dirs: vec!["target".into()],
            },
        ];
        let p = detect_at(&d, &patterns).unwrap();
        assert_eq!(p.kinds, vec!["Node.js", ".NET"]);
        assert_eq!(p.cache_dirs.len(), 2);
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn scan_skips_excluded_subtrees_and_multi_component_skips() {
        let d = tmp("scan");
        for sub in ["ignored/p", "kept/p", ".local/share/p"] {
            let p = d.join(sub);
            fs::create_dir_all(p.join("target")).unwrap();
            fs::write(p.join("Cargo.toml"), "").unwrap();
        }
        let config = ScanConfig {
            patterns: vec![ProjectPattern {
                name: "Rust".into(),
                identifier: "Cargo.toml".into(),
                cache_dirs: vec!["target".into()],
            }],
            skip_dirs: vec![".local/share".into()],
        };
        let found = Scanner::new(config, vec![d.join("ignored")]).scan(std::slice::from_ref(&d), 5);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, d.join("kept/p"));
        fs::remove_dir_all(&d).unwrap();
    }
}
