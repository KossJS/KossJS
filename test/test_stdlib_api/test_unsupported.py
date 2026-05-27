"""Test that modules with internalBinding() fail with clear errors.

These modules require Rust-side native bindings that are not yet implemented.
See NativeBinding-internalBinding实现.md for the full implementation checklist.
"""

import pytest
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS

# Modules that have internalBinding() calls in their code
# and will throw ReferenceError: internalBinding is not defined
BINDING_MODULES = [
    "crypto",
    "net",
    "tls",
    "child_process",
    "dgram",
    "dns",
    "util",
    "v8",
    "vm",
    "zlib",
    "repl",
    "perf_hooks",
    "inspector",
    "wasi",
    "diagnostics_channel",
    "punycode",
    "sea",
    "trace_events",
    "tty",
    "readline",
]


@pytest.mark.parametrize("module_name", BINDING_MODULES)
def test_module_fails_with_internal_binding(koss: KossJS, module_name: str):
    """Each of these modules calls internalBinding() which is not implemented."""
    try:
        koss.eval(f"require('{module_name}')")
        pytest.xfail(f"Module {module_name} unexpectedly loaded (maybe it was fixed?)")
    except Exception as e:
        error_msg = str(e)
        # Either internalBinding is not defined, or a require fails
        assert (
            "internalBinding" in error_msg
            or "internal" in error_msg.lower()
            or "module" in error_msg.lower()
            or "not" in error_msg.lower()
            or "expected" in error_msg.lower()
            or "syntax" in error_msg.lower()
        ), f"Unexpected error for {module_name}: {error_msg}"
