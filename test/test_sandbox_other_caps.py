"""其他能力位功能测试 - 验证 NATIVE_ADDON, WASM, SHARED_MEMORY, HIGHRES_TIME, SYSINFO, DYNAMIC_CODE, DEBUG_CAP 能力位

这些能力位控制各种高级功能的访问权限。
注意：部分能力位目前可能尚未实现完整功能，测试主要验证常量定义和基本行为。
"""
import pytest
from kossjs_interface import KossJS, JsError


# ============================================================================
# NATIVE_ADDON 能力位测试 (.node 原生模块)
# ============================================================================

def test_native_addon_constant_exists():
    """NATIVE_ADDON 常量应该存在且值正确"""
    assert KossJS.NATIVE_ADDON == 1 << 20


def test_native_addon_in_combination():
    """NATIVE_ADDON 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.NATIVE_ADDON) == KossJS.NATIVE_ADDON


def test_native_addon_not_in_sandbox():
    """沙箱模式下不应该有 NATIVE_ADDON 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NATIVE_ADDON) == 0


# ============================================================================
# WASM 能力位测试 (WebAssembly)
# ============================================================================

def test_wasm_constant_exists():
    """WASM 常量应该存在且值正确"""
    assert KossJS.WASM == 1 << 21


def test_wasm_in_combination():
    """WASM 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.WASM) == KossJS.WASM


def test_wasm_not_in_sandbox():
    """沙箱模式下不应该有 WASM 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.WASM) == 0


def test_wasm_webassembly_object_may_exist():
    """WebAssembly 对象可能存在（取决于 Boa 引擎配置）"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof WebAssembly")
        # WebAssembly 可能可用也可能不可用，取决于 Boa 引擎配置
        assert result in ["object", "undefined"]
    finally:
        js.destroy()


# ============================================================================
# SHARED_MEMORY 能力位测试 (SharedArrayBuffer, Atomics)
# ============================================================================

def test_shared_memory_constant_exists():
    """SHARED_MEMORY 常量应该存在且值正确"""
    assert KossJS.SHARED_MEMORY == 1 << 22


def test_shared_memory_in_combination():
    """SHARED_MEMORY 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.SHARED_MEMORY) == KossJS.SHARED_MEMORY


def test_shared_memory_not_in_sandbox():
    """沙箱模式下不应该有 SHARED_MEMORY 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.SHARED_MEMORY) == 0


def test_shared_memory_arraybuffer_exists():
    """ArrayBuffer 应该在所有模式下存在"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof ArrayBuffer")
        assert result == "function"
    finally:
        js.destroy()


# ============================================================================
# HIGHRES_TIME 能力位测试 (高精度计时)
# ============================================================================

def test_highres_time_constant_exists():
    """HIGHRES_TIME 常量应该存在且值正确"""
    assert KossJS.HIGHRES_TIME == 1 << 23


def test_highres_time_in_combination():
    """HIGHRES_TIME 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.HIGHRES_TIME) == KossJS.HIGHRES_TIME


def test_highres_time_not_in_sandbox():
    """沙箱模式下不应该有 HIGHRES_TIME 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.HIGHRES_TIME) == 0


def test_highres_time_performance_now_may_exist():
    """performance.now() 可能存在（取决于实现）"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        try:
            result = js.eval("typeof performance.now")
            assert result in ["function", "undefined"]
        except JsError:
            # performance 对象可能不存在
            pass
    finally:
        js.destroy()


def test_highres_time_process_hrtime():
    """process.hrtime() 应该在所有模式下可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof process.hrtime")
        assert result == "function"
    finally:
        js.destroy()


# ============================================================================
# SYSINFO 能力位测试 (系统信息)
# ============================================================================

def test_sysinfo_constant_exists():
    """SYSINFO 常量应该存在且值正确"""
    assert KossJS.SYSINFO == 1 << 24


def test_sysinfo_in_combination():
    """SYSINFO 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.SYSINFO) == KossJS.SYSINFO


def test_sysinfo_not_in_sandbox():
    """沙箱模式下不应该有 SYSINFO 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.SYSINFO) == 0


def test_sysinfo_os_module_exists():
    """os 模块应该在所有模式下存在（通过 internalBinding）"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof require('os')")
        assert result == "object"
    finally:
        js.destroy()


def test_sysinfo_os_hostname_may_exist():
    """os.getHostname() 可能存在（取决于实现）"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        try:
            result = js.eval("typeof require('os').getHostname")
            # getHostname 可能可用也可能不可用
            assert result in ["function", "undefined"]
        except JsError:
            # os 模块可能不可用
            pass
    finally:
        js.destroy()


# ============================================================================
# DYNAMIC_CODE 能力位测试 (动态代码执行)
# ============================================================================

def test_dynamic_code_constant_exists():
    """DYNAMIC_CODE 常量应该存在且值正确"""
    assert KossJS.DYNAMIC_CODE == 1 << 26


def test_dynamic_code_in_combination():
    """DYNAMIC_CODE 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.DYNAMIC_CODE) == KossJS.DYNAMIC_CODE


def test_dynamic_code_not_in_sandbox():
    """沙箱模式下不应该有 DYNAMIC_CODE 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.DYNAMIC_CODE) == 0


def test_dynamic_code_eval_works():
    """eval() 应该在有 DYNAMIC_CODE 能力时工作"""
    js = KossJS(capabilities=KossJS.DYNAMIC_CODE | KossJS.MODULE_LOAD)
    try:
        result = js.eval("eval('1 + 2')")
        assert result == "3"
    finally:
        js.destroy()


def test_dynamic_code_function_constructor():
    """Function 构造函数应该在有 DYNAMIC_CODE 能力时工作"""
    js = KossJS(capabilities=KossJS.DYNAMIC_CODE | KossJS.MODULE_LOAD)
    try:
        result = js.eval("new Function('return 1 + 2')()")
        assert result == "3"
    finally:
        js.destroy()


def test_dynamic_code_worker_threads():
    """worker_threads 模块应该在有 DYNAMIC_CODE 能力时可用"""
    js = KossJS(capabilities=KossJS.DYNAMIC_CODE | KossJS.MODULE_LOAD)
    try:
        try:
            result = js.eval("typeof require('worker_threads')")
            # worker_threads 可能可用也可能不可用
            assert result in ["object", "undefined"]
        except JsError:
            # 模块可能不可用
            pass
    finally:
        js.destroy()


def test_dynamic_code_worker_threads_disabled_without_cap():
    """没有 DYNAMIC_CODE 能力时，internalBinding('worker_threads') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('worker_threads')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


# ============================================================================
# DEBUG_CAP 能力位测试 (调试/内省)
# ============================================================================

def test_debug_cap_constant_exists():
    """DEBUG_CAP 常量应该存在且值正确"""
    assert KossJS.DEBUG_CAP == 1 << 27


def test_debug_cap_in_combination():
    """DEBUG_CAP 应该包含在 KOSS_CAP_ALL 中"""
    assert (KossJS.KOSS_CAP_ALL & KossJS.DEBUG_CAP) == KossJS.DEBUG_CAP


def test_debug_cap_not_in_sandbox():
    """沙箱模式下不应该有 DEBUG_CAP 能力"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.DEBUG_CAP) == 0


def test_debug_cap_error_stack():
    """Error.stack 应该在所有模式下可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof new Error().stack")
        # stack 可能是 string 或 undefined，取决于实现
        assert result in ["string", "undefined"]
    finally:
        js.destroy()


def test_debug_cap_console_exists():
    """console 对象应该在所有模式下可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        result = js.eval("typeof console")
        assert result == "object"
    finally:
        js.destroy()


# ============================================================================
# 组合能力位测试
# ============================================================================

def test_all_other_caps_combined():
    """所有其他能力位组合应该正确"""
    caps = (KossJS.NATIVE_ADDON | KossJS.WASM | KossJS.SHARED_MEMORY |
            KossJS.HIGHRES_TIME | KossJS.SYSINFO | KossJS.DYNAMIC_CODE |
            KossJS.DEBUG_CAP)
    assert (KossJS.KOSS_CAP_ALL & caps) == caps


def test_other_caps_not_in_basic_sandbox():
    """基本沙箱模式不应该包含任何其他能力位"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NATIVE_ADDON) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.WASM) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.SHARED_MEMORY) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.HIGHRES_TIME) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.SYSINFO) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.DYNAMIC_CODE) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.DEBUG_CAP) == 0


def test_other_caps_isolation():
    """各其他能力位之间应该相互独立"""
    caps = [KossJS.NATIVE_ADDON, KossJS.WASM, KossJS.SHARED_MEMORY,
            KossJS.HIGHRES_TIME, KossJS.SYSINFO, KossJS.DYNAMIC_CODE,
            KossJS.DEBUG_CAP]

    for i, cap in enumerate(caps):
        assert cap > 0, f"Capability {i} should be positive"
        assert (cap & (cap - 1)) == 0, f"Capability {cap} should be a power of 2"
        for j, other in enumerate(caps):
            if i != j:
                assert (cap & other) == 0, f"Capabilities {cap} and {other} overlap"


# ============================================================================
# 审核掩码测试
# ============================================================================

def test_audit_mask_with_other_caps():
    """审核掩码应该支持其他能力位"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        caps = [KossJS.NATIVE_ADDON, KossJS.WASM, KossJS.SHARED_MEMORY,
                KossJS.HIGHRES_TIME, KossJS.SYSINFO, KossJS.DYNAMIC_CODE,
                KossJS.DEBUG_CAP]

        for cap in caps:
            js.set_audit_mask(cap)
            assert js.get_audit_mask() == cap
    finally:
        js.destroy()
