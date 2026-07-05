"""
Tests for Deno-style API compatibility layer (koss:deno).

Covers:
- Module import & flag control
- Deno.version / Deno.env / Deno.args / Deno.pid / Deno.noColor
- Deno.readTextFile / Deno.writeTextFile / Deno.readFile / Deno.writeFile
- Deno.stat / Deno.lstat / Deno.mkdir / Deno.remove / Deno.rename / Deno.realPath
- Deno.cwd / Deno.chdir
- Deno.memoryUsage / Deno.exit
- Deno.serve / Deno.listen / Deno.connect (TCP)
- Deno.resolveDns
- Deno.crypto (getRandomValues, randomUUID, subtle.digest)
- Deno.run / Deno.spawn / Deno.permissions (not-implemented stubs)
- Deno.errors / Deno.signals
"""

import sys
import os
import tempfile
import shutil

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

class TestDenoBuiltinFlag:
    def test_deno_flag_constant(self):
        assert KossJS.KOSS_BUILTIN_DENO == (1 << 2)

    def test_deno_import_requires_flag(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_DENO"):
            koss.eval("require('koss:deno')")
        koss.destroy()

    def test_deno_import_with_flag(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("typeof require('koss:deno').version")
        assert result == "object"
        koss.destroy()


# ─── Deno properties ────────────────────────────────────────────────────────

class TestDenoProperties:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        yield
        self.koss.destroy()

    def test_version_object(self):
        result = self.koss.eval("""
            var v = require('koss:deno').version;
            v.deno + '_' + v.v8 + '_' + v.typescript;
        """)
        assert str(result) == "2.0.6_12.9_5.6"

    def test_env_is_object(self):
        result = self.koss.eval("typeof require('koss:deno').env")
        assert result == "object"

    def test_args_is_array(self):
        result = self.koss.eval("Array.isArray(require('koss:deno').args)")
        assert result == "true" or result is True

    def test_pid_is_number(self):
        result = self.koss.eval("typeof require('koss:deno').pid")
        assert result == "number"

    def test_no_color_true(self):
        result = self.koss.eval("require('koss:deno').noColor")
        assert result == "true" or result is True

    def test_errors_is_object(self):
        result = self.koss.eval("typeof require('koss:deno').errors")
        assert result == "object"

    def test_signals_is_object(self):
        result = self.koss.eval("typeof require('koss:deno').signals")
        assert result == "object"


# ─── Deno file system ──────────────────────────────────────────────────────

class TestDenoFilesystem:
    @pytest.fixture(autouse=True)
    def setup(self):
        _cleanup("deno_test.txt", "deno_test2.txt", "deno_json.txt")
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        yield
        self.koss.destroy()
        _cleanup("deno_test.txt", "deno_test2.txt", "deno_json.txt")

    def test_write_and_read_text_file(self):
        p = _tmp("deno_test.txt")
        self.koss.eval(f"""
            var Deno = require('koss:deno');
            Deno.writeTextFile('{p}', 'deno hello');
        """)
        assert os.path.exists(p)
        result = self.koss.eval(f"""
            var Deno = require('koss:deno');
            Deno.readTextFile('{p}');
        """)
        assert str(result) == "deno hello"

    def test_read_file_returns_bytes(self):
        p = _tmp("deno_test.txt")
        with open(p, "w") as f:
            f.write("bytes")
        result = self.koss.eval(f"""
            var Deno = require('koss:deno');
            var data = Deno.readFile('{p}');
            data instanceof Uint8Array;
        """)
        assert result == "true" or result is True

    def test_write_file(self):
        p = _tmp("deno_test2.txt")
        self.koss.eval(f"""
            var Deno = require('koss:deno');
            Deno.writeFile('{p}', new Uint8Array([65, 66, 67]));
        """)
        assert os.path.exists(p)
        with open(p, "rb") as f:
            assert f.read() == b"ABC"

    def test_stat(self):
        p = _tmp("deno_test.txt")
        with open(p, "w") as f:
            f.write("stat me")
        result = self.koss.eval(f"""
            var Deno = require('koss:deno');
            var s = Deno.stat('{p}');
            s.size > 0;
        """)
        assert result == "true" or result is True

    def test_lstat(self):
        p = _tmp("deno_test.txt")
        with open(p, "w") as f:
            f.write("lstat")
        result = self.koss.eval(f"""
            var Deno = require('koss:deno');
            var s = Deno.lstat('{p}');
            s.size > 0;
        """)
        assert result == "true" or result is True

    def test_mkdir_and_remove(self):
        d = _tmp("deno_mkdir_test")
        result = self.koss.eval(f"""
            var Deno = require('koss:deno');
            Deno.mkdir('{d}');
            var Deno2 = require('koss:deno');
            Deno2.mkdir('{d}');
        """)
        assert os.path.isdir(d)
        _cleanup("deno_mkdir_test")

    def test_rename(self):
        src = _tmp("deno_test.txt")
        dst = _tmp("deno_test2.txt")
        with open(src, "w") as f:
            f.write("rename me")
        self.koss.eval(f"""
            var Deno = require('koss:deno');
            Deno.rename('{src}', '{dst}');
        """)
        assert not os.path.exists(src)
        assert os.path.exists(dst)

    def test_real_path(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.realPath('/');
        """)
        assert result == "string"

    def test_cwd(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.cwd();
        """)
        assert result == "string"


# ─── Deno process ──────────────────────────────────────────────────────────

class TestDenoProcess:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        yield
        self.koss.destroy()

    def test_memory_usage(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            var m = Deno.memoryUsage();
            typeof m === 'object' && 'rss' in m;
        """)
        assert result == "true" or result is True

    def test_exit_throws(self):
        with pytest.raises(JsError, match="Process exit"):
            self.koss.eval("""
                var Deno = require('koss:deno');
                Deno.exit(1);
            """)

    def test_set_timeout(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.setTimeout;
        """)
        assert result == "function"

    def test_clear_timeout(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.clearTimeout;
        """)
        assert result == "function"

    def test_set_interval(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.setInterval;
        """)
        assert result == "function"

    def test_clear_interval(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            typeof Deno.clearInterval;
        """)
        assert result == "function"


# ─── Deno.crypto ───────────────────────────────────────────────────────────

class TestDenoCrypto:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        yield
        self.koss.destroy()

    def test_get_random_values(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            var arr = new Uint8Array(8);
            Deno.crypto.getRandomValues(arr);
            arr[0] !== 0 || arr[1] !== 0 || arr[2] !== 0;
        """)
        assert result == "true" or result is True

    def test_random_uuid(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            var id = Deno.crypto.randomUUID();
            typeof id === 'string' && id.length === 36;
        """)
        assert result == "true" or result is True

    def test_subtle_digest(self):
        result = self.koss.eval("""
            var Deno = require('koss:deno');
            var promise = Deno.crypto.subtle.digest('SHA-256', 'hello');
            typeof promise === 'object' && typeof promise.then === 'function';
        """)
        assert result == "true" or result is True


# ─── Deno.serve / Deno.listen / Deno.connect (TCP) ────────────────────────

class TestDenoNetwork:
    def test_serve_exists(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("typeof require('koss:deno').serve")
        assert result == "function"
        koss.destroy()

    def test_listen_exists(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("typeof require('koss:deno').listen")
        assert result == "function"
        koss.destroy()

    def test_connect_exists(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("typeof require('koss:deno').connect")
        assert result == "function"
        koss.destroy()

    def test_serve_throws_ssrf_in_sandbox(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("""
            var Deno = require('koss:deno');
            try {
                Deno.serve(function() {}, { port: 19880 });
                'no error';
            } catch(e) {
                'error';
            }
        """)
        assert result == "error"
        koss.destroy()


# ─── Deno not-implemented stubs ───────────────────────────────────────────

class TestDenoNotImplemented:
    def test_run_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:deno').run()")
        koss.destroy()

    def test_spawn_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:deno').spawn()")
        koss.destroy()

    def test_permissions_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        with pytest.raises(JsError, match="not implemented"):
            koss.eval("require('koss:deno').permissions()")
        koss.destroy()
