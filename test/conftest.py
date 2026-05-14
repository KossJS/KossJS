import pytest
from kossjs_interface import KossJS


def pytest_addoption(parser: pytest.Parser):
    parser.addoption(
        "--run-skipped",
        action="store_true",
        default=False,
        help="run skipped tests (skip markers are ignored)",
    )


def pytest_collection_modifyitems(config: pytest.Config, items: list[pytest.Item]):
    if config.getoption("--run-skipped"):
        for item in items:
            own = item.own_markers
            own[:] = [m for m in own if m.name != "skip"]
            parent = item.parent
            if parent is not None:
                pown = parent.own_markers
                pown[:] = [m for m in pown if m.name != "skip"]


@pytest.fixture
def koss():
    instance = KossJS()
    yield instance
    instance.destroy()
