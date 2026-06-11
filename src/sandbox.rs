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

use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

/// Synchronous audit callback type for C ABI.
/// Returns true to allow, false to block.
pub type AuditCallback = unsafe extern "C" fn(
    target: *const c_char,
    args: *const *const c_char,
    argc: c_int,
    pwd: *const c_char,
    userdata: *mut c_void,
) -> bool;

/// 沙箱状态：集中管理能力、审核掩码和未来扩展字段
#[derive(Default)]
pub struct SandboxState {
    pub audit_mask: u32,
    /// Synchronous audit callback (called when audit_mask bit is set for an operation).
    pub sync_audit: Option<AuditCallback>,
    /// Userdata pointer passed to the audit callback.
    pub sync_userdata: *mut c_void,
}

/// 检查能力位是否设置
pub fn has_cap(caps: u32, required: u32) -> bool {
    caps & required == required
}

/// 审核决策结果
pub enum AuditDecision {
    /// 直接放行（能力位已设置且审核掩码未设置）
    Allow,
    /// 直接拒绝（能力位未设置）
    DenyCapability,
    /// 需要审核（审核掩码已设置）
    NeedAudit,
}

/// 检查是否需要审核
pub fn check_audit_decision(caps: u32, audit_mask: u32, required: u32) -> AuditDecision {
    // 第一道闸门：能力位检查
    if !has_cap(caps, required) {
        return AuditDecision::DenyCapability;
    }
    // 第二道闸门：审核掩码检查
    if !has_cap(audit_mask, required) {
        return AuditDecision::Allow;
    }
    // 需要审核
    AuditDecision::NeedAudit
}

/// 检查审核掩码是否设置（且能力位已授予）
pub fn needs_audit(caps: u32, audit_mask: u32, required: u32) -> bool {
    // 审核掩码只能审核已授予的能力
    has_cap(caps, required) && has_cap(audit_mask, required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_cap_single_bit() {
        assert!(has_cap(FS_READ, FS_READ));
        assert!(!has_cap(FS_WRITE, FS_READ));
    }

    #[test]
    fn test_has_cap_combination() {
        let caps = FS_READ | FS_WRITE | NET_TCP_CLIENT;
        assert!(has_cap(caps, FS_READ));
        assert!(has_cap(caps, FS_WRITE));
        assert!(has_cap(caps, NET_TCP_CLIENT));
        assert!(!has_cap(caps, FS_DELETE));
        assert!(!has_cap(caps, NET_TCP_SERVER));
    }

    #[test]
    fn test_has_cap_zero_required() {
        assert!(has_cap(0, 0));
        assert!(has_cap(KOSS_CAP_ALL, 0));
    }

    #[test]
    fn test_has_cap_all() {
        assert!(has_cap(KOSS_CAP_ALL, FS_READ));
        assert!(has_cap(KOSS_CAP_ALL, KOSS_CAP_ALL_FS));
        assert!(has_cap(KOSS_CAP_ALL, KOSS_CAP_ALL_NET));
    }

    #[test]
    fn test_has_cap_sandbox_empty() {
        assert!(!has_cap(KOSS_CAP_SANDBOX, FS_READ));
        assert!(!has_cap(KOSS_CAP_SANDBOX, NET_TCP_CLIENT));
    }

    #[test]
    fn test_needs_audit_both_set() {
        let caps = FS_READ | FS_WRITE;
        let audit = FS_READ;
        assert!(needs_audit(caps, audit, FS_READ));
    }

    #[test]
    fn test_needs_audit_cap_set_audit_not() {
        let caps = FS_READ | FS_WRITE;
        let audit = FS_WRITE;
        assert!(!needs_audit(caps, audit, FS_READ));
    }

    #[test]
    fn test_needs_audit_cap_not_set() {
        let caps = FS_READ;
        let audit = FS_READ | FS_WRITE;
        // FS_WRITE not in caps, so audit is irrelevant
        assert!(!needs_audit(caps, audit, FS_WRITE));
    }

    #[test]
    fn test_needs_audit_neither_set() {
        let caps = FS_READ;
        let audit = FS_READ;
        assert!(!needs_audit(caps, audit, NET_TCP_CLIENT));
    }

    #[test]
    fn test_needs_audit_composite() {
        let caps = KOSS_CAP_ALL_FS | KOSS_CAP_ALL_NET;
        let audit = KOSS_CAP_ALL_FS;
        assert!(needs_audit(caps, audit, FS_READ));
        assert!(needs_audit(caps, audit, KOSS_CAP_ALL_FS));
        // NET is in caps but not in audit
        assert!(!needs_audit(caps, audit, NET_TCP_CLIENT));
    }

    #[test]
    fn test_check_audit_decision_deny_capability() {
        let caps = FS_READ;
        let audit_mask = FS_READ | FS_WRITE;
        // FS_WRITE not in caps → DenyCapability
        match check_audit_decision(caps, audit_mask, FS_WRITE) {
            AuditDecision::DenyCapability => {}
            _ => panic!("expected DenyCapability"),
        }
    }

    #[test]
    fn test_check_audit_decision_allow_no_audit() {
        let caps = FS_READ | FS_WRITE;
        let audit_mask = FS_WRITE;
        // FS_READ in caps, not in audit_mask → Allow
        match check_audit_decision(caps, audit_mask, FS_READ) {
            AuditDecision::Allow => {}
            _ => panic!("expected Allow"),
        }
    }

    #[test]
    fn test_check_audit_decision_need_audit() {
        let caps = FS_READ | FS_WRITE;
        let audit_mask = FS_READ;
        // FS_READ in both caps and audit_mask → NeedAudit
        match check_audit_decision(caps, audit_mask, FS_READ) {
            AuditDecision::NeedAudit => {}
            _ => panic!("expected NeedAudit"),
        }
    }

    #[test]
    fn test_check_audit_decision_empty_caps() {
        let caps = KOSS_CAP_SANDBOX;
        let audit_mask = KOSS_CAP_ALL;
        // No caps at all → DenyCapability regardless of audit_mask
        match check_audit_decision(caps, audit_mask, FS_READ) {
            AuditDecision::DenyCapability => {}
            _ => panic!("expected DenyCapability"),
        }
    }

    #[test]
    fn test_check_audit_decision_zero_audit_mask() {
        let caps = KOSS_CAP_ALL;
        let audit_mask = 0;
        // All caps, no audit → Allow for everything
        match check_audit_decision(caps, audit_mask, FS_READ) {
            AuditDecision::Allow => {}
            _ => panic!("expected Allow"),
        }
        match check_audit_decision(caps, audit_mask, NET_TCP_CLIENT) {
            AuditDecision::Allow => {}
            _ => panic!("expected Allow"),
        }
    }
}
