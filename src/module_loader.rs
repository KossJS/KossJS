// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

//! KossJS Module Loader — integrates [ModuleResolver] with BOA's module system.
//!
//! Implements the `ModuleLoader` trait so that `import` / `require` in JS code
//! automatically goes through Node.js-style resolution.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use std::rc::Rc;

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsError, JsNativeError, JsResult, JsString, Module, Source};
use boa_gc::GcRefCell;
use rustc_hash::FxHashMap;

use crate::resolver::ModuleResolver;

fn verify_module_path(root: &Path, resolved: &Path) -> Result<PathBuf, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve module root '{}': {e}", root.display()))?;
    let canonical_resolved = resolved
        .canonicalize()
        .map_err(|e| format!("cannot resolve module '{}': {e}", resolved.display()))?;
    if !canonical_resolved.starts_with(&canonical_root) {
        return Err(format!(
            "module '{}' resolves outside root directory",
            resolved.display()
        ));
    }
    Ok(canonical_resolved)
}

/// A BOA-compatible module loader that uses [ModuleResolver] for Node.js-style
/// module resolution (bare specifiers, node_modules lookup, extension completion),
/// and intercepts `koss:` protocol specifiers for builtin module resolution.
pub struct KossModuleLoader {
    /// The base directory for resolving the initial entry point.
    root: PathBuf,
    /// The underlying resolver with caching.
    resolver: ModuleResolver,
    /// Cache of already-parsed modules keyed by their canonical path.
    module_map: GcRefCell<FxHashMap<PathBuf, Module>>,
    /// Builtin module flags (KOSS_BUILTIN_*) from the KossInstance.
    builtins: u32,
}

impl KossModuleLoader {
    /// Create a new loader rooted at the given directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            resolver: ModuleResolver::new(),
            module_map: GcRefCell::default(),
            builtins: crate::builtins::KOSS_BUILTIN_ALL,
        }
    }

    /// Create a new loader with specific builtin flags.
    pub fn new_with_builtins<P: AsRef<Path>>(root: P, builtins: u32) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            resolver: ModuleResolver::new(),
            module_map: GcRefCell::default(),
            builtins,
        }
    }

    /// Get current builtin flags.
    pub fn builtins(&self) -> u32 {
        self.builtins
    }

    /// Access the underlying resolver (e.g. for direct resolve calls from FFI).
    pub fn resolver(&self) -> &ModuleResolver {
        &self.resolver
    }

    /// Insert a pre-parsed module into the cache.
    pub fn insert(&self, path: PathBuf, module: Module) {
        self.module_map.borrow_mut().insert(path, module);
    }

    /// Get a cached module by path.
    pub fn get(&self, path: &Path) -> Option<Module> {
        self.module_map.borrow().get(path).cloned()
    }

    /// Determine the referrer file path from a `Referrer`.
    fn referrer_file(&self, referrer: &Referrer) -> PathBuf {
        referrer
            .path()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.root.join("__entry__.js"))
    }
}

impl ModuleLoader for KossModuleLoader {
    fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<Module>> {
        let result = (|| {
            let spec = specifier.to_std_string_escaped();

            // ── koss: protocol interception ────────────────────────────
            if crate::builtins::is_koss_specifier(&spec) {
                let (source, _is_internal) =
                    crate::builtins::resolve_builtin_specifier(&spec, self.builtins)
                        .map_err(|msg| {
                            JsError::from(JsNativeError::typ().with_message(msg))
                        })?;
                // Wrap CJS source for ESM import compatibility
                let wrapped = wrap_cjs_for_esm(&source);
                let src = Source::from_bytes(wrapped.as_bytes());
                let module = Module::parse(src, None, &mut context.borrow_mut()).map_err(|err| {
                    JsError::from(
                        JsNativeError::syntax()
                            .with_message(format!("could not parse builtin module '{}'", spec))
                            .with_cause(err),
                    )
                })?;
                return Ok(module);
            }

            let parent_path = self.referrer_file(&referrer);

            // Resolve the module path using our Node.js-style resolver
            let resolved = self
                .resolver
                .resolve(&spec, &parent_path)
                .map_err(|e| JsError::from(JsNativeError::typ().with_message(e.to_string())))?;

            // Check module cache
            if let Some(module) = self.get(&resolved) {
                return Ok(module);
            }

            // Determine if the resolved path is under the stdlib directory
            let stdlib_path = self.resolver.stdlib_path();
            let stdlib_rel = resolved.strip_prefix(stdlib_path).ok().and_then(|r| {
                let s = r.to_str()?.replace('\\', "/");
                Some(s)
            });

            // Read the module source
            let source_bytes = if let Some(rel) = stdlib_rel {
                // Stdlib module: use directly embedded JS source
                match crate::embedded_stdlib::get(&rel) {
                    Some(content) => content.as_bytes().to_vec(),
                    None => {
                        return Err(JsError::from(
                            JsNativeError::typ().with_message(format!(
                                "cannot load stdlib module '{}': '{}' not found",
                                spec, rel,
                            )),
                        ));
                    }
                }
            } else {
                let verified = verify_module_path(&self.root, &resolved)
                    .map_err(|e| JsError::from(JsNativeError::typ().with_message(e)))?;
                std::fs::read(&verified).map_err(|err| {
                    JsError::from(JsNativeError::typ().with_message(format!(
                        "cannot read module '{}': {}",
                        verified.display(),
                        err
                    )))
                })?
            };
            // Wrap CJS source for ESM import compatibility
            let source_str = String::from_utf8_lossy(&source_bytes).to_string();
            let wrapped = wrap_cjs_for_esm(&source_str);
            let source = Source::from_bytes(wrapped.as_bytes());

            let module = Module::parse(source, None, &mut context.borrow_mut()).map_err(|err| {
                JsError::from(
                    JsNativeError::syntax()
                        .with_message(format!("could not parse module '{}'", spec))
                        .with_cause(err),
                )
            })?;

            // Cache and return
            self.insert(resolved, module.clone());
            Ok(module)
        })();

        async { result }
    }
}

/// Check if a JS source string contains ESM export declarations.
fn has_esm_exports(source: &str) -> bool {
    // Simple heuristic: look for 'export' keyword at statement level
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("export ") || trimmed.starts_with("export{") || trimmed.starts_with("export\n") {
            return true;
        }
    }
    false
}

/// Extract named exports from CJS `module.exports = { a, b, c }` pattern.
#[allow(unused)]
fn extract_cjs_named_exports(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        // Match: module.exports = { name1, name2, ... }
        if let Some(idx) = trimmed.find("module.exports") {
            let rest = &trimmed[idx..];
            if let Some(brace_start) = rest.find('{') {
                if let Some(brace_end) = rest[brace_start..].find('}') {
                    let inner = &rest[brace_start + 1..brace_start + brace_end];
                    for part in inner.split(',') {
                        let name = part.trim();
                        // Skip empty, destructuring, assignments
                        if !name.is_empty()
                            && !name.contains(':')
                            && !name.contains('=')
                            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }
        // Match: exports.name = ...
        if trimmed.starts_with("exports.") {
            if let Some(eq_pos) = trimmed.find('=') {
                let name_part = &trimmed[7..eq_pos].trim();
                if !name_part.is_empty()
                    && name_part
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_')
                {
                    names.push(name_part.to_string());
                }
            }
        }
    }
    names
}

/// Wrap a CJS module source for ESM import compatibility.
///
/// Converts `module.exports`-based CJS code to ESM by prepending
/// `module`/`exports` declarations and appending `export default`.
fn wrap_cjs_for_esm(source: &str) -> String {
    if has_esm_exports(source) {
        return source.to_string();
    }

    let mut wrapped = String::with_capacity(source.len() + 256);
    wrapped.push_str("var module = { exports: {} };\n");
    wrapped.push_str("var exports = module.exports;\n");
    wrapped.push_str(source);
    wrapped.push_str("\nexport default module.exports;\n");
    wrapped.push_str("globalThis.__koss_esm_result = module.exports;\n");

    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_module_path_returns_canonical_file() {
        let root = std::env::temp_dir().join(format!(
            "kossjs-module-loader-{}",
            std::process::id()
        ));
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        let module = nested.join("module.js");
        std::fs::write(&module, "export default 1;").unwrap();

        let verified = verify_module_path(&root, &nested.join(".").join("module.js")).unwrap();
        assert_eq!(verified, module.canonicalize().unwrap());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn verified_module_path_rejects_missing_file() {
        let root = std::env::temp_dir();
        let missing = root.join(format!(
            "kossjs-missing-module-{}-{}.js",
            std::process::id(),
            u64::MAX
        ));

        assert!(verify_module_path(&root, &missing).is_err());
    }
}
