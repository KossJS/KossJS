"""网络能力位功能测试 - 验证 NET_TCP_CLIENT, NET_TCP_SERVER, NET_UDP, NET_DNS 能力位

这些能力位控制网络模块（net, dns, dgram）的访问权限。
"""
import pytest
from kossjs_interface import KossJS, JsError


# ============================================================================
# 网络能力位常量测试
# ============================================================================

def test_net_tcp_client_constant_exists():
    """NET_TCP_CLIENT 常量应该存在且值正确"""
    assert KossJS.NET_TCP_CLIENT == 1 << 6


def test_net_tcp_server_constant_exists():
    """NET_TCP_SERVER 常量应该存在且值正确"""
    assert KossJS.NET_TCP_SERVER == 1 << 7


def test_net_udp_constant_exists():
    """NET_UDP 常量应该存在且值正确"""
    assert KossJS.NET_UDP == 1 << 8


def test_net_dns_constant_exists():
    """NET_DNS 常量应该存在且值正确"""
    assert KossJS.NET_DNS == 1 << 9


def test_net_fetch_constant_exists():
    """NET_FETCH 常量应该存在且值正确"""
    assert KossJS.NET_FETCH == 1 << 10


def test_net_capabilities_in_combination():
    """所有网络能力位应该包含在 KOSS_CAP_ALL_NET 中"""
    assert (KossJS.KOSS_CAP_ALL_NET & KossJS.NET_TCP_CLIENT) == KossJS.NET_TCP_CLIENT
    assert (KossJS.KOSS_CAP_ALL_NET & KossJS.NET_TCP_SERVER) == KossJS.NET_TCP_SERVER
    assert (KossJS.KOSS_CAP_ALL_NET & KossJS.NET_UDP) == KossJS.NET_UDP
    assert (KossJS.KOSS_CAP_ALL_NET & KossJS.NET_DNS) == KossJS.NET_DNS
    assert (KossJS.KOSS_CAP_ALL_NET & KossJS.NET_FETCH) == KossJS.NET_FETCH


def test_net_capabilities_not_in_sandbox():
    """沙箱模式下不应该有任何网络能力位"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NET_TCP_CLIENT) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NET_TCP_SERVER) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NET_UDP) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NET_DNS) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.NET_FETCH) == 0


def test_net_capabilities_isolation():
    """各网络能力位之间应该相互独立"""
    caps = [KossJS.NET_TCP_CLIENT, KossJS.NET_TCP_SERVER, KossJS.NET_UDP,
            KossJS.NET_DNS, KossJS.NET_FETCH]

    for i, cap in enumerate(caps):
        assert cap > 0, f"Capability {i} should be positive"
        assert (cap & (cap - 1)) == 0, f"Capability {cap} should be a power of 2"
        for j, other in enumerate(caps):
            if i != j:
                assert (cap & other) == 0, f"Capabilities {cap} and {other} overlap"


# ============================================================================
# internalBinding 网络模块测试
# ============================================================================

def test_internalbinding_net_disabled_without_caps():
    """没有网络能力位时，internalBinding('net') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('net')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_internalbinding_net_enabled_with_all_net_caps():
    """KOSS_CAP_ALL_NET 启用时，internalBinding('net') 应该可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('net')")
        assert result == "object"
    finally:
        js.destroy()


def test_internalbinding_dns_disabled_without_caps():
    """没有网络能力位时，internalBinding('dns') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('dns')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_internalbinding_dns_enabled_with_all_net_caps():
    """KOSS_CAP_ALL_NET 启用时，internalBinding('dns') 应该可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('dns')")
        assert result == "object"
    finally:
        js.destroy()


def test_internalbinding_dgram_disabled_without_caps():
    """没有网络能力位时，internalBinding('dgram') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('dgram')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_internalbinding_dgram_enabled_with_all_net_caps():
    """KOSS_CAP_ALL_NET 启用时，internalBinding('dgram') 应该可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('dgram')")
        assert result == "object"
    finally:
        js.destroy()


def test_internalbinding_url_disabled_without_caps():
    """没有网络能力位时，internalBinding('url') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('url')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_internalbinding_url_enabled_with_all_net_caps():
    """KOSS_CAP_ALL_NET 启用时，internalBinding('url') 应该可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('url')")
        assert result == "object"
    finally:
        js.destroy()


# ============================================================================
# 沙箱模式测试
# ============================================================================

def test_net_module_disabled_in_sandbox():
    """沙箱模式下，internalBinding('net') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('net')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_dns_module_disabled_in_sandbox():
    """沙箱模式下，internalBinding('dns') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('dns')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


# ============================================================================
# 单个网络能力位测试
# ============================================================================

def test_net_with_single_cap_allows_binding():
    """单个网络能力位（属于 KOSS_CAP_ALL_NET 组）足以访问 internalBinding"""
    js = KossJS(capabilities=KossJS.NET_TCP_CLIENT | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('net')")
        assert result == "object"
    finally:
        js.destroy()


def test_net_with_koss_cap_net_alias():
    """KOSS_CAP_NET 别名应该等同于 KOSS_CAP_ALL_NET"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_NET | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('net')")
        assert result == "object"
    finally:
        js.destroy()


# ============================================================================
# 审核回调测试
# ============================================================================

def test_net_audit_callback_triggered():
    """网络模块访问应该触发审核回调"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_NET)

        js.eval("const net = internalBinding('net');")
        assert "net" in calls
    finally:
        js.destroy()


def test_net_audit_callback_can_reject():
    """审核回调可以拒绝网络模块访问"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_NET | KossJS.MODULE_LOAD)
    try:
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_NET)

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('net')")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()


# ============================================================================
# fetch 能力位测试 (NET_FETCH)
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


# ============================================================================
# 审核掩码测试
# ============================================================================

def test_audit_mask_with_net_caps():
    """审核掩码应该支持网络能力位"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        caps = [KossJS.NET_TCP_CLIENT, KossJS.NET_TCP_SERVER, KossJS.NET_UDP,
                KossJS.NET_DNS, KossJS.NET_FETCH]

        for cap in caps:
            js.set_audit_mask(cap)
            assert js.get_audit_mask() == cap
    finally:
        js.destroy()
