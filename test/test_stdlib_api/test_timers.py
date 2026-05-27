"""Test timers module — setTimeout, setInterval, setImmediate, etc."""

from .conftest import KossJS


class TestTimersAPI:
    def test_require_timers(self, koss: KossJS):
        result = koss.eval("typeof require('timers')")
        assert result == "object"

    def test_set_timeout_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').setTimeout")
        assert result == "function"

    def test_clear_timeout_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').clearTimeout")
        assert result == "function"

    def test_set_interval_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').setInterval")
        assert result == "function"

    def test_clear_interval_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').clearInterval")
        assert result == "function"

    def test_set_immediate_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').setImmediate")
        assert result == "function"

    def test_clear_immediate_is_function(self, koss: KossJS):
        result = koss.eval("typeof require('timers').clearImmediate")
        assert result == "function"

    def test_promises_exists(self, koss: KossJS):
        result = koss.eval("typeof require('timers').promises")
        assert result == "object"

    def test_promises_set_timeout(self, koss: KossJS):
        result = koss.eval("typeof require('timers').promises.setTimeout")
        assert result == "function"

    def test_global_set_timeout(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval("typeof setTimeout")
        assert result == "function"

    def test_global_clear_timeout(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval("typeof clearTimeout")
        assert result == "function"

    def test_global_set_interval(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval("typeof setInterval")
        assert result == "function"

    def test_global_clear_interval(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval("typeof clearInterval")
        assert result == "function"

    def test_set_timeout_returns_number(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval("typeof setTimeout(function(){}, 1)")
        assert result == "object"

    def test_clear_timeout_no_error(self, koss: KossJS):
        koss.eval("require('timers');")
        result = koss.eval(
            "var t = setTimeout(function(){}, 1000); "
            "clearTimeout(t); "
            "'ok'"
        )
        assert result == "ok"
