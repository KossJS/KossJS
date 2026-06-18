"""审核决策流程测试 - 验证完整的决策流程：能力位 -> 审核掩码 -> 审核回调"""
import pytest
from kossjs_interface import KossJS, JsError


def test_capability_denial_throws_koss_capability_error():
    """测试能力位不足时抛出 KossCapabilityError"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()

def test_audit_rejection_throws_koss_security_error():
    """测试审核拒绝时抛出 KossSecurityError"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_decision_flow_capability_then_audit():
    """测试决策流程：先检查能力位，再检查审核"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)  # 没有 FS 能力
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # FS 能力被禁用，应该抛出 KossCapabilityError
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()

def test_decision_flow_audit_mask_then_callback():
    """测试决策流程：先检查审核掩码，再调用审核回调"""
    js = KossJS(capabilities=(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD))
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 只有 FS 应该触发审核
        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in calls

        # NET 不应该触发审核（掩码中没有）
        calls.clear()
        js.eval("const net_binding = internalBinding('net');")
        assert "net" not in calls
    finally:
        js.destroy()

def test_decision_flow_no_audit_when_mask_not_set():
    """测试决策流程：审核掩码未设置时，直接放行"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        # 不设置审核掩码

        js.eval("const binding = internalBinding('fs');")
        assert len(calls) == 0
    finally:
        js.destroy()

def test_decision_flow_allow_when_no_callback():
    """测试决策流程：审核掩码已设置但没有审核回调时，直接放行"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # API 调用应该直接放行
        js.eval("const binding = internalBinding('fs');")
    finally:
        js.destroy()

def test_decision_flow_capability_priority_over_audit():
    """测试决策流程：能力位检查优先于审核"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)  # 没有 FS 能力
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # FS 能力被禁用，应该抛出 KossCapabilityError，不触发审核
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)

        assert len(audit_called) == 0
    finally:
        js.destroy()

def test_decision_flow_audit_mask_priority_over_callback():
    """测试决策流程：审核掩码检查优先于审核回调"""
    js = KossJS(capabilities=(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD))
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)
        # 只审核 fs，不审核 net
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in audit_called

        audit_called.clear()
        js.eval("const net_binding = internalBinding('net');")
        assert "net" not in audit_called
    finally:
        js.destroy()

def test_decision_flow_with_multiple_capabilities():
    """测试决策流程：多个能力位的组合"""
    caps = KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD
    js = KossJS(capabilities=caps)
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET)

        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in audit_called

        js.eval("const net_binding = internalBinding('net');")
        assert "net" in audit_called
    finally:
        js.destroy()

def test_decision_flow_audit_callback_can_approve_and_deny():
    """测试决策流程：审核回调可以批准和拒绝"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        call_count = 0
        def audit(target: str, args: list[str], pwd: str | None):
            nonlocal call_count
            call_count += 1
            return call_count <= 1

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 第一次调用应该被批准
        js.eval("const binding = internalBinding('fs');")

        # 第二次调用应该被拒绝
        with pytest.raises(JsError) as exc:
            js.eval("const binding2 = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_decision_flow_with_different_targets():
    """测试决策流程：不同 API 的审核"""
    caps = KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD
    js = KossJS(capabilities=caps)
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return target == "fs"

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET)

        # fs 应该被批准
        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in audit_called

        # net 应该被拒绝
        audit_called.clear()
        with pytest.raises(JsError) as exc:
            js.eval("const net_binding = internalBinding('net');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_decision_flow_capability_only_no_audit():
    """测试决策流程：只有能力位，没有审核"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.eval("const binding = internalBinding('fs');")
    finally:
        js.destroy()

def test_decision_flow_sandbox_mode_bypasses_audit():
    """测试决策流程：沙箱模式下，能力位检查优先于审核"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)

        assert len(audit_called) == 0
    finally:
        js.destroy()

def test_decision_flow_dynamic_audit_mask_change():
    """测试决策流程：动态更改审核掩码"""
    caps = KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD
    js = KossJS(capabilities=caps)
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)

        # 初始审核 fs
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)
        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in audit_called

        # 更改审核掩码为 net
        audit_called.clear()
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_NET)
        js.eval("const net_binding = internalBinding('net');")
        assert "net" in audit_called

        # fs 不再被审核
        audit_called.clear()
        js.eval("const fs_binding2 = internalBinding('fs');")
        assert "fs" not in audit_called
    finally:
        js.destroy()

def test_decision_flow_dynamic_callback_change():
    """测试决策流程：动态更改审核回调"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        # 第一个回调：批准所有
        js.check_sandbox(lambda target, args, pwd: True)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        js.eval("const binding = internalBinding('fs');")

        # 更改回调为拒绝所有
        js.check_sandbox(lambda target, args, pwd: False)

        with pytest.raises(JsError) as exc:
            js.eval("const binding2 = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_decision_flow_callback_cleared_then_api_call():
    """测试决策流程：清除审核回调后调用 API"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 清除审核回调
        js.check_sandbox(None)

        # API 调用应该直接放行
        js.eval("const binding = internalBinding('fs');")
    finally:
        js.destroy()

def test_fetch_capability_denial():
    """测试 fetch 能力位检查"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        # fetch 函数不存在（注册阶段被禁用）
        result = js.eval("typeof fetch")
        assert result == "undefined"
    finally:
        js.destroy()

def test_fetch_capability_enabled():
    """测试 fetch 能力位启用"""
    js = KossJS(capabilities=KossJS.NET_FETCH | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof fetch")
        assert result == "function"
    finally:
        js.destroy()
