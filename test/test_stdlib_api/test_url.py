"""Test url module - URL parsing, formatting and URLSearchParams."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


class TestUrlAPI:
    def test_require_url(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/url')")
        assert result == "object"

    def test_parse(self, koss: KossJS):
        result = koss.eval(
            "var u = require('koss:node/url').parse('https://example.com/path?q=1'); "
            "u.protocol + '//' + u.host + u.pathname"
        )
        assert "https://" in result

    def test_format(self, koss: KossJS):
        result = koss.eval(
            "require('koss:node/url').format({protocol: 'https:', host: 'example.com', pathname: '/a'})"
        )
        assert "https://" in result and "example.com" in result

    def test_resolve(self, koss: KossJS):
        result = koss.eval(
            "require('koss:node/url').resolve('https://example.com/a', '/b')"
        )
        assert "example.com" in result

    def test_url_constructor(self, koss: KossJS):
        result = koss.eval(
            "typeof require('koss:node/url').URL"
        )
        assert result == "function"

    def test_url_search_params(self, koss: KossJS):
        result = koss.eval(
            "var u = new (require('koss:node/url').URL)('https://example.com?a=1&b=2'); "
            "Array.from(u.searchParams.entries()).length"
        )
        assert result == "2"

    def test_url_href(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com/path').href"
        )
        assert result == "https://x.com/path"

    def test_url_protocol(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com').protocol"
        )
        assert result == "https:"

    def test_url_hostname(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://example.com:8080/a').hostname"
        )
        assert result == "example.com"

    def test_url_port(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://example.com:8080').port"
        )
        assert result == "8080"

    def test_url_pathname(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com/a/b/c?q=1').pathname"
        )
        assert result == "/a/b/c"

    def test_url_search(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com?a=1').search"
        )
        assert "a=1" in result

    def test_url_hash(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com#section').hash"
        )
        assert "#section" in result

    def test_url_search_params_get(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com?name=test').searchParams.get('name')"
        )
        assert result == "test"

    def test_url_search_params_set(self, koss: KossJS):
        result = koss.eval(
            "var u = new (require('koss:node/url').URL)('https://x.com'); "
            "u.searchParams.set('key', 'val'); "
            "u.search"
        )
        assert "key=val" in result

    def test_url_search_params_append(self, koss: KossJS):
        result = koss.eval(
            "var u = new (require('koss:node/url').URL)('https://x.com?a=1'); "
            "u.searchParams.append('a', '2'); "
            "Array.from(u.searchParams.getAll('a'))"
        )
        assert "1" in result and "2" in result

    def test_url_search_params_delete(self, koss: KossJS):
        result = koss.eval(
            "var u = new (require('koss:node/url').URL)('https://x.com?a=1&b=2'); "
            "u.searchParams.delete('a'); "
            "u.searchParams.has('a')"
        )
        assert result == "false"

    def test_url_search_params_to_string(self, koss: KossJS):
        result = koss.eval(
            "new (require('koss:node/url').URL)('https://x.com?a=1&b=2').searchParams.toString()"
        )
        assert "a=1" in result and "b=2" in result

    def test_domain_to_ascii(self, koss: KossJS):
        result = koss.eval(
            "require('koss:node/url').domainToASCII('example.com')"
        )
        assert result == "example.com"
