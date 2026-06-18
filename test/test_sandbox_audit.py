"""审核回调触发测试 - 验证审核回调在 API 调用时被正确触发

注意：当前实现中，fs 模块是通过 embedded stdlib 的 stub 实现的，
不会调用原生绑定。因此 fs 操作不会触发审核回调。
审核回调只能通过 __koss_bindings（internalBinding）路径触发，
或者通过 fetch 等直接注册的原生函数触发。
"""
import pytest
from kossjs_interface import KossJS, JsError, KossResult # pyright: ignore[reportUnusedImport]


def test_check_sandbox_register_and_clear():
    """Test that audit callback can be registered and cleared."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        js.check_sandbox(None)
        js.eval("1 + 1")
    finally:
        js.destroy()


def test_check_sandbox_callback_liveness_after_eval():
    """Test that registered callback object persists across eval calls."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            return True
        js.check_sandbox(audit)
        js.eval("1 + 1")
        js.eval("2 + 2")
        assert hasattr(js, '_audit_callback')
        assert js._audit_callback is not None # pyright: ignore[reportPrivateUsage]
    finally:
        js.destroy()


def test_check_sandbox_clear_callback():
    """Test that clearing callback sets internal state to None."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        assert hasattr(js, '_audit_callback')
        js.check_sandbox(None)
        assert js._audit_callback is None # pyright: ignore[reportPrivateUsage]
    finally:
        js.destroy()


def test_audit_mask_and_callback_independent():
    """Test that audit mask and audit callback are independent."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        js.set_audit_mask(KossJS.FS_READ)
        assert js.get_audit_mask() == KossJS.FS_READ

        js.check_sandbox(lambda target, args, pwd: True)
        assert js.get_audit_mask() == KossJS.FS_READ

        js.check_sandbox(None)
        assert js.get_audit_mask() == KossJS.FS_READ
    finally:
        js.destroy()


def test_check_sandbox_callback_wrapper_exists():
    """Test that check_sandbox wraps the callback correctly."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            return True
        js.check_sandbox(audit)
        assert hasattr(js, '_audit_callback')
        assert js._audit_callback is not None # pyright: ignore[reportPrivateUsage]
    finally:
        js.destroy()


def test_check_sandbox_with_audit_mask():
    """Test registering callback with audit mask set."""
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.FS_WRITE | KossJS.MODULE_LOAD)
    try:
        js.set_audit_mask(KossJS.FS_READ)
        js.check_sandbox(lambda target, args, pwd: True)
        assert js.get_audit_mask() == KossJS.FS_READ
        assert js._audit_callback is not None # pyright: ignore[reportPrivateUsage]
    finally:
        js.destroy()


# ============================================================================
# 审核回调通过 internalBinding 路径触发的测试
# ============================================================================

def test_audit_callback_triggered_by_binding():
    """测试审核回调通过 __koss_bindings (internalBinding) 路径触发"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 通过 internalBinding 调用 fs 模块
        js.eval("""
            const binding = internalBinding('fs');
        """)

        # 审核回调应该被触发
        assert "fs" in calls
    finally:
        js.destroy()

def test_audit_callback_rejects_binding():
    """测试审核回调拒绝通过 __koss_bindings 路径的调用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 通过 internalBinding 调用 fs 模块应该被拒绝
        with pytest.raises(JsError) as exc:
            js.eval("""
                const binding = internalBinding('fs');
            """)
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_audit_callback_not_triggered_when_mask_not_set():
    """测试不设置审核掩码时，审核回调不被触发"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        # 不设置审核掩码

        # 通过 internalBinding 调用 fs 模块
        js.eval("""
            const binding = internalBinding('fs');
        """)

        # 审核回调不应该被调用
        assert len(calls) == 0
    finally:
        js.destroy()

def test_audit_callback_with_selective_approval():
    """测试审核回调选择性批准"""
    js = KossJS(capabilities=(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD))
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            # 只允许 fs，拒绝 net
            if target == "fs":
                return True
            return False

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET)

        # fs 应该被允许
        js.eval("const fs_binding = internalBinding('fs');")

        # net 应该被拒绝
        with pytest.raises(JsError) as exc:
            js.eval("const net_binding = internalBinding('net');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_audit_callback_receives_correct_target():
    """测试审核回调接收到正确的 target 参数"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    received_targets: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            received_targets.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        js.eval("const binding = internalBinding('fs');")
        assert "fs" in received_targets
    finally:
        js.destroy()

def test_audit_callback_exception_treated_as_rejection():
    """测试审核回调抛出异常被视为拒绝"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            raise Exception("audit error")

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_audit_mask_controls_which_bindings_are_audited():
    """测试审核掩码控制哪些 binding 需要审核"""
    js = KossJS(capabilities=(KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD))
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        # 只审核 fs，不审核 net
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # fs 应该触发审核
        js.eval("const fs_binding = internalBinding('fs');")
        assert "fs" in calls

        # net 不应该触发审核
        calls.clear()
        js.eval("const net_binding = internalBinding('net');")
        assert "net" not in calls
    finally:
        js.destroy()

def test_audit_callback_with_allow_and_deny():
    """测试审核回调允许和拒绝的完整流程"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        call_count = 0
        def audit(target: str, args: list[str], pwd: str | None):
            nonlocal call_count
            call_count += 1
            return call_count <= 1

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 第一次调用应该被允许
        js.eval("const binding = internalBinding('fs');")

        # 第二次调用应该被拒绝
        with pytest.raises(JsError) as exc:
            js.eval("const binding2 = internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()

def test_no_audit_when_callback_not_registered():
    """测试没有注册审核回调时，API 调用直接放行"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)
    try:
        # 设置审核掩码但不注册审核回调
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # API 调用应该直接放行
        js.eval("const binding = internalBinding('fs');")
    finally:
        js.destroy()

def test_capability_denial_overrides_audit():
    """测试能力位检查优先于审核"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)  # 没有 FS 能力
    audit_called: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            audit_called.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)

        # 没有 FS 能力，应该抛出 KossCapabilityError，不触发审核
        with pytest.raises(JsError) as exc:
            js.eval("const binding = internalBinding('fs');")
        assert "KossCapabilityError" in str(exc.value)

        # 审核回调不应该被调用
        assert len(audit_called) == 0
    finally:
        js.destroy()
