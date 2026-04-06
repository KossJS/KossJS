//! KossJS Module Resolver
//!
//! Node.js-style module resolution: relative/absolute paths, bare specifiers
//! with node_modules lookup, automatic extension completion, and package.json
//! main field support.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Extensions to try when auto-completing ──────────────────────────────────
const FILE_EXTENSIONS: &[&str] = &[".js", ".mjs", ".cjs", ".json"];
const INDEX_FILES: &[&str] = &["index.js", "index.mjs", "index.cjs", "index.json"];

// ── Error type ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub specifier: String,
    pub parent: String,
    pub searched: Vec<PathBuf>,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Cannot find module '{}' from '{}'",
            self.specifier, self.parent
        )?;
        if !self.searched.is_empty() {
            writeln!(f, "Searched in:")?;
            for p in &self.searched {
                writeln!(f, "  - {}", p.display())?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ResolveError {}

// ── Get stdlib path relative to the DLL location ────────────────────────────
fn get_stdlib_path() -> PathBuf {
    // Try to get the stdlib path from an environment variable first
    if let Ok(path) = std::env::var("KOSS_STDLIB_PATH") {
        return PathBuf::from(path);
    }

    // Get the executable/DLL location
    let exe_path = std::env::current_exe()
        .ok()
        .or_else(|| std::env::var("KOSS_DLL_PATH").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    // The stdlib should be in the same directory as the DLL
    let stdlib_path = exe_path
        .parent()
        .map(|p| p.join("src").join("stdlib"))
        .unwrap_or_else(|| PathBuf::from("./src/stdlib"));

    // Fallback to relative path from current directory
    if !stdlib_path.exists() {
        PathBuf::from("./src/stdlib")
    } else {
        stdlib_path
    }
}

// ── Resolver ────────────────────────────────────────────────────────────────
pub struct ModuleResolver {
    /// (specifier, parent_dir) -> resolved absolute path
    resolve_cache: RefCell<HashMap<(String, PathBuf), PathBuf>>,
    /// path -> exists?  (negative caching included)
    exists_cache: RefCell<HashMap<PathBuf, bool>>,
    /// directory -> Option<main field value>
    pkg_cache: RefCell<HashMap<PathBuf, Option<String>>>,
    /// Max entries per cache
    cache_cap: usize,
    /// Path to the stdlib directory
    stdlib_path: PathBuf,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let stdlib_path = get_stdlib_path();
        Self {
            resolve_cache: RefCell::new(HashMap::with_capacity(cap)),
            exists_cache: RefCell::new(HashMap::with_capacity(cap * 4)),
            pkg_cache: RefCell::new(HashMap::with_capacity(cap)),
            cache_cap: cap,
            stdlib_path,
        }
    }

    // ── Public API ──────────────────────────────────────────────────────

    /// Get the stdlib path
    pub fn stdlib_path(&self) -> &Path {
        &self.stdlib_path
    }

    /// Resolve `specifier` relative to `parent_path` (the file that contains the
    /// require/import statement). Returns the absolute path of the resolved module.
    pub fn resolve(&self, specifier: &str, parent_path: &Path) -> Result<PathBuf, ResolveError> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("/")).to_path_buf();

        // Check cache
        let cache_key = (specifier.to_string(), parent_dir.clone());
        if let Some(cached) = self.resolve_cache.borrow().get(&cache_key) {
            return Ok(cached.clone());
        }

        let result = if Self::is_node_internal(specifier) {
            // For node: built-ins, try to load from stdlib first
            self.resolve_nodejs_stdlib(specifier)
        } else if Self::is_relative(specifier) || Self::is_absolute(specifier) {
            self.resolve_path(specifier, &parent_dir)
        } else {
            // First check stdlib, then node_modules
            if let Ok(path) = self.resolve_nodejs_stdlib(specifier) {
                Ok(path)
            } else {
                self.resolve_node_modules(specifier, &parent_dir)
            }
        };

        match result {
            Ok(resolved) => {
                let mut cache = self.resolve_cache.borrow_mut();
                if cache.len() >= self.cache_cap {
                    cache.clear(); // simple eviction
                }
                cache.insert(cache_key, resolved.clone());
                Ok(resolved)
            }
            Err(e) => Err(e),
        }
    }

    /// Resolve a Node.js built-in module from stdlib
    fn resolve_nodejs_stdlib(&self, specifier: &str) -> Result<PathBuf, ResolveError> {
        // Handle node: prefix
        let module_name = if specifier.starts_with("node:") {
            &specifier[5..]
        } else {
            specifier
        };

        // Try to find in stdlib folder
        let stdlib_path = &self.stdlib_path;

        // Direct file match
        let direct_path = stdlib_path.join(module_name).with_extension("js");
        if self.file_exists(&direct_path) {
            return Ok(direct_path);
        }

        // Try as directory with index.js
        let index_path = stdlib_path.join(module_name).join("index.js");
        if self.file_exists(&index_path) {
            return Ok(index_path);
        }

        // Handle internal modules (_http_client, etc.)
        let internal_path = stdlib_path.join(format!("{}.js", module_name));
        if self.file_exists(&internal_path) {
            return Ok(internal_path);
        }

        Err(ResolveError {
            specifier: specifier.to_string(),
            parent: "node:".to_string(),
            searched: vec![direct_path, index_path, internal_path],
        })
    }

    /// Clear all caches (useful when files on disk have changed).
    pub fn clear_cache(&self) {
        self.resolve_cache.borrow_mut().clear();
        self.exists_cache.borrow_mut().clear();
        self.pkg_cache.borrow_mut().clear();
    }

    // ── Path classification ─────────────────────────────────────────────

    fn is_relative(specifier: &str) -> bool {
        specifier.starts_with("./") || specifier.starts_with("../")
    }

    fn is_absolute(specifier: &str) -> bool {
        let p = Path::new(specifier);
        p.is_absolute()
    }

    fn is_node_internal(specifier: &str) -> bool {
        specifier.starts_with("node:")
    }

    // ── Relative / absolute path resolution ─────────────────────────────

    fn resolve_path(&self, specifier: &str, parent_dir: &Path) -> Result<PathBuf, ResolveError> {
        let candidate = if Self::is_absolute(specifier) {
            PathBuf::from(specifier)
        } else {
            parent_dir.join(specifier)
        };

        // Canonicalize as much as possible (resolve .., symlinks, etc.)
        // But don't fail if the path doesn't exist yet — we'll probe below.
        let candidate = Self::normalize_path(&candidate);

        let mut searched = Vec::new();

        // Try candidate as-is (exact file)
        if self.file_exists(&candidate) {
            return Ok(candidate);
        }
        searched.push(candidate.clone());

        // Try with extensions
        for ext in FILE_EXTENSIONS {
            let with_ext = candidate.with_extension(ext.trim_start_matches('.'));
            // Handle case where candidate already has a different extension:
            // e.g. candidate = "foo.bar" → we want "foo.bar.js" not "foo.js"
            let with_ext_appended = PathBuf::from(format!("{}{}", candidate.display(), ext));

            if self.file_exists(&with_ext) {
                return Ok(with_ext);
            }
            searched.push(with_ext.clone());

            if with_ext != with_ext_appended && self.file_exists(&with_ext_appended) {
                return Ok(with_ext_appended);
            }
        }

        // Try as directory with index files
        if self.dir_exists(&candidate) {
            for idx in INDEX_FILES {
                let index_path = candidate.join(idx);
                if self.file_exists(&index_path) {
                    return Ok(index_path);
                }
                searched.push(index_path);
            }
        }

        Err(ResolveError {
            specifier: specifier.to_string(),
            parent: parent_dir.display().to_string(),
            searched,
        })
    }

    // ── Bare specifier / node_modules resolution ────────────────────────

    fn resolve_node_modules(
        &self,
        specifier: &str,
        start_dir: &Path,
    ) -> Result<PathBuf, ResolveError> {
        // Handle scoped packages: @scope/pkg  or  @scope/pkg/sub/path
        // and regular: pkg  or  pkg/sub/path
        let (pkg_name, sub_path) = Self::split_specifier(specifier);

        let mut searched = Vec::new();
        let mut dir = Some(start_dir.to_path_buf());

        while let Some(current) = dir {
            // Skip if current dir is itself named "node_modules" to avoid
            // node_modules/node_modules chains
            let nm_dir = current.join("node_modules").join(pkg_name);

            if self.dir_exists(&nm_dir) {
                searched.push(nm_dir.clone());

                // If there's a sub-path, resolve it relative to the package dir
                if let Some(sub) = sub_path {
                    match self.resolve_path(
                        &format!("./{}", sub),
                        // Use a fake "file" inside nm_dir so parent_dir = nm_dir
                        &nm_dir.join("__dummy__"),
                    ) {
                        Ok(resolved) => return Ok(resolved),
                        Err(mut e) => {
                            searched.append(&mut e.searched);
                        }
                    }
                } else {
                    // Try package.json main
                    if let Some(main_field) = self.read_package_main(&nm_dir) {
                        match self
                            .resolve_path(&format!("./{}", main_field), &nm_dir.join("__dummy__"))
                        {
                            Ok(resolved) => return Ok(resolved),
                            Err(_) => {
                                // main field didn't resolve, fall through to index files
                            }
                        }
                    }

                    // Try index files
                    for idx in INDEX_FILES {
                        let index_path = nm_dir.join(idx);
                        if self.file_exists(&index_path) {
                            return Ok(index_path);
                        }
                        searched.push(index_path);
                    }
                }
            } else {
                searched.push(nm_dir);
            }

            // Move to parent directory
            dir = current.parent().map(|p| p.to_path_buf());
        }

        Err(ResolveError {
            specifier: specifier.to_string(),
            parent: start_dir.display().to_string(),
            searched,
        })
    }

    /// Split a bare specifier into (package_name, optional_sub_path).
    /// Handles scoped packages: "@scope/pkg/foo/bar" → ("@scope/pkg", Some("foo/bar"))
    /// Regular packages: "lodash/fp" → ("lodash", Some("fp"))
    fn split_specifier(specifier: &str) -> (&str, Option<&str>) {
        if specifier.starts_with('@') {
            // Scoped: find second '/'
            if let Some(first_slash) = specifier.find('/') {
                if let Some(second_slash) = specifier[first_slash + 1..].find('/') {
                    let split_at = first_slash + 1 + second_slash;
                    (&specifier[..split_at], Some(&specifier[split_at + 1..]))
                } else {
                    (specifier, None)
                }
            } else {
                (specifier, None)
            }
        } else {
            // Regular: split at first '/'
            if let Some(slash) = specifier.find('/') {
                (&specifier[..slash], Some(&specifier[slash + 1..]))
            } else {
                (specifier, None)
            }
        }
    }

    // ── package.json parsing ────────────────────────────────────────────

    fn read_package_main(&self, pkg_dir: &Path) -> Option<String> {
        // Check cache
        if let Some(cached) = self.pkg_cache.borrow().get(pkg_dir) {
            return cached.clone();
        }

        let pkg_json_path = pkg_dir.join("package.json");
        let result = if self.file_exists(&pkg_json_path) {
            std::fs::read_to_string(&pkg_json_path)
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<serde_json::Value>(&content)
                        .ok()
                        .and_then(|v| v.get("main").and_then(|m| m.as_str().map(String::from)))
                })
        } else {
            None
        };

        let mut cache = self.pkg_cache.borrow_mut();
        if cache.len() >= self.cache_cap {
            cache.clear();
        }
        cache.insert(pkg_dir.to_path_buf(), result.clone());
        result
    }

    // ── File system helpers (with caching) ──────────────────────────────

    fn file_exists(&self, path: &Path) -> bool {
        if let Some(&cached) = self.exists_cache.borrow().get(path) {
            return cached;
        }

        let exists = path.is_file();

        let mut cache = self.exists_cache.borrow_mut();
        if cache.len() >= self.cache_cap * 4 {
            cache.clear();
        }
        cache.insert(path.to_path_buf(), exists);
        exists
    }

    fn dir_exists(&self, path: &Path) -> bool {
        path.is_dir()
    }

    /// Normalize a path: resolve `.` and `..` components without requiring the
    /// path to exist on disk (unlike std::fs::canonicalize).
    fn normalize_path(path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                other => {
                    components.push(other);
                }
            }
        }
        components.iter().collect()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_specifier_regular() {
        assert_eq!(ModuleResolver::split_specifier("lodash"), ("lodash", None));
        assert_eq!(
            ModuleResolver::split_specifier("lodash/fp"),
            ("lodash", Some("fp"))
        );
        assert_eq!(
            ModuleResolver::split_specifier("lodash/fp/object"),
            ("lodash", Some("fp/object"))
        );
    }

    #[test]
    fn test_split_specifier_scoped() {
        assert_eq!(
            ModuleResolver::split_specifier("@babel/core"),
            ("@babel/core", None)
        );
        assert_eq!(
            ModuleResolver::split_specifier("@babel/core/lib/parse"),
            ("@babel/core", Some("lib/parse"))
        );
    }

    #[test]
    fn test_is_relative() {
        assert!(ModuleResolver::is_relative("./foo"));
        assert!(ModuleResolver::is_relative("../bar"));
        assert!(!ModuleResolver::is_relative("lodash"));
        assert!(!ModuleResolver::is_relative("/abs/path"));
    }

    #[test]
    fn test_normalize_path() {
        let p = ModuleResolver::normalize_path(Path::new("/a/b/../c/./d"));
        assert_eq!(p, PathBuf::from("/a/c/d"));
    }
}
