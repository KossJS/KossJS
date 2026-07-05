"""
Tests for Bun-style API compatibility layer (koss:bun).

Covers:
- Module import & flag control
- Bun.version / Bun.build / Bun.env / Bun.argv
- Bun.write / Bun.file (text, json, size, exists, arrayBuffer, stream)
- Bun.sleep / Bun.inspect / Bun.peek / Bun.which
- Bun.randomUUIDv7 / Bun.resolve / Bun.readable
- Bun.serve (TCP)
- Bun.sql / Bun.spawn / Bun.build (not-implemented stubs)
"""

import sys
import os
import tempfile
import shutil
import json

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from kossjs_interface import KossJS, JsError

TMPDIR = tempfile.gettempdir().replace("\\", "/")


def _tmp(name):
    return TMPDIR + "/" + name


def _cleanup(*names):
    for name in names:
        p = os.path.join(tempfile.gettempdir(), name)
        if os.path.isdir(p):
            shutil.rmtree(p, ignore_errors=True)
        elif os.path.isfile(p):
            os.remove(p)


# ─── Flag control ────────────────────────────────────────────────────────────

class TestBunBuiltinFlag:
    def test_bun_flag_constant(self):
        assert KossJS.KOSS_BUILTIN_BUN == (1 << 1)

    def test_bun_import_requires_flag(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_BUN"):
            koss.eval("require('koss:bun')")
        koss.destroy()

    def test_bun_import_with_flag(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("typeof require('koss:bun').version")
        assert result == "string"
        koss.destroy()


# ─── Bun global properties ──────────────────────────────────────────────────

class TestBunProperties:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        yield
        self.koss.destroy()

    def test_version_string(self):
        result = self.koss.eval("require('koss:bun').version")
        assert result == "1.1.42"

    def test_build_value(self):
        result = self.koss.eval("typeof require('koss:bun').build")
        assert result == "function"

    def test_env_is_object(self):
        result = self.koss.eval("typeof require('koss:bun').env")
        assert result == "object"

    def test_argv_is_array(self):
        result = self.koss.eval("Array.isArray(require('koss:bun').argv)")
        assert result == "true" or result is True


# ─── Bun.write / Bun.file ───────────────────────────────────────────────────

class TestBunFileIO:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        _cleanup("bun_test.txt", "bun_json.txt")
        yield
        self.koss.destroy()
        _cleanup("bun_test.txt", "bun_json.txt")

    def test_write_string(self):
        p = _tmp("bun_test.txt")
        self.koss.eval(f"""
            var Bun = require('koss:bun');
            Bun.write('{p}', 'hello bun');
        """)
        assert os.path.exists(p)
        with open(p, "r") as f:
            assert f.read() == "hello bun"

    def test_file_text(self):
        p = _tmp("bun_test.txt")
        with open(p, "w") as f:
            f.write("text content")
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            Bun.file('{p}').text();
        """)
        assert str(result) == "text content"

    def test_file_size(self):
        p = _tmp("bun_test.txt")
        with open(p, "w") as f:
            f.write("12345")
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            Bun.file('{p}').size();
        """)
        assert str(result) == "5"

    def test_file_exists_true(self):
        p = _tmp("bun_test.txt")
        with open(p, "w") as f:
            f.write("x")
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            Bun.file('{p}').exists();
        """)
        assert result == "true" or result is True

    def test_file_exists_false(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.file('/nonexistent_path_999').exists();
        """)
        assert result == "false" or result is False

    def test_file_json(self):
        p = _tmp("bun_json.txt")
        with open(p, "w") as f:
            f.write('{"key":"value","num":42}')
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            var obj = Bun.file('{p}').json();
            obj.key + '_' + obj.num;
        """)
        assert str(result) == "value_42"

    def test_file_path(self):
        p = _tmp("bun_test.txt")
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            Bun.file('{p}').path;
        """)
        assert str(result) == p

    def test_file_arraybuffer(self):
        p = _tmp("bun_test.txt")
        with open(p, "w") as f:
            f.write("abc")
        result = self.koss.eval(f"""
            var Bun = require('koss:bun');
            var ab = Bun.file('{p}').arrayBuffer();
            ab instanceof ArrayBuffer;
        """)
        assert result == "true" or result is True


# ─── Bun.sleep / Bun.inspect / Bun.peek / Bun.which ────────────────────────

class TestBunUtils:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        yield
        self.koss.destroy()

    def test_inspect_object(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            typeof Bun.inspect({a: 1});
        """)
        assert result == "string"

    def test_peek_array(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.peek([42, 99]);
        """)
        assert str(result) == "42"

    def test_peek_empty(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.peek([]);
        """)
        assert result is None or str(result) == "undefined"

    def test_peek_string(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.peek('hello');
        """)
        assert str(result) == "h"

    def test_which_returns_cmd(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.which('node');
        """)
        assert str(result) == "node"

    def test_which_null(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.which(null);
        """)
        assert result is None or str(result) == "null"

    def test_resolve_existing(self):
        result = self.koss.eval("""
            var Bun = require('koss:bun');
            Bun.resolve('/');
        """)
        assert result is not None and str(result) != "undefined"


# ─── Bun.randomUUIDv7 ──────────────────────────────────────────────────────

class TestBunUUID:
    def test_randomuuidv7_format(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("""
            var Bun = require('koss:bun');
            var id = Bun.randomUUIDv7();
            id.length === 36 && id[8] === '-';
        """)
        assert result == "true" or result is True
        koss.destroy()

    def test_randomuuidv7_unique(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("""
            var Bun = require('koss:bun');
            var a = Bun.randomUUIDv7();
            var b = Bun.randomUUIDv7();
            a !== b;
        """)
        assert result == "true" or result is True
        koss.destroy()


# ─── Bun.readable ──────────────────────────────────────────────────────────

class TestBunReadable:
    def test_readable_throws_not_supported(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        with pytest.raises(JsError, match="not supported"):
            koss.eval("""
                var Bun = require('koss:bun');
                Bun.readable('/tmp/test.txt');
            """)
        koss.destroy()


# ─── Bun.serve (TCP) ──────────────────────────────────────────────────────

class TestBunServe:
    def test_serve_exists(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("typeof require('koss:bun').serve")
        assert result == "function"
        koss.destroy()

    def test_serve_throws_ssrf_in_sandbox(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("""
            var Bun = require('koss:bun');
            try {
                Bun.serve({ port: 19876 });
                'no error';
            } catch(e) {
                'error';
            }
        """)
        assert result == "error"
        koss.destroy()


# ─── Bun not-implemented stubs ────────────────────────────────────────────

class TestBunNotImplemented:
    def test_sql_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:bun').sql()")
        koss.destroy()

    def test_spawn_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:bun').spawn()")
        koss.destroy()

    def test_build_function_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:bun').build({})")
        koss.destroy()
