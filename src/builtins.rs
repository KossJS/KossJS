/// Builtin Flags — control which compatibility layers are visible to user code.
/// Independent from Capability bits (sandbox).
pub const KOSS_BUILTIN_NONE: u32 = 0;
pub const KOSS_BUILTIN_NODE: u32 = 1 << 0;
pub const KOSS_BUILTIN_BUN: u32 = 1 << 1;
pub const KOSS_BUILTIN_DENO: u32 = 1 << 2;
pub const KOSS_BUILTIN_KOSS: u32 = 1 << 3;
pub const KOSS_BUILTIN_ALL: u32 = 0xFFFFFFFF;

/// A registered builtin module entry.
pub struct BuiltinModule {
    pub name: &'static str,
    pub flag: u32,
    pub source_path: &'static str,
    pub is_internal: bool,
}

pub static BUILTIN_MODULES: &[BuiltinModule] = &[
    BuiltinModule {
        name: "node/fs",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/fs.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/path",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/path.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/events",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/events.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/buffer",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/buffer.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/assert",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/assert.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/util",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/util.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/url",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/url.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/querystring",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/querystring.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/os",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/os.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/timers",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/timers.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/stream",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/stream.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/crypto",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/crypto.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/zlib",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/zlib.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/net",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/net.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/http",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/http.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/https",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/https.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/dns",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/dns.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/tls",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/tls.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/dgram",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/dgram.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/string_decoder",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/string_decoder.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/constants",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/constants.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/process",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/process.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/perf_hooks",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/perf_hooks.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/trace_events",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/trace_events.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "node/diagnostics_channel",
        flag: KOSS_BUILTIN_NODE,
        source_path: "node_shim/diagnostics_channel.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "bun",
        flag: KOSS_BUILTIN_BUN,
        source_path: "bun_shim.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "deno",
        flag: KOSS_BUILTIN_DENO,
        source_path: "deno_shim.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "io",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/io.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "crypto",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/crypto.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "system",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/system.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "data",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/data.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "ffi",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/ffi.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "worker",
        flag: KOSS_BUILTIN_KOSS,
        source_path: "koss_shim/worker.js",
        is_internal: false,
    },
    BuiltinModule {
        name: "internal/fs",
        flag: KOSS_BUILTIN_NONE,
        source_path: "internal/fs.js",
        is_internal: true,
    },
    BuiltinModule {
        name: "internal/net",
        flag: KOSS_BUILTIN_NONE,
        source_path: "internal/net.js",
        is_internal: true,
    },
    BuiltinModule {
        name: "internal/crypto",
        flag: KOSS_BUILTIN_NONE,
        source_path: "internal/crypto.js",
        is_internal: true,
    },
    BuiltinModule {
        name: "internal/stream",
        flag: KOSS_BUILTIN_NONE,
        source_path: "internal/stream.js",
        is_internal: true,
    },
];

pub fn builtin_module_names() -> Vec<&'static str> {
    BUILTIN_MODULES.iter().map(|m| m.name).collect()
}

pub fn find_builtin(path: &str) -> Option<&'static BuiltinModule> {
    BUILTIN_MODULES.iter().find(|m| m.name == path)
}

pub fn is_koss_specifier(specifier: &str) -> bool {
    specifier.starts_with("koss:")
}

pub fn strip_koss_prefix(specifier: &str) -> &str {
    specifier.strip_prefix("koss:").unwrap_or(specifier)
}

pub fn builtin_disabled_error(specifier: &str, flag: u32, current_flags: u32) -> String {
    let flag_name = flag_to_name(flag);
    let current_names = flags_to_names(current_flags);
    format!(
        "KossBuiltinError: Cannot resolve module '{}' - Builtin flag {} is not enabled. \
         Current builtins: {} \
         To enable: pass builtins={} when creating instance.",
        specifier, flag_name, current_names, flag_name
    )
}

pub fn internal_module_error(path: &str) -> String {
    format!(
        "KossBuiltinError: Cannot import 'koss:internal/{}' - This is an internal module \
         and not accessible to user code. If you are a developer, ensure the import \
         originates from /js_shims/ directory.",
        path
    )
}

pub fn flag_to_name(flag: u32) -> &'static str {
    match flag {
        KOSS_BUILTIN_NODE => "KOSS_BUILTIN_NODE",
        KOSS_BUILTIN_BUN => "KOSS_BUILTIN_BUN",
        KOSS_BUILTIN_DENO => "KOSS_BUILTIN_DENO",
        KOSS_BUILTIN_KOSS => "KOSS_BUILTIN_KOSS",
        KOSS_BUILTIN_ALL => "KOSS_BUILTIN_ALL",
        _ => "UNKNOWN",
    }
}

pub fn flags_to_names(flags: u32) -> String {
    let mut names = Vec::new();
    if flags & KOSS_BUILTIN_NODE != 0 {
        names.push("KOSS_BUILTIN_NODE");
    }
    if flags & KOSS_BUILTIN_BUN != 0 {
        names.push("KOSS_BUILTIN_BUN");
    }
    if flags & KOSS_BUILTIN_DENO != 0 {
        names.push("KOSS_BUILTIN_DENO");
    }
    if flags & KOSS_BUILTIN_KOSS != 0 {
        names.push("KOSS_BUILTIN_KOSS");
    }
    if names.is_empty() {
        names.push("KOSS_BUILTIN_NONE");
    }
    names.join(" | ")
}

pub fn resolve_builtin_specifier(
    specifier: &str,
    builtins: u32,
) -> Result<(&'static str, bool), String> {
    let path = strip_koss_prefix(specifier);

    let module = find_builtin(path)
        .ok_or_else(|| format!("KossBuiltinError: Cannot resolve module '{}' - no such builtin module. Available: {}", specifier, builtin_module_names().join(", ")))?;

    if module.flag != KOSS_BUILTIN_NONE && builtins & module.flag == 0 {
        return Err(builtin_disabled_error(specifier, module.flag, builtins));
    }

    let source = crate::embedded_stdlib::get(module.source_path)
        .ok_or_else(|| format!("KossBuiltinError: Cannot find source for module '{}' at '{}'", specifier, module.source_path))?;

    Ok((source, module.is_internal))
}