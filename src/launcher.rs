use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub path: PathBuf,
    pub terminal: bool,
    pub generic_name: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub comment: Option<String>,
}

static APP_CACHE: Lazy<Mutex<Vec<DesktopEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub struct AppLauncher {
    matcher: SkimMatcherV2,
}

impl AppLauncher {
    pub fn new() -> Self {
        let mut cache = APP_CACHE.lock().unwrap();
        if cache.is_empty() {
            *cache = Self::scan_applications();
        }
        
        AppLauncher {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn refresh_cache(&self) {
        let mut cache = APP_CACHE.lock().unwrap();
        *cache = Self::scan_applications();
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<(DesktopEntry, i64)> {
        let cache = APP_CACHE.lock().unwrap();
        let mut results: Vec<(DesktopEntry, i64)> = cache
            .iter()
            .filter(|entry| !entry.exec.is_empty())
            .filter_map(|entry| {
                let name_score = self.matcher.fuzzy_match(&entry.name, query)?;
                let generic_score = entry.generic_name.as_ref()
                    .and_then(|g| self.matcher.fuzzy_match(g, query))
                    .unwrap_or(0);
                let keyword_score = entry.keywords.iter()
                    .filter_map(|k| self.matcher.fuzzy_match(k, query))
                    .max()
                    .unwrap_or(0);
                let comment_score = entry.comment.as_ref()
                    .and_then(|c| self.matcher.fuzzy_match(c, query))
                    .unwrap_or(0);
                
                let total_score = name_score * 10 + generic_score * 3 + keyword_score * 2 + comment_score;
                Some((entry.clone(), total_score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().take(max_results).collect()
    }

    fn scan_applications() -> Vec<DesktopEntry> {
        let mut entries = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let xdg_dirs: Vec<PathBuf> = std::env::var_os("XDG_DATA_DIRS")
            .map(|dirs| std::env::split_paths(&dirs).map(PathBuf::from).collect())
            .unwrap_or_default();

        let data_dirs = dirs::data_dir()
            .into_iter()
            .chain(xdg_dirs.into_iter().flat_map(|p| [p.join("applications"), p]))
            .chain([PathBuf::from("/usr/share/applications")])
            .chain([PathBuf::from("/usr/local/share/applications")])
            .chain([PathBuf::from("/var/lib/flatpak/exports/share/applications")])
            .chain(dirs::home_dir().into_iter().map(|p| p.join(".local/share/flatpak/exports/share/applications")))
            .collect::<Vec<_>>();

        for dir in data_dirs {
            if let Ok(reader) = fs::read_dir(&dir) {
                for entry in reader.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        if seen.insert(id) {
                            if let Some(desktop) = Self::parse_desktop_file(&path) {
                                entries.push(desktop);
                            }
                        }
                    }
                }
            }
        }

        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        entries
    }

    fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
        let content = fs::read_to_string(path).ok()?;
        
        let mut in_desktop_entry = false;
        let mut name = None;
        let mut name_localized = None;
        let mut exec = None;
        let mut icon = String::new();
        let mut terminal = false;
        let mut no_display = false;
        let mut hidden = false;
        let mut generic_name = None;
        let mut generic_name_localized = None;
        let mut categories = Vec::new();
        let mut keywords = Vec::new();
        let mut comment = None;
        let mut comment_localized = None;

        let locale = std::env::var("LANG").unwrap_or_default();
        let lang_prefix = locale.split('.').next().unwrap_or("");
        let short_lang = lang_prefix.split('_').next().unwrap_or("");

        for line in content.lines() {
            let line = line.trim();
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }
            if line.starts_with('[') {
                in_desktop_entry = false;
                continue;
            }
            if !in_desktop_entry || line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                if key == "Name" {
                    name = Some(value.to_string());
                } else if key.starts_with("Name[") {
                    let l = &key[5..key.len()-1];
                    if l == lang_prefix || l == short_lang {
                        name_localized = Some(value.to_string());
                    }
                } else if key == "Exec" {
                    exec = Some(value.to_string());
                } else if key == "Icon" {
                    icon = value.to_string();
                } else if key == "Terminal" {
                    terminal = value == "true";
                } else if key == "NoDisplay" {
                    no_display = value == "true";
                } else if key == "Hidden" {
                    hidden = value == "true";
                } else if key == "GenericName" {
                    generic_name = Some(value.to_string());
                } else if key.starts_with("GenericName[") {
                    let l = &key[12..key.len()-1];
                    if l == lang_prefix || l == short_lang {
                        generic_name_localized = Some(value.to_string());
                    }
                } else if key == "Categories" {
                    categories = value.split(';').map(|s| s.to_string()).collect();
                } else if key == "Keywords" {
                    keywords = value.split(';').map(|s| s.to_string()).collect();
                } else if key == "Comment" {
                    comment = Some(value.to_string());
                } else if key.starts_with("Comment[") {
                    let l = &key[8..key.len()-1];
                    if l == lang_prefix || l == short_lang {
                        comment_localized = Some(value.to_string());
                    }
                }
            }
        }

        let name = name_localized.or(name)?;
        let generic_name = generic_name_localized.or(generic_name);
        let comment = comment_localized.or(comment);

        if no_display || hidden {
            return None;
        }

        let exec = exec?;
        let exec_clean = exec
            .replace(" %f", "")
            .replace(" %F", "")
            .replace(" %u", "")
            .replace(" %U", "")
            .replace(" %d", "")
            .replace(" %D", "")
            .replace(" %n", "")
            .replace(" %N", "")
            .replace(" %i", "")
            .replace(" %c", "")
            .replace(" %k", "")
            .replace(" %v", "")
            .replace(" %m", "");

        Some(DesktopEntry {
            name,
            exec: exec_clean,
            icon,
            path: path.to_path_buf(),
            terminal,
            generic_name,
            categories,
            keywords,
            comment,
        })
    }

    pub fn launch(entry: &DesktopEntry) -> Result<(), String> {
        let exec = entry.exec.trim();
        if exec.is_empty() {
            return Err("Empty command".to_string());
        }

        if entry.terminal {
            let term = std::env::var("TERMINAL").unwrap_or_else(|_| "kitty".to_string());
            std::process::Command::new(&term)
                .arg("-e")
                .arg("sh")
                .arg("-c")
                .arg(exec)
                .spawn()
                .map_err(|e| format!("Failed to launch: {}", e))?;
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(exec)
                .spawn()
                .map_err(|e| format!("Failed to launch: {}", e))?;
        }
        Ok(())
    }
}
