import sys
import os
import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from kossjs_interface import KossJS


@pytest.fixture
def koss():
    instance = KossJS()
    yield instance
    instance.destroy()


def eval_js(koss: KossJS, code: str) -> str:
    """Evaluate JS code and return the string result."""
    return koss.eval(code)


def require_module(koss: KossJS, name: str) -> str:
    """Try to require() a module and return a status string."""
    return koss.eval(f"typeof require('{name}')")
