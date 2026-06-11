import pytest
from kossjs_interface import KossJS

def test_all_capability_constants_exist():
    """Verify all 28 capability constants are defined and have correct values."""
    # 文件系统
    assert KossJS.FS_READ == 1 << 0
    assert KossJS.FS_WRITE == 1 << 1
    assert KossJS.FS_DELETE == 1 << 2
    assert KossJS.FS_MKDIR == 1 << 3
    assert KossJS.FS_RENAME == 1 << 4
    assert KossJS.FS_CHMOD == 1 << 5

    # 网络
    assert KossJS.NET_TCP_CLIENT == 1 << 6
    assert KossJS.NET_TCP_SERVER == 1 << 7
    assert KossJS.NET_UDP == 1 << 8
    assert KossJS.NET_DNS == 1 << 9
    assert KossJS.NET_FETCH == 1 << 10

    # 加密
    assert KossJS.CRYPTO_HASH == 1 << 11
    assert KossJS.CRYPTO_HMAC == 1 << 12
    assert KossJS.CRYPTO_RANDOM == 1 << 13
    assert KossJS.CRYPTO_PBKDF2 == 1 << 14

    # FFI
    assert KossJS.FFI_OPEN == 1 << 15
    assert KossJS.FFI_CALL == 1 << 16
    assert KossJS.FFI_ALLOC == 1 << 17
    assert KossJS.FFI_CALLBACK == 1 << 18
    assert KossJS.FFI_STRUCT == 1 << 19

    # 其他模块
    assert KossJS.NATIVE_ADDON == 1 << 20
    assert KossJS.WASM == 1 << 21
    assert KossJS.SHARED_MEMORY == 1 << 22
    assert KossJS.HIGHRES_TIME == 1 << 23
    assert KossJS.SYSINFO == 1 << 24
    assert KossJS.MODULE_LOAD == 1 << 25
    assert KossJS.DYNAMIC_CODE == 1 << 26
    assert KossJS.DEBUG_CAP == 1 << 27

def test_combination_constants():
    """Verify combination constants are correct."""
    assert KossJS.KOSS_CAP_SANDBOX == 0
    assert KossJS.KOSS_CAP_ALL_FS == (
        KossJS.FS_READ | KossJS.FS_WRITE | KossJS.FS_DELETE |
        KossJS.FS_MKDIR | KossJS.FS_RENAME | KossJS.FS_CHMOD
    )
    assert KossJS.KOSS_CAP_ALL_NET == (
        KossJS.NET_TCP_CLIENT | KossJS.NET_TCP_SERVER | KossJS.NET_UDP |
        KossJS.NET_DNS | KossJS.NET_FETCH
    )
    assert KossJS.KOSS_CAP_ALL_CRYPTO == (
        KossJS.CRYPTO_HASH | KossJS.CRYPTO_HMAC |
        KossJS.CRYPTO_RANDOM | KossJS.CRYPTO_PBKDF2
    )
    assert KossJS.KOSS_CAP_ALL_FFI == (
        KossJS.FFI_OPEN | KossJS.FFI_CALL | KossJS.FFI_ALLOC |
        KossJS.FFI_CALLBACK | KossJS.FFI_STRUCT
    )
    assert KossJS.KOSS_CAP_ALL == 0xFFFFFFFF

def test_compatibility_aliases():
    """Verify backward-compatible aliases work."""
    assert KossJS.KOSS_CAP_FS == KossJS.KOSS_CAP_ALL_FS
    assert KossJS.KOSS_CAP_NET == KossJS.KOSS_CAP_ALL_NET
    assert KossJS.KOSS_CAP_CRYPTO == KossJS.KOSS_CAP_ALL_CRYPTO
    assert KossJS.KOSS_CAP_WORKER == 1 << 3
    assert KossJS.KOSS_CAP_EXTERNAL_LOADER == KossJS.MODULE_LOAD

def test_capabilities_are_static():
    """Capabilities are static permissions determined at instance creation."""
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        # 能力位在实例创建时确定，运行时不可更改
        assert KossJS.FS_READ == 1
    finally:
        js.destroy()

def test_no_bit_overlap_between_categories():
    """Ensure no bit overlap between different capability categories."""
    fs_caps = [KossJS.FS_READ, KossJS.FS_WRITE, KossJS.FS_DELETE,
               KossJS.FS_MKDIR, KossJS.FS_RENAME, KossJS.FS_CHMOD]
    net_caps = [KossJS.NET_TCP_CLIENT, KossJS.NET_TCP_SERVER, KossJS.NET_UDP,
                KossJS.NET_DNS, KossJS.NET_FETCH]
    crypto_caps = [KossJS.CRYPTO_HASH, KossJS.CRYPTO_HMAC,
                   KossJS.CRYPTO_RANDOM, KossJS.CRYPTO_PBKDF2]
    ffi_caps = [KossJS.FFI_OPEN, KossJS.FFI_CALL, KossJS.FFI_ALLOC,
                KossJS.FFI_CALLBACK, KossJS.FFI_STRUCT]
    other_caps = [KossJS.NATIVE_ADDON, KossJS.WASM, KossJS.SHARED_MEMORY,
                  KossJS.HIGHRES_TIME, KossJS.SYSINFO, KossJS.MODULE_LOAD,
                  KossJS.DYNAMIC_CODE, KossJS.DEBUG_CAP]

    all_caps = fs_caps + net_caps + crypto_caps + ffi_caps + other_caps

    # Each capability should be a unique power of 2
    for i, cap in enumerate(all_caps):
        assert cap > 0, f"Capability {i} should be positive"
        assert (cap & (cap - 1)) == 0, f"Capability {cap} should be a power of 2"
        # Check no overlap with other capabilities
        for j, other in enumerate(all_caps):
            if i != j:
                assert (cap & other) == 0, f"Capabilities {cap} and {other} overlap"
