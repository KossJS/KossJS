// 文件系统（6 个细粒度操作）
pub const FS_READ: u32 = 1 << 0;
pub const FS_WRITE: u32 = 1 << 1;
pub const FS_DELETE: u32 = 1 << 2;
pub const FS_MKDIR: u32 = 1 << 3;
pub const FS_RENAME: u32 = 1 << 4;
pub const FS_CHMOD: u32 = 1 << 5;

// 网络（5 个细粒度操作）
pub const NET_TCP_CLIENT: u32 = 1 << 6;
pub const NET_TCP_SERVER: u32 = 1 << 7;
pub const NET_UDP: u32 = 1 << 8;
pub const NET_DNS: u32 = 1 << 9;
pub const NET_FETCH: u32 = 1 << 10;

// 加密（4 个细粒度操作）
pub const CRYPTO_HASH: u32 = 1 << 11;
pub const CRYPTO_HMAC: u32 = 1 << 12;
pub const CRYPTO_RANDOM: u32 = 1 << 13;
pub const CRYPTO_PBKDF2: u32 = 1 << 14;

// 内置 FFI（5 个细粒度操作）
pub const FFI_OPEN: u32 = 1 << 15;
pub const FFI_CALL: u32 = 1 << 16;
pub const FFI_ALLOC: u32 = 1 << 17;
pub const FFI_CALLBACK: u32 = 1 << 18;
pub const FFI_STRUCT: u32 = 1 << 19;

// 其他模块（8 个操作）
pub const NATIVE_ADDON: u32 = 1 << 20;
pub const WASM: u32 = 1 << 21;
pub const SHARED_MEMORY: u32 = 1 << 22;
pub const HIGHRES_TIME: u32 = 1 << 23;
pub const SYSINFO: u32 = 1 << 24;
pub const MODULE_LOAD: u32 = 1 << 25;
pub const DYNAMIC_CODE: u32 = 1 << 26;
pub const DEBUG_CAP: u32 = 1 << 27;

// 组合常量
pub const KOSS_CAP_SANDBOX: u32 = 0;
pub const KOSS_CAP_ALL_FS: u32 = FS_READ | FS_WRITE | FS_DELETE | FS_MKDIR | FS_RENAME | FS_CHMOD;
pub const KOSS_CAP_ALL_NET: u32 = NET_TCP_CLIENT | NET_TCP_SERVER | NET_UDP | NET_DNS | NET_FETCH;
pub const KOSS_CAP_ALL_CRYPTO: u32 = CRYPTO_HASH | CRYPTO_HMAC | CRYPTO_RANDOM | CRYPTO_PBKDF2;
pub const KOSS_CAP_ALL_FFI: u32 = FFI_OPEN | FFI_CALL | FFI_ALLOC | FFI_CALLBACK | FFI_STRUCT;
pub const KOSS_CAP_ALL: u32 = 0xFFFFFFFF;

// 兼容别名（用于旧宿主代码过渡）
pub const KOSS_CAP_FS: u32 = KOSS_CAP_ALL_FS;
pub const KOSS_CAP_NET: u32 = KOSS_CAP_ALL_NET;
pub const KOSS_CAP_CRYPTO: u32 = KOSS_CAP_ALL_CRYPTO;
pub const KOSS_CAP_WORKER: u32 = 1 << 3;
pub const KOSS_CAP_EXTERNAL_LOADER: u32 = MODULE_LOAD;

/// 检查能力位是否设置
pub fn has_cap(caps: u32, required: u32) -> bool {
    caps & required == required
}

/// 检查审核掩码是否设置（且能力位已授予）
pub fn needs_audit(caps: u32, audit_mask: u32, required: u32) -> bool {
    // 审核掩码只能审核已授予的能力
    has_cap(caps, required) && has_cap(audit_mask, required)
}
