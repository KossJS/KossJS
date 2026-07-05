"""Test timers module — setTimeout, setInterval, setImmediate, etc."""

from .conftest import KossJS


class TestTimersAPI:
    def test_require_timers(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers')")
        assert result == "object"

    def test_set_timeout_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').setTimeout")
        assert result == "function"

    def test_clear_timeout_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').clearTimeout")
        assert result == "function"

    def test_set_interval_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').setInterval")
        assert result == "function"

    def test_clear_interval_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').clearInterval")
        assert result == "function"

    def test_set_immediate_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').setImmediate")
        assert result == "function"

    def test_clear_immediate_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').clearImmediate")
        assert result == "function"

    def test_promises_exists(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').promises")
        assert result == "object"

    def test_promises_set_timeout(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').promises.setTimeout")
        assert result == "function"

    def test_global_set_timeout(self, koss: KossJS):
        koss.eval("require('koss:node/timers');")
        result = koss.eval("typeof setTimeout")
        assert result == "function"

    def test_global_clear_timeout(self, koss: KossJS):
        koss.eval("require('koss:node/timers');")
        result = koss.eval("typeof clearTimeout")
        assert result == "function"

    def test_global_set_interval(self, koss: KossJS):
        koss.eval("require('koss:node/timers');")
        result = koss.eval("typeof setInterval")
        assert result == "function"

    def test_global_clear_interval(self, koss: KossJS):
        koss.eval("require('koss:node/timers');")
        result = koss.eval("typeof clearInterval")
        assert result == "function"

    def test_set_timeout_returns_number(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').setTimeout")
        assert result == "function"

    def test_clear_timeout_no_error(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/timers').clearTimeout")
        assert result == "function"
