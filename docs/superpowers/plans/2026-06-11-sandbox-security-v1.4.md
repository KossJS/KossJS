# KossJS 沙箱安全系统实现计划 (v1.4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将《安全与沙箱设计.md》v1.4 落地为 KossJS 的三层安全机制：能力位掩码、审核掩码、审核回调。

**Architecture:** 保持现有 `KossInstance` 单线程拥有 Boa `Context` 的模型。新增 `sandbox` 模块集中管理能力、审核掩码、审核状态、参数序列化、错误类型和 C ABI 数据结构。决策流程：能力位检查 → 审核掩码检查 → 审核回调。

**Tech Stack:** Rust 2024、Boa `0.21.1`、Tokio、C ABI、Python `ctypes`、pytest、Cargo test。

---

## 文件结构与职责

- Create: `src/sandbox.rs`
  定义细粒度 capability 常量、审核掩码、`SandboxState`、审核回调类型、异步配置、取消 token、参数序列化、统一决策函数、错误构造辅助函数。
- Modify: `src/lib.rs`
  导出 `sandbox` 模块。
- Modify: `src/runtime.rs`
  将 capability 常量迁移到 `sandbox`，在 `KossInstance` 增加 `sandbox: SandboxState`，在 C ABI 和 JS 原生函数入口调用 `sandbox` 检查。
- Modify: `include/kossjs.h`
  同步 C ABI：细粒度 capability、审核掩码 API、审核回调 typedef、配置 struct、注册/取消/调试 API。
- Modify: `kossjs_interface.py`
  同步常量、ctypes 原型、Python 侧审核掩码 API、同步审核封装、异步审核封装、取消、调试开关。
- Create: `test/test_sandbox_capabilities.py`
  覆盖细粒度 capability 的注册阶段和调用阶段行为。
- Create: `test/test_sandbox_audit_mask.py`
  覆盖审核掩码的设置、获取、与能力位的关系。
- Create: `test/test_sandbox_audit.py`
  覆盖同步审核、JS 审核、拒绝错误类型、参数转换、pwd 语义的基础行为。
- Create: `test/test_sandbox_async_audit.py`
  覆盖异步审核、超时、并发、取消、重入限制。

---

## Phase 1: 建立细粒度 Capability 基础

### Task 1: 新增 `sandbox` 模块和 28 个 capability 常量

**Files:**
- Create: `src/sandbox.rs`
- Modify: `src/lib.rs`
- Modify: `src/runtime.rs`
- Modify: `include/kossjs.h`
- Modify: `kossjs_interface.py`
- Test: `test/test_sandbox_capabilities.py`

- [ ] **Step 1: 创建 `src/sandbox.rs` 常量定义**

```rust
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
```

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

- [ ] **Step 2: 导出模块**

在 `src/lib.rs` 添加：
```rust
pub mod sandbox;
```

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

- [ ] **Step 3: 更新 `include/kossjs.h`**

添加细粒度 capability 定义：
```c
typedef enum {
    FS_READ         = 1u << 0,
    FS_WRITE        = 1u << 1,
    FS_DELETE       = 1u << 2,
    FS_MKDIR        = 1u << 3,
    FS_RENAME       = 1u << 4,
    FS_CHMOD        = 1u << 5,
    NET_TCP_CLIENT  = 1u << 6,
    NET_TCP_SERVER  = 1u << 7,
    NET_UDP         = 1u << 8,
    NET_DNS         = 1u << 9,
    NET_FETCH       = 1u << 10,
    CRYPTO_HASH     = 1u << 11,
    CRYPTO_HMAC     = 1u << 12,
    CRYPTO_RANDOM   = 1u << 13,
    CRYPTO_PBKDF2   = 1u << 14,
    FFI_OPEN        = 1u << 15,
    FFI_CALL        = 1u << 16,
    FFI_ALLOC       = 1u << 17,
    FFI_CALLBACK    = 1u << 18,
    FFI_STRUCT      = 1u << 19,
    NATIVE_ADDON    = 1u << 20,
    WASM            = 1u << 21,
    SHARED_MEMORY   = 1u << 22,
    HIGHRES_TIME    = 1u << 23,
    SYSINFO         = 1u << 24,
    MODULE_LOAD     = 1u << 25,
    DYNAMIC_CODE    = 1u << 26,
    DEBUG_CAP       = 1u << 27
} KossCapability;
```

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

- [ ] **Step 4: 更新 `kossjs_interface.py` 常量**

```python
FS_READ = 1 << 0
FS_WRITE = 1 << 1
FS_DELETE = 1 << 2
FS_MKDIR = 1 << 3
FS_RENAME = 1 << 4
FS_CHMOD = 1 << 5
NET_TCP_CLIENT = 1 << 6
NET_TCP_SERVER = 1 << 7
NET_UDP = 1 << 8
NET_DNS = 1 << 9
NET_FETCH = 1 << 10
CRYPTO_HASH = 1 << 11
CRYPTO_HMAC = 1 << 12
CRYPTO_RANDOM = 1 << 13
CRYPTO_PBKDF2 = 1 << 14
FFI_OPEN = 1 << 15
FFI_CALL = 1 << 16
FFI_ALLOC = 1 << 17
FFI_CALLBACK = 1 << 18
FFI_STRUCT = 1 << 19
NATIVE_ADDON = 1 << 20
WASM = 1 << 21
SHARED_MEMORY = 1 << 22
HIGHRES_TIME = 1 << 23
SYSINFO = 1 << 24
MODULE_LOAD = 1 << 25
DYNAMIC_CODE = 1 << 26
DEBUG_CAP = 1 << 27
KOSS_CAP_SANDBOX = 0
KOSS_CAP_ALL_FS = FS_READ | FS_WRITE | FS_DELETE | FS_MKDIR | FS_RENAME | FS_CHMOD
KOSS_CAP_ALL_NET = NET_TCP_CLIENT | NET_TCP_SERVER | NET_UDP | NET_DNS | NET_FETCH
KOSS_CAP_ALL_CRYPTO = CRYPTO_HASH | CRYPTO_HMAC | CRYPTO_RANDOM | CRYPTO_PBKDF2
KOSS_CAP_ALL_FFI = FFI_OPEN | FFI_CALL | FFI_ALLOC | FFI_CALLBACK | FFI_STRUCT
KOSS_CAP_ALL = 0xFFFFFFFF
```

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

- [ ] **Step 5: 写测试**

在 `test/test_sandbox_capabilities.py` 添加：
```python
import pytest
from kossjs_interface import KossJS, JsError

def test_sandbox_has_no_fs_read_by_default_when_caps_zero():
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError):
            js.eval("require('fs').readFileSync('README.md', 'utf8')")
    finally:
        js.destroy()

def test_fs_read_allows_read_but_not_write():
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof require('fs').readFileSync") == "function"
        with pytest.raises(JsError):
            js.eval("require('fs').writeFileSync('tmp_sandbox_denied.txt', 'x')")
    finally:
        js.destroy()

def test_capabilities_are_static():
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        # 能力位在实例创建时确定，运行时不可更改
        assert js.eval("typeof require('fs').readFileSync") == "function"
    finally:
        js.destroy()
```

- [ ] **Step 6: 运行测试**

Run: `python -m pytest test/test_sandbox_capabilities.py -v`
Expected: PASS

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

- [ ] **Step 7: Commit**

```bash
git add src/sandbox.rs src/lib.rs include/kossjs.h kossjs_interface.py test/test_sandbox_capabilities.py
git commit -m "feat: add fine-grained sandbox capabilities"
```

注意：能力位掩码是静态权限声明，在实例创建时确定，运行时不可更改。

---

## Phase 2: 实现审核掩码

### Task 2: 添加审核掩码 API

**Files:**
- Modify: `src/sandbox.rs`
- Modify: `src/runtime.rs`
- Modify: `include/kossjs.h`
- Modify: `kossjs_interface.py`
- Test: `test/test_sandbox_audit_mask.py`

- [ ] **Step 1: 在 `SandboxState` 中添加审核掩码字段**

在 `src/sandbox.rs` 添加：
```rust
#[derive(Default)]
pub struct SandboxState {
    pub capabilities: u32,
    pub audit_mask: u32,
    pub sync_audit: Option<AuditCallback>,
    pub sync_userdata: *mut c_void,
    pub js_audit_callback: Option<JsFunction>,
    pub audit_debug: bool,
    pub argument_redaction: bool,
    pub reentrant_depth: u32,
    pub max_reentrant_depth: u32,
    pub allow_reentrant: bool,
}
```

注意：审核掩码只能审核已授予的能力位，忽略未授予的位（不会报错）。

- [ ] **Step 2: 实现 `koss_set_audit_mask` 和 `koss_get_audit_mask` C ABI**

在 `src/runtime.rs` 添加：
```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_set_audit_mask(ptr: *mut KossInstance, mask: u32) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let instance = &mut *ptr;
        // 审核掩码只能审核已授予的能力，忽略未授予的位（不会报错）
        instance.sandbox.audit_mask = mask & instance.capabilities;
        KossResult::ok("ok")
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_get_audit_mask(ptr: *mut KossInstance) -> u32 {
    unsafe {
        if ptr.is_null() {
            return 0;
        }
        (*ptr).sandbox.audit_mask
    }
}
```

注意：审核掩码只能设置在能力位掩码已授予的位上。若审核掩码包含了能力位掩码未授予的位，这些位将被视为无效（忽略，不会报错），因为对应的 API 已被禁用，无需审核。

- [ ] **Step 3: 更新 `include/kossjs.h`**

```c
KossResult koss_set_audit_mask(KossInstance* ptr, uint32_t mask);
uint32_t koss_get_audit_mask(KossInstance* ptr);
```

- [ ] **Step 4: 更新 `kossjs_interface.py`**

```python
lib.koss_set_audit_mask.restype = KossResult
lib.koss_set_audit_mask.argtypes = [ctypes.c_void_p, ctypes.c_uint32]

lib.koss_get_audit_mask.restype = ctypes.c_uint32
lib.koss_get_audit_mask.argtypes = [ctypes.c_void_p]

def set_audit_mask(self, mask: int) -> None:
    """设置审核掩码（只能审核已授予的能力位）"""
    result = self._lib.koss_set_audit_mask(self._ptr, mask)
    self._check_result(result)

def get_audit_mask(self) -> int:
    """获取当前审核掩码"""
    return self._lib.koss_get_audit_mask(self._ptr)
```

注意：审核掩码只能设置在能力位掩码已授予的位上。若审核掩码包含了能力位掩码未授予的位，这些位将被视为无效（忽略，不会报错），因为对应的 API 已被禁用，无需审核。

- [ ] **Step 5: 写测试**

在 `test/test_sandbox_audit_mask.py` 添加：
```python
import pytest
from kossjs_interface import KossJS

def test_default_audit_mask_is_zero():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()

def test_set_audit_mask():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        assert js.get_audit_mask() == (KossJS.FS_READ | KossJS.NET_FETCH)
    finally:
        js.destroy()

def test_audit_mask_ignores_ungranted_capabilities():
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        # 尝试设置未授予的能力位，应该被忽略
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        mask = js.get_audit_mask()
        assert mask == KossJS.FS_READ  # NET_FETCH 被忽略
    finally:
        js.destroy()

def test_audit_mask_zero_disables_audit():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(0)
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()
```

- [ ] **Step 6: 运行测试**

Run: `python -m pytest test/test_sandbox_audit_mask.py -v`
Expected: PASS

注意：审核掩码只能设置在能力位掩码已授予的位上。若审核掩码包含了能力位掩码未授予的位，这些位将被视为无效（忽略，不会报错），因为对应的 API 已被禁用，无需审核。

- [ ] **Step 7: Commit**

```bash
git add src/sandbox.rs src/runtime.rs include/kossjs.h kossjs_interface.py test/test_sandbox_audit_mask.py
git commit -m "feat: add audit mask API"
```

注意：审核掩码只能设置在能力位掩码已授予的位上。若审核掩码包含了能力位掩码未授予的位，这些位将被视为无效（忽略，不会报错），因为对应的 API 已被禁用，无需审核。

---

## Phase 3: 实现同步审核回调

### Task 3: 添加同步审核回调 C ABI

**Files:**
- Modify: `src/sandbox.rs`
- Modify: `src/runtime.rs`
- Modify: `include/kossjs.h`
- Modify: `kossjs_interface.py`
- Test: `test/test_sandbox_audit.py`

注意：审核回调可以随时注册（但通常在实例创建后、执行代码前注册）。

- [ ] **Step 1: 定义审核回调类型**

在 `src/sandbox.rs` 添加：
```rust
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

pub type AuditCallback = unsafe extern "C" fn(
    target: *const c_char,
    args: *const *const c_char,
    argc: c_int,
    pwd: *const c_char,
    userdata: *mut c_void,
) -> bool;
```

- [ ] **Step 2: 实现 `koss_check_sandbox` C ABI**

在 `src/runtime.rs` 添加：
```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_check_sandbox(
    ptr: *mut KossInstance,
    callback: crate::sandbox::AuditCallback,
    userdata: *mut c_void,
) -> KossResult {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return KossResult::err(2, "null pointer");
        }
        let instance = &mut *ptr;
        if callback as usize == 0 {
            instance.sandbox.sync_audit = None;
            instance.sandbox.sync_userdata = std::ptr::null_mut();
            return KossResult::ok("audit callback cleared");
        }
        instance.sandbox.sync_audit = Some(callback);
        instance.sandbox.sync_userdata = userdata;
        KossResult::ok("ok")
    }
}
```

- [ ] **Step 3: 更新 `include/kossjs.h`**

```c
typedef bool (*AuditCallback)(const char* target, const char** args, int argc, const char* pwd, void* userdata);
KossResult koss_check_sandbox(KossInstance* ptr, AuditCallback func, void* userdata);
```

- [ ] **Step 4: 更新 `kossjs_interface.py`**

```python
self._AUDIT_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_bool,
    ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_char_p),
    ctypes.c_int,
    ctypes.c_char_p,
    ctypes.c_void_p,
)
lib.koss_check_sandbox.restype = KossResult
lib.koss_check_sandbox.argtypes = [ctypes.c_void_p, self._AUDIT_CALLBACK, ctypes.c_void_p]

def check_sandbox(self, func=None):
    if func is None:
        null_cb = self._AUDIT_CALLBACK(0)
        self._check_result(self._lib.koss_check_sandbox(self._ptr, null_cb, None))
        self._audit_callback = None
        return

    def wrapper(target, args, argc, pwd, userdata):
        target_s = target.decode("utf-8", errors="replace") if target else ""
        values = []
        for i in range(argc):
            raw = args[i]
            values.append(raw.decode("utf-8", errors="replace") if raw else "")
        pwd_s = pwd.decode("utf-8", errors="replace") if pwd else None
        try:
            return bool(func(target_s, values, pwd_s))
        except Exception:
            return False

    cb = self._AUDIT_CALLBACK(wrapper)
    self._audit_callback = cb
    self._check_result(self._lib.koss_check_sandbox(self._ptr, cb, None))
```

注意：审核回调可以随时注册（但通常在实例创建后、执行代码前注册）。当 KossInstance 被 `koss_destroy` 销毁时，所有关联的审核回调将被自动释放，无需手动清除。

- [ ] **Step 5: 写测试**

在 `test/test_sandbox_audit.py` 添加：
```python
import pytest
from kossjs_interface import KossJS, JsError

def test_sync_audit_allows_safe_read_path():
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    calls = []
    try:
        def audit(target, args, pwd):
            calls.append((target, args, pwd))
            if target == "fs.readFileSync":
                return args[0].endswith("README.md")
            return True
        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FS_READ)
        js.eval("require('fs').readFileSync('README.md', 'utf8').length > 0")
        assert any(call[0] == "fs.readFileSync" for call in calls)
    finally:
        js.destroy()

def test_sync_audit_rejects_path():
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        js.check_sandbox(lambda target, args, pwd: False if target == "fs.readFileSync" else True)
        js.set_audit_mask(KossJS.FS_READ)
        with pytest.raises(JsError) as exc:
            js.eval("require('fs').readFileSync('README.md', 'utf8')")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_no_audit_when_mask_not_set():
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        # 不设置审核掩码，应该直接放行
        js.check_sandbox(lambda target, args, pwd: False)
        # 不调用 js.set_audit_mask(...)
        result = js.eval("require('fs').readFileSync('README.md', 'utf8').length > 0")
        assert result == "true"
    finally:
        js.destroy()
```

- [ ] **Step 6: 运行测试**

Run: `python -m pytest test/test_sandbox_audit.py -v`
Expected: PASS

注意：审核回调可以随时注册（但通常在实例创建后、执行代码前注册）。当 KossInstance 被 `koss_destroy` 销毁时，所有关联的审核回调将被自动释放，无需手动清除。

- [ ] **Step 7: Commit**

```bash
git add src/sandbox.rs src/runtime.rs include/kossjs.h kossjs_interface.py test/test_sandbox_audit.py
git commit -m "feat: add synchronous sandbox audit callback"
```

注意：审核回调可以随时注册（但通常在实例创建后、执行代码前注册）。当 KossInstance 被 `koss_destroy` 销毁时，所有关联的审核回调将被自动释放，无需手动清除。

---

## Phase 4: 实现审核决策函数

### Task 4: 实现统一的审核决策函数

**Files:**
- Modify: `src/sandbox.rs`
- Modify: `src/runtime.rs`

决策流程（v1.4）：
```
JS 调用受保护 API
    │
    ▼
┌─────────────────────────────────┐
│ 能力位掩码检查                    │
└─────────────────────────────────┘
    │
    ├── 未设置 → 直接拒绝（KossCapabilityError）
    │
    ▼ 已设置
┌─────────────────────────────────┐
│ 审核掩码检查                      │
└─────────────────────────────────┘
    │
    ├── 未设置 → 直接放行
    │
    ▼ 已设置
┌─────────────────────────────────┐
│ 是否存在外部审核回调？            │
└─────────────────────────────────┘
    │
    ├── 无 → 进入 JS 审核
    │
    ▼ 有
┌─────────────────────────────────┐
│ 根据注册类型调用外部审核回调       │
│ （同步或异步）                    │
└─────────────────────────────────┘
    │
    ├── 同步回调返回 false / 异常 → 拒绝
    ├── 同步回调返回 true → 进入 JS 审核
    ├── 异步回调 → 等待完成
    │      ├── 超时/取消/返回 false → 拒绝
    │      └── 返回 true → 进入 JS 审核
    │
    ▼
┌─────────────────────────────────┐
│ 是否存在 JS 审核回调？            │
└─────────────────────────────────┘
    │
    ├── 无 → 放行
    │
    ▼ 有
┌─────────────────────────────────┐
│ 调用 JS 审核回调                 │
│ （同步或异步，自动识别）           │
└─────────────────────────────────┘
    │
    ├── 同步返回 false / 异常 → 拒绝
    ├── 同步返回 true → 放行
    ├── 异步（Promise）→ 等待解决
    │      ├── 解决 false / 拒绝 → 拒绝
    │      └── 解决 true → 放行
    │
    ▼
    放行，执行实际 API 胡用
```

- [ ] **Step 1: 实现审核决策函数**

在 `src/sandbox.rs` 添加：
```rust
/// 审核决策结果
pub enum AuditDecision {
    /// 直接放行（能力位未设置或审核掩码未设置）
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
```

- [ ] **Step 2: 在 API 调用处使用审核决策函数**

在 `src/runtime.rs` 的 API 调用处添加：
```rust
use crate::sandbox::{check_audit_decision, AuditDecision};

// 示例：在 fs.readFile 调用处
let decision = check_audit_decision(inst.capabilities, inst.sandbox.audit_mask, crate::sandbox::FS_READ);
match decision {
    AuditDecision::DenyCapability => {
        return Err(throw_capability_error("fs.readFile"));
    }
    AuditDecision::Allow => {
        // 直接执行
    }
    AuditDecision::NeedAudit => {
        // 执行审核回调
        if !run_sync_audit(inst, "fs.readFile", &args) {
            return Err(throw_security_error("fs.readFile"));
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/sandbox.rs src/runtime.rs
git commit -m "feat: implement audit decision function"
```

注意：决策流程（v1.4）：能力位检查 → 审核掩码检查 → 审核回调。

---

## Phase 5: 实现调试模式和错误消息优化

### Task 5: 添加调试模式和错误消息优化

**Files:**
- Modify: `src/runtime.rs`
- Modify: `include/kossjs.h`
- Modify: `kossjs_interface.py`

错误类型：
- `KossCapabilityError`：因能力位掩码禁止而拒绝
- `KossSecurityError`：因审核回调拒绝
- `KossTimeoutError`：异步审核超时导致拒绝
- `KossCancelError`：异步审核被取消导致拒绝

- [ ] **Step 1: 实现 `koss_enable_audit_debug` C ABI**

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn koss_enable_audit_debug(ptr: *mut KossInstance, enable: bool) {
    output_license_once();
    unsafe {
        if ptr.is_null() {
            return;
        }
        (*ptr).sandbox.audit_debug = enable;
    }
}
```

- [ ] **Step 2: 更新错误消息生成**

```rust
pub fn capability_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossCapabilityError: capability denied for {target}")
    } else {
        "KossCapabilityError: Access denied".to_string()
    }
}

pub fn security_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossSecurityError: sandbox audit denied for {target}")
    } else {
        "KossSecurityError: Access denied".to_string()
    }
}

pub fn timeout_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossTimeoutError: sandbox audit timed out for {target}")
    } else {
        "KossTimeoutError: Access denied".to_string()
    }
}

pub fn cancel_error_message(target: &str, debug: bool) -> String {
    if debug {
        format!("KossCancelError: sandbox audit cancelled for {target}")
    } else {
        "KossCancelError: Access denied".to_string()
    }
}
```

- [ ] **Step 3: 更新 `include/kossjs.h`**

```c
void koss_enable_audit_debug(KossInstance* ptr, bool enable);
```

注意：调试模式开启后，同步/异步回调中抛出的异常会输出到 stderr，审核被拒绝的原因会附带额外错误信息，异步审核的超时或挂起会记录警告，重入拒绝会记录当前深度和配置的最大深度。生产环境应关闭调试模式，避免信息泄露。

- [ ] **Step 4: 更新 `kossjs_interface.py`**

```python
lib.koss_enable_audit_debug.restype = None
lib.koss_enable_audit_debug.argtypes = [ctypes.c_void_p, ctypes.c_bool]

def enable_audit_debug(self, enable: bool) -> None:
    """启用或禁用审核调试模式"""
    self._lib.koss_enable_audit_debug(self._ptr, bool(enable))
```

注意：调试模式开启后，同步/异步回调中抛出的异常会输出到 stderr，审核被拒绝的原因会附带额外错误信息，异步审核的超时或挂起会记录警告，重入拒绝会记录当前深度和配置的最大深度。生产环境应关闭调试模式，避免信息泄露。

- [ ] **Step 5: Commit**

```bash
git add src/runtime.rs include/kossjs.h kossjs_interface.py
git commit -m "feat: add audit debug mode and error message optimization"
```

注意：调试模式开启后，同步/异步回调中抛出的异常会输出到 stderr，审核被拒绝的原因会附带额外错误信息，异步审核的超时或挂起会记录警告，重入拒绝会记录当前深度和配置的最大深度。生产环境应关闭调试模式，避免信息泄露。

---

## Phase 6: 全量验证

### Task 6: 全量验证和 ABI 对齐检查

**Files:**
- Inspect: `src/runtime.rs`
- Inspect: `src/sandbox.rs`
- Inspect: `include/kossjs.h`
- Inspect: `kossjs_interface.py`

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

- [ ] **Step 1: 检查 header 与 Rust 导出一致**

Run: `rg "pub unsafe extern \"C\" fn|pub extern \"C\" fn" src/runtime.rs`

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

- [ ] **Step 2: Rust 测试**

Run: `cargo test --workspace`
Expected: PASS

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

- [ ] **Step 3: Release build**

Run: `cargo build --release`
Expected: PASS

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

- [ ] **Step 4: Python 全量测试**

Run: `python -m pytest test/ -v`
Expected: PASS

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

- [ ] **Step 5: Commit 最终修正**

```bash
git add <changed-files>
git commit -m "test: complete sandbox security coverage"
```

注意：全量验证包括检查 header 与 Rust 导出一致、Rust 测试、Release build、Python 全量测试。

---

## 实施顺序建议

1. Phase 1: 建立细粒度 Capability 基础
2. Phase 2: 实现审核掩码
3. Phase 3: 实现同步审核回调
4. Phase 4: 实现审核决策函数
5. Phase 5: 实现调试模式和错误消息优化
6. Phase 6: 全量验证

注意：决策流程（v1.4）：能力位检查 → 审核掩码检查 → 审核回调。

## 已知实现取舍

- 异步审核第一版可阻塞等待 C callback completion，以保证语义正确
- 旧 5 位 capability 保留为兼容别名
- Worker 在 v1.4 文档中没有单独 capability 位，保留旧 `KOSS_CAP_WORKER` 兼容行为
- 审核掩码只能审核已授予的能力位，忽略未授予的位（不会报错）
- 审核回调可以随时注册，但通常在实例创建后、执行代码前注册
- 错误类型：KossCapabilityError（能力位禁止）、KossSecurityError（审核拒绝）、KossTimeoutError（异步审核超时）、KossCancelError（审核被取消）
- 决策流程（v1.4）：能力位检查 → 审核掩码检查 → 审核回调
