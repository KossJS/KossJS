import pytest # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS, JsError # pyright: ignore[reportUnusedImport]


def test_default_audit_mask_is_zero():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()


def test_set_audit_mask():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        assert js.get_audit_mask() == (KossJS.FS_READ | KossJS.NET_FETCH)
    finally:
        js.destroy()


def test_audit_mask_ignores_ungranted_capabilities():
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        mask = js.get_audit_mask()
        assert mask == KossJS.FS_READ
    finally:
        js.destroy()


def test_audit_mask_zero_disables_audit():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(0)
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()


# ============================================================================
# 审核掩码边界情况测试
# ============================================================================

def test_audit_mask_with_all_fs():
    """测试审核掩码设置所有文件系统操作"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_FS)
        assert js.get_audit_mask() == KossJS.KOSS_CAP_ALL_FS
    finally:
        js.destroy()

def test_audit_mask_with_all_net():
    """测试审核掩码设置所有网络操作"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_NET)
        assert js.get_audit_mask() == KossJS.KOSS_CAP_ALL_NET
    finally:
        js.destroy()

def test_audit_mask_with_all_crypto():
    """测试审核掩码设置所有加密操作"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.KOSS_CAP_ALL_CRYPTO)
        assert js.get_audit_mask() == KossJS.KOSS_CAP_ALL_CRYPTO
    finally:
        js.destroy()

def test_audit_mask_with_all():
    """测试审核掩码设置所有操作"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.KOSS_CAP_ALL)
        assert js.get_audit_mask() == KossJS.KOSS_CAP_ALL
    finally:
        js.destroy()

def test_audit_mask_dynamic_change():
    """测试审核掩码动态更改"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        # 初始设置
        js.set_audit_mask(KossJS.FS_READ)
        assert js.get_audit_mask() == KossJS.FS_READ

        # 更改为其他值
        js.set_audit_mask(KossJS.NET_FETCH)
        assert js.get_audit_mask() == KossJS.NET_FETCH

        # 更改为组合值
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        assert js.get_audit_mask() == (KossJS.FS_READ | KossJS.NET_FETCH)

        # 更改为零
        js.set_audit_mask(0)
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()

def test_audit_mask_with_single_capability():
    """测试审核掩码设置单个能力位"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        # 测试每个单独的能力位
        capabilities = [
            KossJS.FS_READ, KossJS.FS_WRITE, KossJS.FS_DELETE,
            KossJS.FS_MKDIR, KossJS.FS_RENAME, KossJS.FS_CHMOD,
            KossJS.NET_TCP_CLIENT, KossJS.NET_TCP_SERVER, KossJS.NET_UDP,
            KossJS.NET_DNS, KossJS.NET_FETCH,
            KossJS.CRYPTO_HASH, KossJS.CRYPTO_HMAC,
            KossJS.CRYPTO_RANDOM, KossJS.CRYPTO_PBKDF2,
            KossJS.FFI_OPEN, KossJS.FFI_CALL, KossJS.FFI_ALLOC,
            KossJS.FFI_CALLBACK, KossJS.FFI_STRUCT,
            KossJS.NATIVE_ADDON, KossJS.WASM, KossJS.SHARED_MEMORY,
            KossJS.HIGHRES_TIME, KossJS.SYSINFO, KossJS.MODULE_LOAD,
            KossJS.DYNAMIC_CODE, KossJS.DEBUG_CAP,
        ]

        for cap in capabilities:
            js.set_audit_mask(cap)
            assert js.get_audit_mask() == cap
    finally:
        js.destroy()

def test_audit_mask_with_combination():
    """测试审核掩码设置组合能力位"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        # 测试各种组合
        combinations = [
            KossJS.FS_READ | KossJS.FS_WRITE,
            KossJS.NET_TCP_CLIENT | KossJS.NET_FETCH,
            KossJS.CRYPTO_HASH | KossJS.CRYPTO_HMAC,
            KossJS.FFI_OPEN | KossJS.FFI_CALL,
            KossJS.FS_READ | KossJS.NET_FETCH | KossJS.CRYPTO_HASH,
        ]

        for combo in combinations:
            js.set_audit_mask(combo)
            assert js.get_audit_mask() == combo
    finally:
        js.destroy()

def test_audit_mask_ignores_all_ungranted():
    """测试审核掩码忽略所有未授予的能力位"""
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        # 尝试设置多个未授予的能力位
        js.set_audit_mask(KossJS.FS_READ | KossJS.FS_WRITE | KossJS.NET_FETCH | KossJS.CRYPTO_HASH)
        mask = js.get_audit_mask()
        # 只有 FS_READ 应该被保留
        assert mask == KossJS.FS_READ
    finally:
        js.destroy()

def test_audit_mask_with_no_capabilities():
    """测试审核掩码在没有能力位时的行为"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        # 尝试设置审核掩码
        js.set_audit_mask(KossJS.FS_READ)
        mask = js.get_audit_mask()
        # 应该返回 0，因为没有授予任何能力位
        assert mask == 0
    finally:
        js.destroy()

def test_audit_mask_with_capability_and_audit():
    """测试审核掩码与能力位的交互"""
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.FS_WRITE | KossJS.MODULE_LOAD)
    try:
        # 设置审核掩码为 FS_READ
        js.set_audit_mask(KossJS.FS_READ)
        assert js.get_audit_mask() == KossJS.FS_READ

        # 验证审核掩码只包含授予的能力位
        js.set_audit_mask(KossJS.FS_READ | KossJS.FS_WRITE | KossJS.NET_FETCH)
        mask = js.get_audit_mask()
        # NET_FETCH 应该被忽略
        assert mask == (KossJS.FS_READ | KossJS.FS_WRITE)
    finally:
        js.destroy()

def test_audit_mask_independence_from_audit_callback():
    """测试审核掩码与审核回调的独立性"""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        # 设置审核掩码
        js.set_audit_mask(KossJS.FS_READ)
        assert js.get_audit_mask() == KossJS.FS_READ

        # 注册审核回调
        js.check_sandbox(lambda target, args, pwd: True)
        # 审核掩码应该保持不变
        assert js.get_audit_mask() == KossJS.FS_READ

        # 清除审核回调
        js.check_sandbox(None)
        # 审核掩码应该仍然保持不变
        assert js.get_audit_mask() == KossJS.FS_READ
    finally:
        js.destroy()

def test_audit_mask_with_dynamic_capability_change():
    """测试审核掩码在能力位动态更改时的行为"""
    # 注意：能力位在实例创建时确定，运行时不可更改
    # 这个测试验证审核掩码正确反映初始能力位
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.MODULE_LOAD)
    try:
        # 设置审核掩码包含未授予的能力位
        js.set_audit_mask(KossJS.FS_READ | KossJS.FS_WRITE | KossJS.NET_FETCH)
        mask = js.get_audit_mask()
        # 只有 FS_READ 应该被保留
        assert mask == KossJS.FS_READ
    finally:
        js.destroy()
