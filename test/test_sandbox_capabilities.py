import pytest
from kossjs_interface import KossJS, JsError


# ============================================================================
# 常量定义测试
# ============================================================================

def test_all_capability_constants_exist():
    """Verify all 28 capability constants are defined and have correct values."""
    assert KossJS.FS_READ == 1 << 0
    assert KossJS.FS_WRITE == 1 << 1
    assert KossJS.FS_DELETE == 1 << 2
    assert KossJS.FS_MKDIR == 1 << 3
    assert KossJS.FS_RENAME == 1 << 4
    assert KossJS.FS_CHMOD == 1 << 5
    assert KossJS.NET_TCP_CLIENT == 1 << 6
    assert KossJS.NET_TCP_SERVER == 1 << 7
    assert KossJS.NET_UDP == 1 << 8
    assert KossJS.NET_DNS == 1 << 9
    assert KossJS.NET_FETCH == 1 << 10
    assert KossJS.CRYPTO_HASH == 1 << 11
    assert KossJS.CRYPTO_HMAC == 1 << 12
    assert KossJS.CRYPTO_RANDOM == 1 << 13
    assert KossJS.CRYPTO_PBKDF2 == 1 << 14
    assert KossJS.FFI_OPEN == 1 << 15
    assert KossJS.FFI_CALL == 1 << 16
    assert KossJS.FFI_ALLOC == 1 << 17
    assert KossJS.FFI_CALLBACK == 1 << 18
    assert KossJS.FFI_STRUCT == 1 << 19
    assert KossJS.NATIVE_ADDON == 1 << 20
    assert KossJS.WASM == 1 << 21
    assert KossJS.SHARED_MEMORY == 1 << 22
    assert KossJS.HIGHRES_TIME == 1 << 23
    assert KossJS.SYSINFO == 1 << 24
    assert KossJS.MODULE_LOAD == 1 << 25
    assert KossJS.DYNAMIC_CODE == 1 << 26
    assert KossJS.DEBUG_CAP == 1 << 27

def test_combination_constants():
    """Verify combination constants are correct."""
    assert KossJS.KOSS_CAP_SANDBOX == 0
    assert KossJS.KOSS_CAP_ALL_FS == (
        KossJS.FS_READ | KossJS.FS_WRITE | KossJS.FS_DELETE |
        KossJS.FS_MKDIR | KossJS.FS_RENAME | KossJS.FS_CHMOD
    )
    assert KossJS.KOSS_CAP_ALL_NET == (
        KossJS.NET_TCP_CLIENT | KossJS.NET_TCP_SERVER | KossJS.NET_UDP |
        KossJS.NET_DNS | KossJS.NET_FETCH
    )
    assert KossJS.KOSS_CAP_ALL_CRYPTO == (
        KossJS.CRYPTO_HASH | KossJS.CRYPTO_HMAC |
        KossJS.CRYPTO_RANDOM | KossJS.CRYPTO_PBKDF2
    )
    assert KossJS.KOSS_CAP_ALL_FFI == (
        KossJS.FFI_OPEN | KossJS.FFI_CALL | KossJS.FFI_ALLOC |
        KossJS.FFI_CALLBACK | KossJS.FFI_STRUCT
    )
    assert KossJS.KOSS_CAP_ALL == 0xFFFFFFFF

def test_compatibility_aliases():
    """Verify backward-compatible aliases work."""
    assert KossJS.KOSS_CAP_FS == KossJS.KOSS_CAP_ALL_FS
    assert KossJS.KOSS_CAP_NET == KossJS.KOSS_CAP_ALL_NET
    assert KossJS.KOSS_CAP_CRYPTO == KossJS.KOSS_CAP_ALL_CRYPTO
    assert KossJS.KOSS_CAP_WORKER == 1 << 3
    assert KossJS.KOSS_CAP_EXTERNAL_LOADER == KossJS.MODULE_LOAD

def test_no_bit_overlap_between_categories():
    """Ensure no bit overlap between different capability categories."""
    fs_caps = [KossJS.FS_READ, KossJS.FS_WRITE, KossJS.FS_DELETE,
               KossJS.FS_MKDIR, KossJS.FS_RENAME, KossJS.FS_CHMOD]
    net_caps = [KossJS.NET_TCP_CLIENT, KossJS.NET_TCP_SERVER, KossJS.NET_UDP,
                KossJS.NET_DNS, KossJS.NET_FETCH]
    crypto_caps = [KossJS.CRYPTO_HASH, KossJS.CRYPTO_HMAC,
                   KossJS.CRYPTO_RANDOM, KossJS.CRYPTO_PBKDF2]
    ffi_caps = [KossJS.FFI_OPEN, KossJS.FFI_CALL, KossJS.FFI_ALLOC,
                KossJS.FFI_CALLBACK, KossJS.FFI_STRUCT]
    other_caps = [KossJS.NATIVE_ADDON, KossJS.WASM, KossJS.SHARED_MEMORY,
                  KossJS.HIGHRES_TIME, KossJS.SYSINFO, KossJS.MODULE_LOAD,
                  KossJS.DYNAMIC_CODE, KossJS.DEBUG_CAP]

    all_caps = fs_caps + net_caps + crypto_caps + ffi_caps + other_caps

    for i, cap in enumerate(all_caps):
        assert cap > 0, f"Capability {i} should be positive"
        assert (cap & (cap - 1)) == 0, f"Capability {cap} should be a power of 2"
        for j, other in enumerate(all_caps):
            if i != j:
                assert (cap & other) == 0, f"Capabilities {cap} and {other} overlap"


# ============================================================================
# 能力位注册阶段行为测试
# ============================================================================

def test_fetch_enabled_with_net_fetch():
    """NET_FETCH 启用时，fetch 应该存在"""
    js = KossJS(capabilities=KossJS.NET_FETCH | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof fetch")
        assert result == "function"
    finally:
        js.destroy()

def test_fetch_disabled_without_net_fetch():
    """NET_FETCH 禁用时，fetch 应该不存在"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof fetch")
        assert result == "undefined"
    finally:
        js.destroy()

def test_fetch_disabled_in_sandbox():
    """沙箱模式下，fetch 应该不存在"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        result = js.eval("typeof fetch")
        assert result == "undefined"
    finally:
        js.destroy()

def test_fetch_call_throws_without_capability():
    """没有 NET_FETCH 能力时，fetch 调用应该抛出 KossCapabilityError"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        # fetch 函数不存在，所以应该抛出 TypeError
        with pytest.raises(JsError):
            js.eval("fetch('http://example.com')")
    finally:
        js.destroy()

def test_fs_module_exists_with_fs_caps():
    """有文件系统能力时，fs 模块应该存在"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof require('fs')")
        assert result == "object"
    finally:
        js.destroy()

def test_fs_module_exists_in_sandbox():
    """沙箱模式下，fs 模块仍然存在（stub），但调用会失败"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        # fs 模块对象仍然存在（stub）
        result = js.eval("typeof require('fs')")
        assert result == "object"
    finally:
        js.destroy()

def test_all_net_capabilities():
    """KOSS_CAP_ALL_NET 应该启用 fetch"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof fetch") == "function"
    finally:
        js.destroy()

def test_all_fs_capabilities():
    """KOSS_CAP_ALL_FS 应该启用 fs 模块"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof require('fs')") == "object"
    finally:
        js.destroy()

def test_combined_capabilities():
    """组合多个能力位应该正确启用对应的 API"""
    caps = KossJS.FS_READ | KossJS.NET_FETCH | KossJS.MODULE_LOAD
    js = KossJS(capabilities=caps)
    try:
        assert js.eval("typeof require('fs')") == "object"
        assert js.eval("typeof fetch") == "function"
    finally:
        js.destroy()

def test_capability_is_static_at_creation():
    """能力位在实例创建时确定，运行时不可更改"""
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.MODULE_LOAD)
    try:
        # fetch 不应该存在（没有 NET_FETCH）
        assert js.eval("typeof fetch") == "undefined"
        # fs 模块应该存在（有 FS_READ）
        assert js.eval("typeof require('fs')") == "object"
    finally:
        js.destroy()

def test_fs_delete_capability():
    """FS_DELETE 能力位测试"""
    js = KossJS(capabilities=KossJS.FS_DELETE | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof require('fs').unlinkSync")
        assert result == "function"
    finally:
        js.destroy()

def test_fs_mkdir_capability():
    """FS_MKDIR 能力位测试"""
    js = KossJS(capabilities=KossJS.FS_MKDIR | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof require('fs').mkdirSync")
        assert result == "function"
    finally:
        js.destroy()

def test_fs_rename_capability():
    """FS_RENAME 能力位测试"""
    js = KossJS(capabilities=KossJS.FS_RENAME | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof require('fs').renameSync")
        assert result == "function"
    finally:
        js.destroy()

def test_fs_chmod_capability():
    """FS_CHMOD 能力位测试"""
    js = KossJS(capabilities=KossJS.FS_CHMOD | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof require('fs').chmodSync")
        assert result == "function"
    finally:
        js.destroy()
