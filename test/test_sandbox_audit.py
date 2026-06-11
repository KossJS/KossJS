import pytest
from kossjs_interface import KossJS, JsError, KossResult


def test_check_sandbox_register_and_clear():
    """Test that audit callback can be registered and cleared."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        # Register a callback
        js.check_sandbox(lambda target, args, pwd: True)
        # Clear the callback
        js.check_sandbox(None)
        # Should still be able to eval
        js.eval("1 + 1")
    finally:
        js.destroy()


def test_check_sandbox_callback_liveness_after_eval():
    """Test that registered callback object persists across eval calls.

    Note: Actual invocation testing requires Task 4 (audit decision function)
    wired into API calls. This test only verifies the callback object is
    retained (not garbage collected) after eval.
    """
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        def audit(target, args, pwd):
            return True
        js.check_sandbox(audit)
        # Eval some code - callback should still be registered
        js.eval("1 + 1")
        js.eval("2 + 2")
        # The callback object should be kept alive (not garbage collected)
        assert hasattr(js, '_audit_callback')
        assert js._audit_callback is not None
    finally:
        js.destroy()


def test_check_sandbox_clear_callback():
    """Test that clearing callback sets internal state to None."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        js.check_sandbox(lambda target, args, pwd: True)
        assert hasattr(js, '_audit_callback')
        js.check_sandbox(None)
        assert js._audit_callback is None
    finally:
        js.destroy()


def test_audit_mask_and_callback_independent():
    """Test that audit mask and audit callback are independent."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        # Set audit mask
        js.set_audit_mask(KossJS.FS_READ)
        assert js.get_audit_mask() == KossJS.FS_READ

        # Register callback - audit mask should be unchanged
        js.check_sandbox(lambda target, args, pwd: True)
        assert js.get_audit_mask() == KossJS.FS_READ

        # Clear callback - audit mask should still be unchanged
        js.check_sandbox(None)
        assert js.get_audit_mask() == KossJS.FS_READ
    finally:
        js.destroy()


def test_check_sandbox_callback_wrapper_exists():
    """Test that check_sandbox wraps the callback correctly.

    Note: Actual type conversion verification requires Task 4 integration
    to trigger the callback from the native side with real arguments.
    This test only verifies the wrapper is stored.
    """
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        def audit(target, args, pwd):
            return True
        js.check_sandbox(audit)
        # Verify the callback wrapper was set up
        assert hasattr(js, '_audit_callback')
        assert js._audit_callback is not None
    finally:
        js.destroy()


def test_check_sandbox_with_audit_mask():
    """Test registering callback with audit mask set."""
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.FS_WRITE | KossJS.MODULE_LOAD)
    try:
        js.set_audit_mask(KossJS.FS_READ)
        js.check_sandbox(lambda target, args, pwd: True)

        # Verify both are set
        assert js.get_audit_mask() == KossJS.FS_READ
        assert js._audit_callback is not None
    finally:
        js.destroy()


# --- Tests requiring Task 4 (audit decision integration) ---


@pytest.mark.skip(reason="Requires Task 4: audit decision function wired into API calls")
def test_sync_audit_allows_safe_read_path():
    """Test that sync audit allows a safe read path."""
    pass


@pytest.mark.skip(reason="Requires Task 4: audit decision function wired into API calls")
def test_sync_audit_rejects_path():
    """Test that sync audit rejects a disallowed path."""
    pass


@pytest.mark.skip(reason="Requires Task 4: audit decision function wired into API calls")
def test_no_audit_when_mask_not_set():
    """Test that no audit occurs when mask is not set."""
    pass
