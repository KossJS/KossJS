import pytest # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS


class TestModule:
    def test_require_fs_type(self, koss: KossJS):
        r = koss.eval("typeof require('fs')")
        assert r == "object"

    def test_fs_module_info(self, koss: KossJS):
        r = koss.eval("""
        var m = require('fs');
        var info = {
            type: typeof m,
            isNull: m === null,
            keys: Object.keys(m),
            ownKeys: Object.getOwnPropertyNames(m),
        };
        JSON.stringify(info)
        """)
        assert r["type"] == "object"
        assert r["isNull"] == False
        assert "readFileSync" in r["keys"] or "readFileSync" in r["ownKeys"]

    def test_path_module_info(self, koss: KossJS):
        r = koss.eval("""
        var path = require('path');
        var info = {
            keys: Object.keys(path),
            ownKeys: Object.getOwnPropertyNames(path),
        };
        JSON.stringify(info)
        """)
        assert "join" in r["keys"] or "join" in r["ownKeys"]
        assert "resolve" in r["keys"] or "resolve" in r["ownKeys"]

    def test_basic_require(self, koss: KossJS):
        r = koss.eval("""
        (function() {
            var m = { exports: {} };
            m.exports = { hello: 42, world: function() {} };
            m.exports.extra = true;
            return JSON.stringify({
                keys: Object.keys(m.exports),
                hello: m.exports.hello,
                extra: m.exports.extra
            });
        })()
        """)
        assert r["hello"] == 42
        assert r["extra"] == True
        assert "hello" in r["keys"]
        assert "extra" in r["keys"]

    def test_koss_load_module(self, koss: KossJS):
        r = koss.eval("""
        var result = __koss_load_module('fs');
        JSON.stringify({
            hasResult: typeof result !== 'undefined' && result !== null,
            type: typeof result,
            preview: typeof result === 'string' ? result.substring(0, 200) : String(result)
        })
        """)
        assert r["hasResult"] == True
        assert r["type"] == "string"
