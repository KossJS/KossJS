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

/// A BOA-compatible module loader that uses [ModuleResolver] for Node.js-style
/// module resolution (bare specifiers, node_modules lookup, extension completion).
pub struct KossModuleLoader {
    /// The base directory for resolving the initial entry point.
    root: PathBuf,
    /// The underlying resolver with caching.
    resolver: ModuleResolver,
    /// Cache of already-parsed modules keyed by their canonical path.
    module_map: GcRefCell<FxHashMap<PathBuf, Module>>,
}

impl KossModuleLoader {
    /// Create a new loader rooted at the given directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            resolver: ModuleResolver::new(),
            module_map: GcRefCell::default(),
        }
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
                // Security: verify the resolved path is within the root directory
                let canonical_root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
                let canonical_resolved = match resolved.canonicalize() {
                    Ok(p) => p,
                    Err(_) => {
                        // Resolved path doesn't exist — still verify it's within root
                        // by ensuring no `..` escapes above root
                        let normalized = crate::resolver::ModuleResolver::normalize_path_static(&resolved);
                        match normalized {
                            Some(p) => {
                                if !p.starts_with(&canonical_root) {
                                    return Err(JsError::from(
                                        JsNativeError::typ().with_message(format!(
                                            "module '{}' resolves outside root directory",
                                            spec,
                                        )),
                                    ));
                                }
                                resolved.clone()
                            }
                            None => {
                                return Err(JsError::from(
                                    JsNativeError::typ().with_message(format!(
                                        "module '{}' path traversal detected",
                                        spec,
                                    )),
                                ));
                            }
                        }
                    }
                };
                if canonical_resolved.starts_with(&canonical_root) {
                    std::fs::read(&resolved).map_err(|err| {
                        JsError::from(JsNativeError::typ().with_message(format!(
                            "cannot read module '{}': {}",
                            resolved.display(),
                            err
                        )))
                    })?
                } else {
                    return Err(JsError::from(
                        JsNativeError::typ().with_message(format!(
                            "module '{}' resolves outside root directory",
                            spec,
                        )),
                    ));
                }
            };
            let source = Source::from_bytes(&source_bytes);

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
