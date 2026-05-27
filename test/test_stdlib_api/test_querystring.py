"""Test querystring module — query string parsing and formatting."""

from .conftest import KossJS


class TestQuerystringAPI:
    def test_require_querystring(self, koss: KossJS):
        result = koss.eval("typeof require('querystring')")
        assert result == "object"

    def test_parse_simple(self, koss: KossJS):
        result: dict[str, str] = koss.eval(
            "JSON.stringify(require('querystring').parse('a=1&b=2'))"
        )
        # koss.eval auto-parses JSON, so result is a dict
        assert isinstance(result, dict)
        assert result.get("a") == "1"
        assert result.get("b") == "2"

    def test_parse_empty(self, koss: KossJS):
        result: dict[str, str] = koss.eval(
            "JSON.stringify(require('querystring').parse(''))"
        )
        # koss.eval auto-parses JSON, so result is a dict
        assert isinstance(result, dict) and len(result) == 0

    def test_stringify_simple(self, koss: KossJS):
        result = koss.eval(
            "require('querystring').stringify({a: 1, b: 2})"
        )
        assert "a=1" in result and "b=2" in result

    def test_stringify_empty(self, koss: KossJS):
        result = koss.eval(
            "require('querystring').stringify({})"
        )
        assert result == ""

    def test_encode_equals_decode(self, koss: KossJS):
        result = koss.eval(
            "var qs = require('querystring'); "
            "var obj = {name: 'hello world', x: 42}; "
            "var str = qs.stringify(obj); "
            "var parsed = qs.parse(str); "
            "parsed.name === 'hello world' && parsed.x === '42'"
        )
        assert result == "true"

    def test_decode_alias(self, koss: KossJS):
        result = koss.eval(
            "typeof require('querystring').decode"
        )
        assert result == "function"

    def test_encode_alias(self, koss: KossJS):
        result = koss.eval(
            "typeof require('querystring').encode"
        )
        assert result == "function"

    def test_special_chars(self, koss: KossJS):
        result = koss.eval(
            "var qs = require('querystring'); "
            "var obj = qs.parse('key=' + encodeURIComponent('hello world')); "
            "obj.key"
        )
        assert result == "hello world"

    def test_nested_values(self, koss: KossJS):
        result = koss.eval(
            "var qs = require('querystring'); "
            "var obj = qs.parse('a[0]=x&a[1]=y'); "
            "JSON.stringify(obj)"
        )
        # koss.eval auto-parses JSON, so result is a dict; check values
        assert isinstance(result, dict)
        assert "x" in result.values() and "y" in result.values()
