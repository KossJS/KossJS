"""FFI 沙箱测试 - 验证 FFI 能力位对 _senri_ffi API 的控制

测试使用 test-lib 目录下的动态库（senri_test.dll / libsenri_test.so）。
"""
import os
import pytest
from kossjs_interface import KossJS, JsError

# 动态库路径
TEST_LIB_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "test-lib", "target", "release")
if os.name == "nt":
    TEST_LIB_PATH = os.path.join(TEST_LIB_DIR, "senri_test.dll").replace("\\", "/")
else:
    TEST_LIB_PATH = os.path.join(TEST_LIB_DIR, "libsenri_test.so").replace("\\", "/")

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
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
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
    js = KossJS(capabilities=KossJS.FFI_OPEN | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.open") == "function"
    finally:
        js.destroy()

def test_ffi_struct_exists_with_ffi_struct():
    """FFI_STRUCT 启用时，_senri_ffi.struct 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_STRUCT | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.struct") == "function"
    finally:
        js.destroy()

def test_ffi_alloc_exists_with_ffi_alloc():
    """FFI_ALLOC 启用时，_senri_ffi.alloc 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_ALLOC | KossJS.MODULE_LOAD)
    try:
        assert js.eval("typeof _senri_ffi") == "object"
        assert js.eval("typeof _senri_ffi.alloc") == "function"
    finally:
        js.destroy()

def test_ffi_callback_exists_with_ffi_callback():
    """FFI_CALLBACK 启用时，_senri_ffi.callback 应该存在"""
    js = KossJS(capabilities=KossJS.FFI_CALLBACK | KossJS.MODULE_LOAD)
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
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
    try:
        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')
        assert js.eval("typeof lib") == "object"
        assert js.eval("typeof lib.func") == "function"
    finally:
        js.destroy()

def test_ffi_call_invokes_function():
    """FFI_CALL 启用时，应该能调用 C 函数"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
    try:
        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')
        js.eval('var addFn = lib.func("add_int", _senri_ffi.types.int32, [_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        result = js.eval("addFn(3, 4)")
        assert result == "7"
    finally:
        js.destroy()

def test_ffi_alloc_creates_buffer():
    """FFI_ALLOC 启用时，应该能分配内存"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
    try:
        js.eval('var buf = _senri_ffi.alloc(16);')
        assert js.eval("typeof buf") == "object"
    finally:
        js.destroy()

def test_ffi_struct_creates_type():
    """FFI_STRUCT 启用时，应该能创建结构体类型"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
    try:
        js.eval('var Point = _senri_ffi.struct([_senri_ffi.types.int32, _senri_ffi.types.int32]);')
        assert js.eval("typeof Point") == "function"
    finally:
        js.destroy()


# ============================================================================
# 审核回调测试：FFI 函数不经过 __koss_bindings，审核回调不会被触发
# 根据设计文档，FFI 函数直接注册为原生函数，审核通过能力位门控实现
# ============================================================================

def test_ffi_no_audit_through_internalbinding():
    """FFI 函数不经过 internalBinding 路径，审核回调不会被触发

    FFI 函数（如 _senri_ffi.open）直接注册为原生函数，
    不通过 __koss_bindings 路径，因此审核回调机制不适用于 FFI。
    审核通过能力位门控实现：没有 FFI_OPEN 能力位则 _senri_ffi.open 不存在。
    """
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
    calls = []
    try:
        def audit(target, args, pwd):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.FFI_OPEN)

        js.eval(f'var lib = _senri_ffi.open("{TEST_LIB_PATH}");')
        # FFI 不经过 internalBinding，审核回调不会被触发
        # 这是预期行为：FFI 通过能力位门控，不通过审核回调
        assert len(calls) == 0, f"FFI should not trigger audit callback through internalBinding: {calls}"
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
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
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
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FFI | KossJS.MODULE_LOAD)
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
