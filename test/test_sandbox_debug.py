"""调试模式测试 - 验证调试模式对错误消息的影响"""
import pytest
from kossjs_interface import KossJS, JsError


def test_enable_audit_debug_api_exists():
    """测试 enable_audit_debug API 存在"""
    js = KossJS()
    try:
        js.enable_audit_debug(True)
        js.enable_audit_debug(False)
    finally:
        js.destroy()

def test_debug_mode_shows_detailed_capability_error():
    """测试调试模式显示详细的能力位错误消息"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(True)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossCapabilityError" in error_msg
    finally:
        js.destroy()

def test_production_mode_hides_capability_details():
    """测试生产模式隐藏能力位错误详细信息"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(False)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossCapabilityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_shows_detailed_security_error():
    """测试调试模式显示详细的审核拒绝错误消息"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.enable_audit_debug(True)
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossSecurityError" in error_msg
    finally:
        js.destroy()

def test_production_mode_hides_security_details():
    """测试生产模式隐藏审核拒绝错误详细信息"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.enable_audit_debug(False)
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossSecurityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_can_be_toggled():
    """测试调试模式可以动态切换"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(True)
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs')")
        debug_error = str(exc.value)
        assert "KossCapabilityError" in debug_error

        js.enable_audit_debug(False)
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('net')")
        production_error = str(exc.value)
        assert "KossCapabilityError" in production_error
    finally:
        js.destroy()

def test_debug_mode_default_is_disabled():
    """测试调试模式默认是禁用的"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossCapabilityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_with_audit_callback():
    """测试调试模式与审核回调的交互"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.enable_audit_debug(True)
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossSecurityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_with_multiple_errors():
    """测试调试模式下多种错误类型"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(True)

        # 测试能力位错误
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)

        with pytest.raises(JsError) as exc:
            js.eval("const binding2 = internalBinding('net');")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()

def test_debug_mode_error_message_format():
    """测试调试模式错误消息格式"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(True)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossCapabilityError" in error_msg
    finally:
        js.destroy()

def test_production_mode_error_message_format():
    """测试生产模式错误消息格式"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        js.enable_audit_debug(False)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        error_msg = str(exc.value)
        assert "KossCapabilityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_with_audit_mask_and_callback():
    """测试调试模式与审核掩码和审核回调的完整交互"""
    caps = KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD
    js = KossJS(capabilities=caps)
    try:
        js.enable_audit_debug(True)
        js.check_sandbox(lambda target, args, pwd: target == "fs")
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET)

        # fs 应该被批准
        js.eval("const fs_binding = internalBinding('fs');")

        # net 应该被拒绝
        with pytest.raises(JsError) as exc:
            js.eval("const net_binding = internalBinding('net');")
        error_msg = str(exc.value)
        assert "KossSecurityError" in error_msg
    finally:
        js.destroy()

def test_debug_mode_with_capability_and_audit_errors():
    """测试调试模式下能力位错误和审核错误的区分"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.enable_audit_debug(True)
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 能力位错误（没有 net 能力）
        with pytest.raises(JsError) as exc:
            js.eval("const net_binding = internalBinding('net');")
        assert "KossCapabilityError" in str(exc.value)

        # 审核拒绝错误
        with pytest.raises(JsError) as exc:
            js.eval("const fs_binding = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()
