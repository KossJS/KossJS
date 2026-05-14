import pytest # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS


class TestModuleLoader:
    def test_require_function_exists(self, koss: KossJS):
        assert koss.eval("typeof require") == "function"

    def test_require_path_module(self, koss: KossJS):
        result = koss.eval("require('path')")
        assert result == "[object Object]"

    def test_path_basename(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.basename('/foo/bar.txt')")
        assert result == "bar.txt"

    def test_path_join(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.join('/foo', 'bar', 'baz.txt')")
        assert result == "/foo/bar/baz.txt"

    def test_path_dirname(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.dirname('/foo/bar/baz.txt')")
        assert result == "/foo/bar"

    def test_path_extname(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.extname('/foo/bar.txt')")
        assert result == ".txt"

    def test_path_resolve(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.resolve('/foo', 'bar')")
        assert result == "/foo/bar"

    def test_path_relative(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.relative('/a/b/c', '/a/b/d')")
        assert result == "../d"

    def test_path_normalize(self, koss: KossJS):
        result = koss.eval("var p = require('path'); p.normalize('/a/b/../c/./d')")
        assert result == "/a/c/d"

    def test_path_parse(self, koss: KossJS):
        result = koss.eval("var p = require('path'); JSON.stringify(p.parse('/foo/bar.txt'))")
        assert "bar.txt" in result
        assert ".txt" in result

    def test_require_cache(self, koss: KossJS):
        result = koss.eval("var p1 = require('path'); var p2 = require('path'); p1 === p2")
        assert result == "true"

    def test_require_nonexistent_returns_object(self, koss: KossJS):
        result = koss.eval("require('non_existent_module')")
        assert result == "[object Object]"

    def test_modules_querystring(self, koss: KossJS):
        result = koss.eval("var qs = require('querystring'); typeof qs")
        assert result == "object"

    def test_modules_url(self, koss: KossJS):
        result = koss.eval("var url = require('url'); typeof url")
        assert result == "object"

    def test_modules_assert(self, koss: KossJS):
        result = koss.eval("var a = require('assert'); typeof a")
        assert result == "function", "assert exports a function"

    def test_assert_strict_equal(self, koss: KossJS):
        result = koss.eval("""
        (function() {
            var assert = require('assert');
            assert.strictEqual(1 + 1, 2);
            return 'ok';
        })()
        """)
        assert result == "ok"

    def test_fs_module_exists(self, koss: KossJS):
        result = koss.eval("var fs = require('fs'); typeof fs")
        assert result == "object"

    def test_buffer_module(self, koss: KossJS):
        result = koss.eval("var buf = require('buffer'); typeof buf")
        assert result == "object"

    def test_events_module(self, koss: KossJS):
        result = koss.eval("""
        (function() {
            var events = require('events');
            var emitter = new events.EventEmitter();
            var count = 0;
            emitter.on('event', function() { count++; });
            emitter.emit('event');
            return 'count=' + count;
        })()
        """)
        assert result == "count=1"
