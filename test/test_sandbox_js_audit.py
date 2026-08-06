"""JS 层审核回调（两级审核链）测试 - v0.1.0-dev.10

语义：
  1. Audit Mask = 0            → 不触发审核，直接依据 Capability 放行/拒绝
  2. Audit Mask ≠ 0 且宿主回调 NULL → 抛 KossConfigError（即使已注册 JS 回调）
  3. Audit Mask ≠ 0 且宿主回调存在：
       - 宿主返回 false → 拒绝（KossSecurityError），JS 回调不被调用
       - 宿主返回 true  → 若注册了 JS 回调则调用其进一步限制；
                          JS 返回 false / 抛异常 / 重入 → 拒绝
"""
import pytest

from kossjs_interface import KossJS, JsError


def _fs_js() -> KossJS:
    """带 FS 能力 + 模块加载能力的实例。"""
    return KossJS(capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.MODULE_LOAD)


def _audit_fs(js: KossJS) -> None:
    js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)


def _register_js_audit(js: KossJS, body: str) -> None:
    js.eval(f"__koss_set_audit_callback(function(t, a, p) {{ {body} }})")


# ---------------------------------------------------------------------------
# JS 回调：拒绝 / 放行 / 异常
# ---------------------------------------------------------------------------

def test_js_audit_denies_when_callback_returns_false():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(js, "return false;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()


def test_js_audit_allows_when_callback_returns_true():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(js, "return true;")

        js.eval("internalBinding('fs');")
    finally:
        js.destroy()


def test_js_audit_receives_target_args_and_pwd():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(
            js,
            "globalThis.__audit_target = t;"
            "globalThis.__audit_pwd = p;"
            "return true;",
        )

        js.eval("internalBinding('fs');")
        target = js.eval("globalThis.__audit_target")
        assert target == "fs"
        # eval 场景下没有模块上下文，pwd 应为 null
        pwd = js.eval("globalThis.__audit_pwd")
        assert pwd == "null" or pwd is None
    finally:
        js.destroy()


def test_js_audit_exception_denies():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(js, "throw new Error('js policy error');")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "js policy error" in str(exc.value)
    finally:
        js.destroy()


def test_js_audit_reentrancy_is_denied():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        # JS 回调内部再次调用受保护 API → 触发重入，应被拒绝并抛出
        _register_js_audit(js, "internalBinding('fs'); return true;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "re-entrancy" in str(exc.value)
    finally:
        js.destroy()


# ---------------------------------------------------------------------------
# 宿主回调与 JS 回调的关系
# ---------------------------------------------------------------------------

def test_host_denial_skips_js_audit():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: False)  # 宿主拒绝
        _audit_fs(js)
        _register_js_audit(js, "globalThis.__js_audit_called = true; return true;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)

        called = js.eval("typeof globalThis.__js_audit_called")
        assert called == "undefined"
    finally:
        js.destroy()


def test_js_audit_without_host_callback_raises_config_error():
    js = _fs_js()
    try:
        # 不注册宿主回调，仅注册 JS 回调
        _audit_fs(js)
        _register_js_audit(js, "return true;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossConfigError" in str(exc.value)
        assert "Audit mask is set but no callback is registered" in str(exc.value)
    finally:
        js.destroy()


def test_host_allow_then_js_deny():
    js = _fs_js()
    try:
        host_calls = []
        js.check_sandbox(lambda target, args, pwd: host_calls.append(target) or True)
        _audit_fs(js)
        _register_js_audit(js, "return false;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)
        # 宿主回调确实被调用了
        assert "fs" in host_calls
    finally:
        js.destroy()


def test_js_audit_can_be_cleared_via_c_abi():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(js, "return false;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)

        # 宿主通过 C ABI 清除 JS 审核回调
        result = js.clear_js_audit()
        assert "cleared" in result

        js.eval("internalBinding('fs');")  # 放行
    finally:
        js.destroy()


def test_js_audit_can_be_cleared_via_js():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        _audit_fs(js)
        _register_js_audit(js, "return false;")

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('fs');")
        assert "KossSecurityError" in str(exc.value)

        # JS 通过传 null 清除审核回调
        js.eval("__koss_set_audit_callback(null);")
        js.eval("internalBinding('fs');")  # 放行
    finally:
        js.destroy()


def test_js_audit_mask_zero_does_not_invoke_js_callback():
    js = _fs_js()
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        # 不设置审核掩码
        _register_js_audit(js, "globalThis.__js_audit_called = true; return true;")

        js.eval("internalBinding('fs');")
        called = js.eval("typeof globalThis.__js_audit_called")
        assert called == "undefined"
    finally:
        js.destroy()
