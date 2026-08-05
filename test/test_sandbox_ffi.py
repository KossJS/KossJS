"""FFI 沙箱测试 - 验证 FFI 能力位对 _senri_ffi API 的控制

测试使用 test-lib 目录下的动态库（senri_test.dll / libsenri_test.so）。
"""
import os
import pytest
from kossjs_interface import KossJS, JsError # pyright: ignore[reportUnusedImport]

# 动态库路径
TEST_LIB_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "test-lib", "target", "release")
if os.name == "nt":
    TEST_LIB_PATH = os.path.join(TEST_LIB_DIR, "senri_test.dll").replace("\\", "/")
else:
    TEST_LIB_PATH = os.path.join(TEST_LIB_DIR, "senri_test.so").replace("\\", "/")

# 跳过测试如果动态库不存在
pytestmark = pytest.mark.skipif(
    not os.path.exists(os.path.join(TEST_LIB_DIR, "senri_test.dll" if os.name == "nt" else "libsenri_test.so")),
    reason="Test library not built"
)


# ============================================================================
# 注册阶段测试：_senri_ffi 对象是否存在
# ============================================================================

def test_ffi_exists_with_all_ffi_caps():
    """KOSS_CAP_ALL_FFI 启用时，_senri_ffi 应该存在"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.open") == "function"
        assert js.eval("typeof _senri_ffi.struct") == "function"
        assert js.eval("typeof _senri_ffi.alloc") == "function"
        assert js.eval("typeof _senri_ffi.callback") == "function"
        assert js.eval("typeof _senri_ffi.types") == "object"
    finally:
        js.destroy()

def test_ffi_open_exists_with_ffi_open():
    """FFI_OPEN 启用时，_senri_ffi.open 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_OPEN | KossJS.MODULE_LOAD, stable=False)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.open") == "function"
    finally:
        js.destroy()

def test_ffi_struct_exists_with_ffi_struct():
    """FFI_STRUCT 启用时，_senri_ffi.struct 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_STRUCT | KossJS.MODULE_LOAD, stable=False)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.struct") == "function"
    finally:
        js.destroy()

def test_ffi_alloc_exists_with_ffi_alloc():
    """FFI_ALLOC 启用时，_senri_ffi.alloc 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_ALLOC | KossJS.MODULE_LOAD, stable=False)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.alloc") == "function"
    finally:
        js.destroy()

def test_ffi_callback_exists_with_ffi_callback():
    """FFI_CALLBACK 启用时，_senri_ffi.callback 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_CALLBACK | KossJS.MODULE_LOAD, stable=False)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.callback") == "function"
    finally:
        js.destroy()


# ============================================================================
# 功能测试：FFI 能力位正确控制 API
# ============================================================================

def test_ffi_open_loads_library():
    """FFI_OPEN 启用时，应该能加载动态库"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')
        assert js.eval("typeof lib") == "object"
        assert js.eval("typeof lib.func") == "function"
    finally:
        js.destroy()

def test_ffi_call_invokes_function():
    """FFI_CALL 启用时，应该能调用 C 函数"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')
        js.eval('var addFn = lib.func("add_int", _senri_ffi.types.int32, [_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        result = js.eval("addFn(3, 4)")
        assert result == "7"
    finally:
        js.destroy()

def test_ffi_alloc_creates_buffer():
    """FFI_ALLOC 启用时，应该能分配内存"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        js.eval('var buf = _senri_ffi.alloc(16);')
        assert js.eval("typeof buf") == "object"
    finally:
        js.destroy()

def test_ffi_struct_creates_type():
    """FFI_STRUCT 启用时，应该能创建结构体类型"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        js.eval('var Point = _senri_ffi.struct([_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        assert js.eval("typeof Point") == "function"
    finally:
        js.destroy()


# ============================================================================
# 审核回调测试
# ============================================================================

def test_ffi_open_audit_denial_blocks_operation_and_calls_callback():
    """FFI_OPEN 审核拒绝应在动态库加载前阻止 _senri_ffi.open。"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[tuple[str, list[str]]] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append((target, args))
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_OPEN)

        with pytest.raises(JsError):
            js.eval(f'_senri_ffi.open("{TEST_LIB_PATH}")')

        assert calls == [("ffi.open", [TEST_LIB_PATH])]
    finally:
        js.destroy()

def test_ffi_call_audit_denial_blocks_symbol_lookup_and_close():
    """Existing library handles must honor later FFI_CALL audit policy changes."""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[str] = []
    try:
        js.eval(f'var auditLib = _senri_ffi.open("{TEST_LIB_PATH}");')

        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_CALL)

        with pytest.raises(JsError):
            js.eval('auditLib.func("add_int", _senri_ffi.types.int32, [])')
        with pytest.raises(JsError):
            js.eval('auditLib.funcAsync("add_int", _senri_ffi.types.int32, [])')
        with pytest.raises(JsError):
            js.eval("auditLib.close()")
        with pytest.raises(JsError):
            js.eval("auditLib.closeAsync()")

        assert calls == ["ffi.func", "ffi.funcAsync", "ffi.close", "ffi.closeAsync"]
        js.set_audit_mask(0)
        js.eval("auditLib.close()")
    finally:
        js.destroy()

def test_ffi_alloc_audit_denial_blocks_existing_pointer_access():
    """Existing pointers must not bypass later FFI_ALLOC audit policy changes."""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[str] = []
    try:
        js.eval("var auditPtr = _senri_ffi.alloc(8); auditPtr.writeInt32(7);")

        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_ALLOC)

        for expression in [
            "auditPtr.readInt32()",
            "auditPtr.writeInt32(9)",
            "auditPtr.add(4)",
            "auditPtr.toBigInt()",
        ]:
            with pytest.raises(JsError):
                js.eval(expression)

        assert calls == [
            "ffi.pointer.readInt32",
            "ffi.pointer.writeInt32",
            "ffi.pointer.add",
            "ffi.pointer.toBigInt",
        ]
        js.set_audit_mask(0)
        js.eval("_senri_ffi.free(auditPtr)")
    finally:
        js.destroy()

def test_ffi_call_audit_denial_blocks_errno_operations():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_CALL)

        with pytest.raises(JsError):
            js.eval("_senri_ffi.errno()")
        with pytest.raises(JsError):
            js.eval("_senri_ffi.strerror(0)")

        assert calls == ["ffi.errno", "ffi.strerror"]
    finally:
        js.destroy()

def test_ffi_alloc_audit_denial_blocks_struct_instance_memory_operations():
    """Struct construction and field memory access require FFI_ALLOC authorization."""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[str] = []
    try:
        js.eval("var AuditPoint = _senri_ffi.struct([_senri_ffi.types.int32]); var auditPoint = AuditPoint({int32: 1});")

        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_ALLOC)

        with pytest.raises(JsError):
            js.eval("AuditPoint({int32: 1})")
        with pytest.raises(JsError):
            js.eval("auditPoint.int32")
        with pytest.raises(JsError):
            js.eval("auditPoint.int32 = 2")
        with pytest.raises(JsError):
            js.eval("auditPoint.toPointer()")

        assert calls == ["ffi.struct.new", "ffi.struct.get", "ffi.struct.set", "ffi.struct.toPointer"]
    finally:
        js.destroy()

def test_ffi_callback_audit_denial_suppresses_existing_trampoline():
    """A denied callback invocation returns zero to C without running JavaScript."""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    calls: list[str] = []
    try:
        js.eval(f'''
            var callbackLib = _senri_ffi.open("{TEST_LIB_PATH}");
            var values = _senri_ffi.alloc(12);
            values.writeInt32(0, 1);
            values.writeInt32(4, 9);
            values.writeInt32(8, 3);
            var callbackRan = false;
            var compareCallback = _senri_ffi.createCallback(
                _senri_ffi.types.int32,
                [_senri_ffi.types.pointer, _senri_ffi.types.pointer],
                function() {{ callbackRan = true; return 1; }}
            );
            var findMax = callbackLib.func(
                "find_max",
                _senri_ffi.types.int32,
                [_senri_ffi.types.pointer, _senri_ffi.types.int32,
                 _senri_ffi.types.pointer]
            );
        ''')

        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_CALLBACK)

        assert js.eval("findMax(values, 3, compareCallback)") == "1"
        assert js.eval("callbackRan") == "false"
        assert calls == ["ffi.callbackInvoke", "ffi.callbackInvoke"]

        js.set_audit_mask(0)
        js.eval("_senri_ffi.freeCallback(compareCallback); _senri_ffi.free(values); callbackLib.close();")
    finally:
        js.destroy()

def test_ffi_capability_gating_is_primary_security():
    """FFI 的安全性通过能力位门控实现

    没有 FFI 能力位时，_senri_ffi 对象不存在，无法调用任何 FFI 函数。
    这是 FFI 的主要安全机制，不需要审核回调。
    """
    # 没有 FFI 能力位时，_senri_ffi 不存在
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof _senri_ffi") == "undefined"
    finally:
        js.destroy()

    # 沙箱模式下，_senri_ffi 不存在
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        assert js.eval("typeof _senri_ffi") == "undefined"
    finally:
        js.destroy()


# ============================================================================
# 完整 FFI 工作流测试
# ============================================================================

def test_ffi_full_workflow_with_all_caps():
    """完整的 FFI 工作流测试：加载库、调用函数、使用结构体"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        # 加载库
        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')

        # 调用 add_int
        js.eval('var addFn = lib.func("add_int", _senri_ffi.types.int32, [_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        assert js.eval("addFn(10, 20)") == "30"

        # 调用 multiply_int
        js.eval('var mulFn = lib.func("multiply_int", _senri_ffi.types.int32, [_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        assert js.eval("mulFn(5, 6)") == "30"

        # 调用 negate_int
        js.eval('var negFn = lib.func("negate_int", _senri_ffi.types.int32, [_senri_ffi.types.int32]);')
        assert js.eval("negFn(42)") == "-42"

        # 调用 add_float
        js.eval('var addFloatFn = lib.func("add_float", _senri_ffi.types.float64, [_senri_ffi.types.float64, _senri_ffi.types.float64]);')
        result = js.eval("addFloatFn(1.5, 2.5)")
        assert result == "4"
    finally:
        js.destroy()

def test_ffi_struct_creation():
    """结构体类型创建测试"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD, stable=False)
    try:
        # 创建 Point 结构体类型
        js.eval('var Point = _senri_ffi.struct([_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        assert js.eval("typeof Point") == "function"
    finally:
        js.destroy()


# ============================================================================
# 能力位不足时的拒绝测试
# ============================================================================

def test_ffi_no_caps_means_no_ffi():
    """没有 FFI 能力位时，_senri_ffi 不应该存在"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof _senri_ffi")
        # 如果 _senri_ffi 存在但没有方法，也是可以接受的
        if result == "object":
            # 检查是否有方法
            open_type = js.eval("typeof _senri_ffi.open")
            assert open_type == "undefined", f"_senri_ffi.open should not exist without FFI caps"
        else:
            assert result == "undefined"
    finally:
        js.destroy()

def test_ffi_sandbox_mode_no_ffi():
    """沙箱模式下，FFI 不应该存在"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        result = js.eval("typeof _senri_ffi")
        if result == "object":
            open_type = js.eval("typeof _senri_ffi.open")
            assert open_type == "undefined", f"_senri_ffi.open should not exist in sandbox mode"
        else:
            assert result == "undefined"
    finally:
        js.destroy()
