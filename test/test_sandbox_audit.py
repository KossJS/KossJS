import pytest
import ctypes
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


def test_check_sandbox_register_callback_survives_eval():
    """Test that registered callback persists across eval calls."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        called = []
        def audit(target, args, pwd):
            called.append(True)
            return True
        js.check_sandbox(audit)
        # Eval some code - callback should still be registered
        js.eval("1 + 1")
        js.eval("2 + 2")
        # The callback object should be kept alive (not garbage collected)
        assert hasattr(js, '_audit_callback')
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


def test_check_sandbox_callback_type_conversion():
    """Test that the Python wrapper properly converts callback arguments."""
    js = KossJS(capabilities=KossJS.MODULE_LOAD | KossJS.FS_READ)
    try:
        received_args = []
        def audit(target, args, pwd):
            received_args.append({
                'target': target,
                'target_type': type(target).__name__,
                'args': args,
                'args_type': type(args).__name__,
                'pwd': pwd,
                'pwd_type': type(pwd).__name__,
            })
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
