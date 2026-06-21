import pytest # pyright: ignore[reportUnusedImport]
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from kossjs_interface import KossJS, JsError


# ============================================================================
# Stable mode — stable=true (default)
# ============================================================================

def test_stable_default_disables_ffi_caps():
    """stable=True (default) should strip FFI capability bits."""
    instance = KossJS(stable=True)
    caps = instance.get_capabilities()
    # FFI bits (15-19) should all be 0
    assert caps & KossJS.KOSS_CAP_ALL_FFI == 0
    instance.destroy()


def test_stable_default_disables_worker_caps():
    """stable=True (default) should strip Worker capability bit."""
    instance = KossJS(stable=True)
    caps = instance.get_capabilities()
    assert caps & KossJS.KOSS_CAP_WORKER == 0
    instance.destroy()


def test_stable_true_preserves_other_caps():
    """stable=True should only strip FFI/Worker, not other capabilities."""
    # Note: KOSS_CAP_WORKER (1<<3) overlaps with FS_MKDIR (1<<3) in the original design,
    # so stripping Worker also clears that FS bit. This test verifies FFI is stripped.
    instance = KossJS(
        capabilities=KossJS.KOSS_CAP_ALL_FS | KossJS.KOSS_CAP_ALL_CRYPTO | KossJS.KOSS_CAP_ALL_FFI,
        stable=True,
    )
    caps = instance.get_capabilities()
    # FFI bits should be stripped
    assert caps & KossJS.KOSS_CAP_ALL_FFI == 0
    # CRYPTO bits should be preserved
    assert caps & KossJS.KOSS_CAP_ALL_CRYPTO == KossJS.KOSS_CAP_ALL_CRYPTO
    instance.destroy()


def test_stable_true_ffi_call_throws_error():
    """In stable mode, calling FFI should throw an explicit error."""
    instance = KossJS(stable=True)
    # _senri_ffi.func is defined as a stub, calling it should throw
    try:
        result = instance.eval("_senri_ffi.func()") # pyright: ignore[reportUnusedVariable]
        # If eval succeeds, the result should be undefined or error
        # The stub should have thrown, so we shouldn't reach here
        assert False, "Expected JsError to be raised"
    except JsError as e:
        assert "stable mode" in str(e) or "FFI is disabled" in str(e)
    instance.destroy()


def test_stable_true_worker_call_throws_error():
    """In stable mode, calling Worker should throw an explicit error."""
    instance = KossJS(stable=True)
    try:
        result = instance.eval("__koss_create_worker_pool(2)") # pyright: ignore[reportUnusedVariable]
        # If eval succeeds, the result should be undefined or error
        # The stub should have thrown, so we shouldn't reach here
        assert False, "Expected JsError to be raised"
    except JsError as e:
        assert "stable mode" in str(e) or "Worker is disabled" in str(e)
    instance.destroy()


def test_is_stable_true():
    """koss_is_stable should return True for stable instances."""
    instance = KossJS(stable=True)
    assert instance.is_stable is True
    instance.destroy()


# ============================================================================
# Unstable mode — stable=false
# ============================================================================

def test_unstable_preserves_ffi_caps():
    """stable=False should preserve FFI capability bits."""
    instance = KossJS(
        capabilities=KossJS.KOSS_CAP_ALL_FFI,
        stable=False,
    )
    caps = instance.get_capabilities()
    assert caps & KossJS.KOSS_CAP_ALL_FFI == KossJS.KOSS_CAP_ALL_FFI
    instance.destroy()


def test_unstable_preserves_worker_caps():
    """stable=False should preserve Worker capability bit."""
    instance = KossJS(
        capabilities=KossJS.KOSS_CAP_ALL | KossJS.KOSS_CAP_WORKER,
        stable=False,
    )
    caps = instance.get_capabilities()
    assert caps & KossJS.KOSS_CAP_WORKER != 0
    instance.destroy()


def test_unstable_ffi_function_accessible():
    """In unstable mode, FFI functions should be callable (not throw)."""
    instance = KossJS(
        capabilities=KossJS.KOSS_CAP_ALL_FFI,
        stable=False,
    )
    # _senri_ffi.func should be accessible (may fail with args, but not with stable error)
    result = instance.eval("typeof _senri_ffi")
    # result is a string like "object" - if FFI is registered, it should be "object"
    assert result == "object"
    instance.destroy()


def test_is_stable_false():
    """koss_is_stable should return False for unstable instances."""
    instance = KossJS(stable=False)
    assert instance.is_stable is False
    instance.destroy()


# ============================================================================
# Default creation (no stable param) should be stable
# ============================================================================

def test_default_creation_is_stable():
    """Default KossJS() should be in stable mode."""
    instance = KossJS()
    assert instance.is_stable is True
    caps = instance.get_capabilities()
    assert caps & KossJS.KOSS_CAP_ALL_FFI == 0
    assert caps & KossJS.KOSS_CAP_WORKER == 0
    instance.destroy()
