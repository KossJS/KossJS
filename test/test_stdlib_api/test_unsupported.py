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
# All modules that previously needed internalBinding are now supported.
# Remaining unsupported modules are listed in UNSUPPORTED_MODULES below.
UNSUPPORTED_MODULES = [
    "wasi",
    "sea",
    "punycode",
    "v8",
    "inspector",
    "async_hooks",
    "cluster",
    "readline",
    "repl",
    "vm",
    "child_process",
]


@pytest.mark.parametrize("module_name", UNSUPPORTED_MODULES)
def test_unsupported_module_fails(koss: KossJS, module_name: str):
    """These modules are not supported and should fail to load."""
    with pytest.raises(Exception):
        koss.eval(f"require('koss:node/{module_name}')")
