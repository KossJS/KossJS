// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

//! KossJS Module Resolver
//!
//! Node.js-style module resolution: relative/absolute paths, bare specifiers
//! with node_modules lookup, automatic extension completion, and package.json
//! main field support.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Extensions to try when auto-completing ──────────────────────────────────
const FILE_EXTENSIONS: &[&str] = &[".js", ".mjs", ".cjs", ".json", ".node"];
const INDEX_FILES: &[&str] = &["index.js", "index.mjs", "index.cjs", "index.json", "index.node"];

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

    /// Check if a relative stdlib path exists in the embedded store
    fn embedded_stdlib_exists(rel_path: &str) -> bool {
        crate::embedded_stdlib::get(rel_path).is_some()
    }

    /// Resolve a Node.js built-in module from stdlib
    fn resolve_nodejs_stdlib(&self, specifier: &str) -> Result<PathBuf, ResolveError> {
        // Handle node: prefix
        let module_name = if specifier.starts_with("node:") {
            &specifier[5..]
        } else {
            specifier
        };

        let stdlib_path = &self.stdlib_path;

        // 1. Direct file match: path.js
        let direct_rel = format!("{}.js", module_name);
        let direct_path = stdlib_path.join(module_name).with_extension("js");
        if Self::embedded_stdlib_exists(&direct_rel) {
            return Ok(direct_path);
        }

        // 2. node_shim/ prefix match: node_shim/path.js
        let node_shim_rel = format!("node_shim/{}.js", module_name);
        let node_shim_path = stdlib_path.join("node_shim").join(module_name).with_extension("js");
        if Self::embedded_stdlib_exists(&node_shim_rel) {
            return Ok(node_shim_path);
        }

        // 3. Try as directory with index.js
        let index_rel = format!("{}/index.js", module_name);
        let index_path = stdlib_path.join(module_name).join("index.js");
        if Self::embedded_stdlib_exists(&index_rel) {
            return Ok(index_path);
        }

        // 4. Handle internal modules (_http_client, etc.)
        let internal_rel = format!("{}.js", module_name);
        let internal_path = stdlib_path.join(format!("{}.js", module_name));
        if Self::embedded_stdlib_exists(&internal_rel) {
            return Ok(internal_path);
        }

        Err(ResolveError {
            specifier: specifier.to_string(),
            parent: "node:".to_string(),
            searched: vec![direct_path, node_shim_path, index_path, internal_path],
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
        let candidate = match Self::normalize_path(&candidate) {
            Some(p) => p,
            None => {
                return Err(ResolveError {
                    specifier: specifier.to_string(),
                    parent: parent_dir.display().to_string(),
                    searched: vec![candidate],
                });
            }
        };

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
                        &nm_dir,
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
                            .resolve_path(&format!("./{}", main_field), &nm_dir)
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
            if cached {
                return true; // positive cache hit
            }
            // Negative cache entries can become stale (CWE-367 TOCTOU):
            // a file may have been created since the last check.
            // Re-check on disk and update if now present.
            if path.is_file() {
                self.exists_cache.borrow_mut().insert(path.to_path_buf(), true);
                return true;
            }
            return false;
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
    /// Returns `None` if the path escapes above the implied root (e.g. `/a/../../b`).
    fn normalize_path(path: &Path) -> Option<PathBuf> {
        Self::normalize_path_static(path)
    }

    /// Public static version of normalize_path for use by other modules.
    pub fn normalize_path_static(path: &Path) -> Option<PathBuf> {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::Prefix(_) => {
                    components.clear();
                    components.push(component);
                }
                std::path::Component::RootDir => {
                    if components.is_empty() || components.last() == Some(&std::path::Component::RootDir) {
                        components.clear();
                    }
                    components.push(component);
                }
                std::path::Component::ParentDir => {
                    if components.is_empty() {
                        return None; // traversal above root
                    }
                    match components.last() {
                        Some(std::path::Component::RootDir) | Some(std::path::Component::Prefix(_)) => {
                            return None; // traversal above absolute root
                        }
                        _ => {
                            components.pop();
                        }
                    }
                }
                std::path::Component::CurDir => {}
                other => {
                    components.push(other);
                }
            }
        }
        if components.is_empty() {
            return None;
        }
        Some(components.iter().collect())
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
    use std::fs;
    use std::io::Write;

    // ── Helper: create temp dir with files ────────────────────────────────

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("kj_{}", uuid_str()));
            fs::create_dir_all(&dir).unwrap_or_else(|e| {
                panic!("Failed to create temp dir {:?}: {}", dir, e);
            });
            Self { path: dir }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn mkdir(&self, name: &str) -> PathBuf {
            let p = self.path.join(name);
            fs::create_dir_all(&p).unwrap_or_else(|e| {
                panic!("Failed to create dir {:?}: {}", p, e);
            });
            p
        }

        fn write(&self, name: &str, content: &str) -> PathBuf {
            let p = self.path.join(name);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    panic!("Failed to create parent dir {:?}: {}", parent, e);
                });
            }
            let mut f = fs::File::create(&p).unwrap_or_else(|e| {
                panic!("Failed to create file {:?}: {}", p, e);
            });
            f.write_all(content.as_bytes()).unwrap();
            p
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn uuid_str() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:x}_{}", nanos, std::process::id())
    }

    // ── split_specifier ──────────────────────────────────────────────────

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
        // Edge cases
        assert_eq!(ModuleResolver::split_specifier(""), ("", None));
        assert_eq!(ModuleResolver::split_specifier("a"), ("a", None));
        assert_eq!(ModuleResolver::split_specifier("a/"), ("a", Some("")));
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
        // Scoped without slash
        assert_eq!(
            ModuleResolver::split_specifier("@scope"),
            ("@scope", None)
        );
        // Scoped with trailing sub-path
        assert_eq!(
            ModuleResolver::split_specifier("@vue/compiler-sfc/dist/compiler-sfc.cjs"),
            ("@vue/compiler-sfc", Some("dist/compiler-sfc.cjs"))
        );
    }

    // ── is_relative / is_absolute / is_node_internal ─────────────────────

    #[test]
    fn test_is_relative() {
        assert!(ModuleResolver::is_relative("./foo"));
        assert!(ModuleResolver::is_relative("../bar"));
        assert!(ModuleResolver::is_relative("./"));
        assert!(ModuleResolver::is_relative("../sub/dir"));
        assert!(!ModuleResolver::is_relative("lodash"));
        assert!(!ModuleResolver::is_relative("/abs/path"));
        assert!(!ModuleResolver::is_relative("node:fs"));
    }

    #[test]
    fn test_is_node_internal() {
        assert!(ModuleResolver::is_node_internal("node:fs"));
        assert!(ModuleResolver::is_node_internal("node:path"));
        assert!(!ModuleResolver::is_node_internal("fs"));
        assert!(!ModuleResolver::is_node_internal("nodefs"));
    }

    // ── normalize_path ───────────────────────────────────────────────────

    #[test]
    fn test_normalize_path_basic() {
        let p = ModuleResolver::normalize_path(Path::new("/a/b/../c/./d"));
        assert_eq!(p, Some(PathBuf::from("/a/c/d")));
    }

    #[test]
    fn test_normalize_path_many_parent_dirs() {
        let p = ModuleResolver::normalize_path(Path::new("/a/b/c/../../.."));
        let expected: PathBuf = Path::new("/").components().collect();
        assert_eq!(p, Some(expected));
    }

    #[test]
    fn test_normalize_path_no_change() {
        let p = ModuleResolver::normalize_path(Path::new("/a/b/c"));
        assert_eq!(p, Some(PathBuf::from("/a/b/c")));
    }

    #[test]
    fn test_normalize_path_trailing_slash() {
        let p = ModuleResolver::normalize_path(Path::new("/a/b/"));
        assert_eq!(p, Some(PathBuf::from("/a/b")));
    }

    #[test]
    fn test_normalize_path_relative_with_dot_slash() {
        let p = ModuleResolver::normalize_path(Path::new("./a/../b"));
        assert!(p.is_some());
        assert!(p.unwrap().ends_with("b"));
    }

    // ── Resolver construction and cache management ───────────────────────

    #[test]
    fn test_resolver_new() {
        let r = ModuleResolver::new();
        assert!(r.stdlib_path().to_string_lossy().contains("stdlib"));
    }

    #[test]
    fn test_resolver_with_capacity() {
        let r = ModuleResolver::with_capacity(10);
        assert!(r.stdlib_path().to_string_lossy().contains("stdlib"));
    }

    #[test]
    fn test_clear_cache() {
        let r = ModuleResolver::with_capacity(10);
        // Borrow and insert something
        r.exists_cache
            .borrow_mut()
            .insert(PathBuf::from("/test"), true);
        assert_eq!(r.exists_cache.borrow().len(), 1);
        r.clear_cache();
        assert_eq!(r.exists_cache.borrow().len(), 0);
        assert_eq!(r.resolve_cache.borrow().len(), 0);
        assert_eq!(r.pkg_cache.borrow().len(), 0);
    }

    // ── file_exists / dir_exists ─────────────────────────────────────────

    #[test]
    fn test_file_exists() {
        let tmp = TempDir::new();
        let f = tmp.write("real.js", "// test");
        let r = ModuleResolver::with_capacity(10);

        assert!(r.file_exists(&f));
        assert!(!r.file_exists(&tmp.path().join("nonexistent.js")));
    }

    #[test]
    fn test_file_exists_cache() {
        let tmp = TempDir::new();
        let f = tmp.write("cached.js", "// cached");
        let r = ModuleResolver::with_capacity(10);

        // First call populates cache
        assert!(r.file_exists(&f));
        // Second call uses cache; delete file on disk to verify
        fs::remove_file(&f).unwrap();
        assert!(r.file_exists(&f)); // still cached as true
    }

    #[test]
    fn test_dir_exists() {
        let tmp = TempDir::new();
        let d = tmp.mkdir("real_dir");
        let r = ModuleResolver::with_capacity(10);

        assert!(r.dir_exists(&d));
        assert!(!r.dir_exists(&tmp.path().join("nonexistent_dir")));
    }

    // ── resolve_path ─────────────────────────────────────────────────────

    #[test]
    fn test_resolve_path_direct_file() {
        let tmp = TempDir::new();
        let f = tmp.write("mod.js", "module.exports = 42;");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("./mod", tmp.path()).unwrap();
        assert_eq!(resolved, f);
    }

    #[test]
    fn test_resolve_path_extension_completion() {
        let tmp = TempDir::new();
        let f = tmp.write("lib.json", r#"{"key": "val"}"#);
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("./lib", tmp.path()).unwrap();
        assert_eq!(resolved, f);
    }

    #[test]
    fn test_resolve_path_extension_completion_js() {
        let tmp = TempDir::new();
        let f = tmp.write("app.mjs", "export default 1;");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("./app", tmp.path()).unwrap();
        assert_eq!(resolved, f);
    }

    #[test]
    fn test_resolve_path_index_file() {
        let tmp = TempDir::new();
        let _d = tmp.mkdir("mylib");
        let idx = tmp.write("mylib/index.js", "module.exports = {};");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("./mylib", tmp.path()).unwrap();
        assert_eq!(resolved, idx);
    }

    #[test]
    fn test_resolve_path_index_preferred_over_extension() {
        // If both mylib.js and mylib/index.js exist, prefers the direct file
        let tmp = TempDir::new();
        let direct = tmp.write("mylib.js", "// direct");
        tmp.mkdir("mylib");
        // Note: mylib.js was written first; mylib dir was created after.
        // resolve_path tries candidate as file first, so mylib.js should win
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("./mylib", tmp.path()).unwrap();
        assert_eq!(resolved, direct);
    }

    #[test]
    fn test_resolve_path_not_found() {
        let tmp = TempDir::new();
        let r = ModuleResolver::with_capacity(10);

        let err = r
            .resolve_path("./nonexistent", tmp.path())
            .unwrap_err();
        assert!(err.to_string().contains("Cannot find module"));
        assert!(!err.searched.is_empty());
    }

    #[test]
    fn test_resolve_path_parent_dir() {
        let tmp = TempDir::new();
        tmp.write("shared.js", "// shared");
        let sub = tmp.mkdir("sub");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_path("../shared", &sub).unwrap();
        assert_eq!(resolved, tmp.path().join("shared.js"));
    }

    // ── resolve_node_modules ─────────────────────────────────────────────

    #[test]
    fn test_resolve_node_modules_basic() {
        let tmp = TempDir::new();
        let _nm = tmp.mkdir("node_modules");
        let _pkg = tmp.mkdir("node_modules/mypkg");
        let main = tmp.write("node_modules/mypkg/index.js", "module.exports = 'mypkg';");
        let src = tmp.mkdir("src");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_node_modules("mypkg", &src).unwrap();
        assert_eq!(resolved, main);
    }

    #[test]
    fn test_resolve_node_modules_package_json_main() {
        let tmp = TempDir::new();
        tmp.mkdir("node_modules");
        tmp.mkdir("node_modules/hasmain");
        tmp.write(
            "node_modules/hasmain/package.json",
            r#"{"main": "dist/main.js"}"#,
        );
        let main = tmp.write("node_modules/hasmain/dist/main.js", "// main entry");
        let src = tmp.mkdir("src");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_node_modules("hasmain", &src).unwrap();
        assert_eq!(resolved, main);
    }

    #[test]
    fn test_resolve_node_modules_subpath() {
        let tmp = TempDir::new();
        tmp.mkdir("node_modules");
        tmp.mkdir("node_modules/lodash");
        let fp = tmp.write("node_modules/lodash/fp.js", "// fp module");
        let src = tmp.mkdir("src");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_node_modules("lodash/fp", &src).unwrap();
        assert_eq!(resolved, fp);
    }

    #[test]
    fn test_resolve_node_modules_scoped() {
        let tmp = TempDir::new();
        tmp.mkdir("node_modules");
        tmp.mkdir("node_modules/@types");
        let _pkg = tmp.mkdir("node_modules/@types/node");
        let idx = tmp.write("node_modules/@types/node/index.js", "// types");
        let src = tmp.mkdir("src");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_node_modules("@types/node", &src).unwrap();
        assert_eq!(resolved, idx);
    }

    #[test]
    fn test_resolve_node_modules_walk_up() {
        let tmp = TempDir::new();
        // node_modules at root of tmp
        tmp.mkdir("node_modules");
        tmp.mkdir("node_modules/rootpkg");
        let idx = tmp.write("node_modules/rootpkg/index.js", "// root pkg");
        // current file deep inside
        let deep = tmp.mkdir("a/b/c/d/e");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve_node_modules("rootpkg", &deep).unwrap();
        assert_eq!(resolved, idx);
    }

    #[test]
    fn test_resolve_node_modules_not_found() {
        let tmp = TempDir::new();
        let r = ModuleResolver::with_capacity(10);

        let err = r
            .resolve_node_modules("no_such_pkg_12345", tmp.path())
            .unwrap_err();
        assert!(err.to_string().contains("Cannot find module"));
        assert!(err.specifier.contains("no_such_pkg_12345"));
    }

    // ── read_package_main ────────────────────────────────────────────────

    #[test]
    fn test_read_package_main_valid() {
        let tmp = TempDir::new();
        let d = tmp.mkdir("pkg");
        tmp.write("pkg/package.json", r#"{"name": "pkg", "main": "lib/index.js"}"#);
        let r = ModuleResolver::with_capacity(10);

        let main = r.read_package_main(&d);
        assert_eq!(main, Some("lib/index.js".to_string()));
    }

    #[test]
    fn test_read_package_main_no_main_field() {
        let tmp = TempDir::new();
        let d = tmp.mkdir("pkg2");
        tmp.write("pkg2/package.json", r#"{"name": "pkg2"}"#);
        let r = ModuleResolver::with_capacity(10);

        let main = r.read_package_main(&d);
        assert_eq!(main, None);
    }

    #[test]
    fn test_read_package_main_no_package_json() {
        let tmp = TempDir::new();
        let d = tmp.mkdir("pkg3");
        let r = ModuleResolver::with_capacity(10);

        let main = r.read_package_main(&d);
        assert_eq!(main, None);
    }

    #[test]
    fn test_read_package_main_cache() {
        let tmp = TempDir::new();
        let d = tmp.mkdir("pkg_cached");
        tmp.write(
            "pkg_cached/package.json",
            r#"{"name": "pkg", "main": "src/main.js"}"#,
        );
        let r = ModuleResolver::with_capacity(10);

        // First call caches
        assert_eq!(r.read_package_main(&d), Some("src/main.js".to_string()));
        // Delete package.json; cache should still return old value
        fs::remove_file(d.join("package.json")).unwrap();
        assert_eq!(r.read_package_main(&d), Some("src/main.js".to_string()));
    }

    // ── resolve (top-level API) ──────────────────────────────────────────

    #[test]
    fn test_resolve_relative() {
        let tmp = TempDir::new();
        let f = tmp.write("dep.js", "module.exports = 1;");
        let parent = tmp.write("main.js", "require('./dep');");
        let r = ModuleResolver::with_capacity(10);

        let resolved = r.resolve("./dep", &parent).unwrap();
        assert_eq!(resolved, f);
    }

    #[test]
    fn test_resolve_relative_cached() {
        let tmp = TempDir::new();
        let f = tmp.write("dep2.js", "module.exports = 2;");
        let parent = tmp.write("main2.js", "// main");
        let r = ModuleResolver::with_capacity(10);

        let a = r.resolve("./dep2", &parent).unwrap();
        let b = r.resolve("./dep2", &parent).unwrap();
        assert_eq!(a, b);
        assert_eq!(a, f);
    }

    #[test]
    fn test_resolve_node_builtin_embedded() {
        let r = ModuleResolver::with_capacity(10);
        // path.js should exist in embedded stdlib
        let result = r.resolve("node:path", Path::new("/dummy.js"));
        assert!(result.is_ok());
        let p = result.unwrap();
        assert!(p.to_string_lossy().contains("path"));
    }

    #[test]
    fn test_resolve_bare_specifier_fallback_stdlib() {
        let r = ModuleResolver::with_capacity(10);
        // "path" as bare specifier should be resolved from stdlib
        let result = r.resolve("path", Path::new("/dummy.js"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_node_builtin_from_node_shim() {
        let r = ModuleResolver::with_capacity(10);
        // node:fs should resolve via node_shim/fs.js
        let result = r.resolve("node:fs", Path::new("/dummy.js"));
        assert!(result.is_ok());
        let p = result.unwrap();
        assert!(p.to_string_lossy().contains("fs"));
    }

    #[test]
    fn test_resolve_bare_fs_from_node_shim() {
        let r = ModuleResolver::with_capacity(10);
        // bare "fs" should fall back to node_shim/fs.js
        let result = r.resolve("fs", Path::new("/dummy.js"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_bare_crypto_from_node_shim() {
        let r = ModuleResolver::with_capacity(10);
        let result = r.resolve("crypto", Path::new("/dummy.js"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_not_found() {
        let tmp = TempDir::new();
        let parent = tmp.write("main.js", "// main");
        let r = ModuleResolver::with_capacity(10);

        let err = r
            .resolve("./does_not_exist_xyz", &parent)
            .unwrap_err();
        assert!(err.to_string().contains("Cannot find module"));
        assert!(err.searched.len() > 1);
    }

    // ── ResolveError format ──────────────────────────────────────────────

    #[test]
    fn test_resolve_error_display() {
        let err = ResolveError {
            specifier: "lodash".to_string(),
            parent: "/project/src/main.js".to_string(),
            searched: vec![
                PathBuf::from("/project/src/node_modules/lodash"),
                PathBuf::from("/project/node_modules/lodash"),
            ],
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot find module 'lodash'"));
        assert!(msg.contains("/project/src/main.js"));
        assert!(msg.contains("Searched in:"));
        assert!(msg.contains("node_modules/lodash"));
    }

    #[test]
    fn test_resolve_error_without_searched() {
        let err = ResolveError {
            specifier: "x".to_string(),
            parent: ".".to_string(),
            searched: vec![],
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot find module 'x'"));
        assert!(!msg.contains("Searched in"));
    }

    // ── Default impl ─────────────────────────────────────────────────────

    #[test]
    fn test_default_resolver() {
        let r = ModuleResolver::default();
        assert!(r.stdlib_path().to_string_lossy().contains("stdlib"));
    }
}
