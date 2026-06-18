"""加密能力位功能测试 - 验证 CRYPTO_HASH, CRYPTO_HMAC, CRYPTO_RANDOM, CRYPTO_PBKDF2 能力位

这些能力位控制加密模块（crypto）的访问权限。
"""
import pytest
from kossjs_interface import KossJS, JsError


# ============================================================================
# 加密能力位常量测试
# ============================================================================

def test_crypto_hash_constant_exists():
    """CRYPTO_HASH 常量应该存在且值正确"""
    assert KossJS.CRYPTO_HASH == 1 << 11


def test_crypto_hmac_constant_exists():
    """CRYPTO_HMAC 常量应该存在且值正确"""
    assert KossJS.CRYPTO_HMAC == 1 << 12


def test_crypto_random_constant_exists():
    """CRYPTO_RANDOM 常量应该存在且值正确"""
    assert KossJS.CRYPTO_RANDOM == 1 << 13


def test_crypto_pbkdf2_constant_exists():
    """CRYPTO_PBKDF2 常量应该存在且值正确"""
    assert KossJS.CRYPTO_PBKDF2 == 1 << 14


def test_crypto_capabilities_in_combination():
    """所有加密能力位应该包含在 KOSS_CAP_ALL_CRYPTO 中"""
    assert (KossJS.KOSS_CAP_ALL_CRYPTO & KossJS.CRYPTO_HASH) == KossJS.CRYPTO_HASH
    assert (KossJS.KOSS_CAP_ALL_CRYPTO & KossJS.CRYPTO_HMAC) == KossJS.CRYPTO_HMAC
    assert (KossJS.KOSS_CAP_ALL_CRYPTO & KossJS.CRYPTO_RANDOM) == KossJS.CRYPTO_RANDOM
    assert (KossJS.KOSS_CAP_ALL_CRYPTO & KossJS.CRYPTO_PBKDF2) == KossJS.CRYPTO_PBKDF2


def test_crypto_capabilities_not_in_sandbox():
    """沙箱模式下不应该有任何加密能力位"""
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.CRYPTO_HASH) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.CRYPTO_HMAC) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.CRYPTO_RANDOM) == 0
    assert (KossJS.KOSS_CAP_SANDBOX & KossJS.CRYPTO_PBKDF2) == 0


def test_crypto_capabilities_isolation():
    """各加密能力位之间应该相互独立"""
    caps = [KossJS.CRYPTO_HASH, KossJS.CRYPTO_HMAC, KossJS.CRYPTO_RANDOM,
            KossJS.CRYPTO_PBKDF2]

    for i, cap in enumerate(caps):
        assert cap > 0, f"Capability {i} should be positive"
        assert (cap & (cap - 1)) == 0, f"Capability {cap} should be a power of 2"
        for j, other in enumerate(caps):
            if i != j:
                assert (cap & other) == 0, f"Capabilities {cap} and {other} overlap"


# ============================================================================
# internalBinding 加密模块测试
# ============================================================================

def test_internalbinding_crypto_disabled_without_caps():
    """没有加密能力位时，internalBinding('crypto') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('crypto')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


def test_internalbinding_crypto_enabled_with_all_crypto_caps():
    """KOSS_CAP_ALL_CRYPTO 启用时，internalBinding('crypto') 应该可用"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_CRYPTO | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('crypto')")
        assert result == "object"
    finally:
        js.destroy()


# ============================================================================
# 沙箱模式测试
# ============================================================================

def test_crypto_module_disabled_in_sandbox():
    """沙箱模式下，internalBinding('crypto') 应该被拒绝"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('crypto')")
        assert "KossCapabilityError" in str(exc.value)
    finally:
        js.destroy()


# ============================================================================
# 单个加密能力位测试
# ============================================================================

def test_crypto_with_single_cap_allows_binding():
    """单个加密能力位（属于 KOSS_CAP_ALL_CRYPTO 组）足以访问 internalBinding"""
    js = KossJS(capabilities=KossJS.CRYPTO_HASH | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('crypto')")
        assert result == "object"
    finally:
        js.destroy()


def test_crypto_with_koss_cap_crypto_alias():
    """KOSS_CAP_CRYPTO 别名应该等同于 KOSS_CAP_ALL_CRYPTO"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_CRYPTO | KossJS.MODULE_LOAD)
    try:
        result = js.eval("typeof internalBinding('crypto')")
        assert result == "object"
    finally:
        js.destroy()


# ============================================================================
# 审核回调测试
# ============================================================================

def test_crypto_audit_callback_triggered():
    """加密模块访问应该触发审核回调"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_CRYPTO | KossJS.MODULE_LOAD)
    calls: list[str] = []
    try:
        def audit(target: str, args: list[str], pwd: str | None):
            calls.append(target)
            return True

        js.check_sandbox(audit)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_CRYPTO)

        js.eval("const crypto = internalBinding('crypto');")
        assert "crypto" in calls
    finally:
        js.destroy()


def test_crypto_audit_callback_can_reject():
    """审核回调可以拒绝加密模块访问"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL_CRYPTO | KossJS.MODULE_LOAD)
    try:
        js.check_sandbox(lambda target, args, pwd: False)
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_CRYPTO)

        with pytest.raises(JsError) as exc:
            js.eval("internalBinding('crypto')")
        assert "KossSecurityError" in str(exc.value)
    finally:
        js.destroy()


# ============================================================================
# 审核掩码测试
# ============================================================================

def test_audit_mask_with_crypto_caps():
    """审核掩码应该支持加密能力位"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        caps = [KossJS.CRYPTO_HASH, KossJS.CRYPTO_HMAC, KossJS.CRYPTO_RANDOM,
                KossJS.CRYPTO_PBKDF2]

        for cap in caps:
            js.set_audit_mask(cap)
            assert js.get_audit_mask() == cap
    finally:
        js.destroy()
