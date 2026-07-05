"""
Tests for the Koss native API modules (koss:io, koss:crypto, koss:system, koss:data, koss:ffi, koss:worker).

Tests cover:
1. KOSS_BUILTIN_KOSS flag control
2. koss:io - file and network operations
3. koss:crypto - hash, hmac, random, uuid
4. koss:system - platform info
5. koss:data - buffer and encoding
6. koss:ffi - foreign function interface
7. koss:worker - worker thread pool
"""

import sys
import os
import tempfile
import shutil
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from kossjs_interface import KossJS

TMPDIR = tempfile.gettempdir().replace('\\', '/')


def tmpfile(name):
    return TMPDIR + '/' + name


def cleanup(*names):
    for name in names:
        p = os.path.join(tempfile.gettempdir(), name)
        if os.path.isdir(p):
            shutil.rmtree(p, ignore_errors=True)
        elif os.path.isfile(p):
            os.remove(p)


class TestKossBuiltinFlag:
    """Test KOSS_BUILTIN_KOSS flag."""

    def test_koss_builtin_constant(self):
        assert KossJS.KOSS_BUILTIN_KOSS == (1 << 3)

    def test_koss_builtin_flag_required(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_KOSS"):
            koss.eval("require('koss:io')")
        koss.destroy()

    def test_koss_builtin_flag_enabled(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS)
        result = koss.eval("typeof require('koss:io').readText")
        assert result == "function"
        koss.destroy()

    def test_koss_builtin_with_all(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("typeof require('koss:io').readText")
        assert result == "function"
        koss.destroy()

    def test_is_builtin_enabled_koss(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS)
        assert koss.is_builtin_enabled(KossJS.KOSS_BUILTIN_KOSS) is True
        assert koss.is_builtin_enabled(KossJS.KOSS_BUILTIN_NODE) is False
        koss.destroy()


class TestKossIo:
    """Test koss:io module - file and network operations."""

    def setup_method(self):
        cleanup('koss_test_native.txt', 'koss_stat_test.txt',
                'koss_list_test', 'koss_cp_src.txt', 'koss_cp_dst.txt',
                'koss_mv_dst.txt', 'koss_mkdir_test', 'koss_writetext_test.txt')
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)

    def teardown_method(self):
        self.koss.destroy()
        cleanup('koss_test_native.txt', 'koss_stat_test.txt',
                'koss_list_test', 'koss_cp_src.txt', 'koss_cp_dst.txt',
                'koss_mv_dst.txt', 'koss_mkdir_test', 'koss_writetext_test.txt')

    def test_read_text(self):
        result = self.koss.eval("typeof require('koss:io').readText")
        assert result == "function"

    def test_write_and_read(self):
        p = tmpfile('koss_test_native.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.write('{p}', 'hello koss');
        var content = io.readText('{p}');
        io.rm('{p}');
        content;
        """)
        assert str(result) == 'hello koss'

    def test_write_text(self):
        p = tmpfile('koss_writetext_test.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.writeText('{p}', 'writetext test');
        io.readText('{p}');
        """)
        assert str(result) == 'writetext test'

    def test_read_returns_uint8array(self):
        p = tmpfile('koss_test_native.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.write('{p}', 'bytes');
        var data = io.read('{p}');
        io.rm('{p}');
        data instanceof Uint8Array;
        """)
        assert result == 'true' or result is True

    def test_stat(self):
        p = tmpfile('koss_stat_test.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.write('{p}', 'test data');
        var s = io.stat('{p}');
        io.rm('{p}');
        s.size > 0 && s.isFile;
        """)
        assert result == 'true' or result is True

    def test_stat_has_fields(self):
        p = tmpfile('koss_stat_test.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.write('{p}', 'data');
        var s = io.stat('{p}');
        io.rm('{p}');
        'size' in s && 'mtime' in s && 'isFile' in s && 'isDir' in s && 'isSymlink' in s;
        """)
        assert result == 'true' or result is True

    def test_list(self):
        d = tmpfile('koss_list_test')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.mkdir('{d}');
        io.write('{d}/a.txt', 'a');
        io.write('{d}/b.txt', 'b');
        var entries = io.list('{d}');
        io.rm('{d}/a.txt');
        io.rm('{d}/b.txt');
        io.rm('{d}');
        entries.length;
        """)
        assert str(result) == '2'

    def test_exists(self):
        result = self.koss.eval("""
        var io = require('koss:io');
        io.exists('/');
        """)
        assert result == 'true' or result is True

    def test_cp_mv(self):
        src = tmpfile('koss_cp_src.txt')
        dst = tmpfile('koss_cp_dst.txt')
        mv = tmpfile('koss_mv_dst.txt')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.write('{src}', 'copy me');
        io.cp('{src}', '{dst}');
        var c1 = io.readText('{dst}');
        io.mv('{dst}', '{mv}');
        var c2 = io.readText('{mv}');
        io.rm('{src}');
        io.rm('{mv}');
        c1 + '|' + c2;
        """)
        assert str(result) == 'copy me|copy me'

    def test_mkdir(self):
        d = tmpfile('koss_mkdir_test')
        result = self.koss.eval(f"""
        var io = require('koss:io');
        io.mkdir('{d}/nested/deep', {{ recursive: true }});
        var ok = io.exists('{d}/nested/deep');
        io.rm('{d}', {{ recursive: true }});
        ok;
        """)
        assert result == 'true' or result is True


class TestKossCrypto:
    """Test koss:crypto module - hash, hmac, random, uuid, sign, verify, encrypt, decrypt, pbkdf2."""

    def setup_method(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)

    def teardown_method(self):
        self.koss.destroy()

    def test_hash(self):
        result = self.koss.eval("""
        var h = require('koss:crypto').hash('sha256', 'hello');
        h.length === 64;
        """)
        assert result == 'true' or result is True

    def test_hash_sha1(self):
        result = self.koss.eval("""
        var h = require('koss:crypto').hash('sha1', 'hello');
        h.length === 40;
        """)
        assert result == 'true' or result is True

    def test_hash_md5(self):
        result = self.koss.eval("""
        var h = require('koss:crypto').hash('md5', 'hello');
        h.length === 32;
        """)
        assert result == 'true' or result is True

    def test_hash_deterministic(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var h1 = c.hash('sha256', 'test');
        var h2 = c.hash('sha256', 'test');
        h1 === h2;
        """)
        assert result == 'true' or result is True

    def test_hmac(self):
        result = self.koss.eval("""
        var h = require('koss:crypto').hmac('sha256', 'secret', 'message');
        typeof h === 'string' && h.length > 0;
        """)
        assert result == 'true' or result is True

    def test_hmac_deterministic(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var h1 = c.hmac('sha256', 'key', 'msg');
        var h2 = c.hmac('sha256', 'key', 'msg');
        h1 === h2;
        """)
        assert result == 'true' or result is True

    def test_random_bytes(self):
        result = self.koss.eval("""
        var bytes = require('koss:crypto').randomBytes(16);
        bytes instanceof Uint8Array && bytes.length === 16;
        """)
        assert result == 'true' or result is True

    def test_random_bytes_default(self):
        result = self.koss.eval("""
        var bytes = require('koss:crypto').randomBytes();
        bytes instanceof Uint8Array && bytes.length === 32;
        """)
        assert result == 'true' or result is True

    def test_random_bytes_unique(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var a = c.randomBytes(8);
        var b = c.randomBytes(8);
        var diff = false;
        for (var i = 0; i < a.length; i++) { if (a[i] !== b[i]) diff = true; }
        diff;
        """)
        assert result == 'true' or result is True

    def test_uuid(self):
        result = self.koss.eval("""
        var id = require('koss:crypto').uuid();
        typeof id === 'string' && id.length === 36;
        """)
        assert result == 'true' or result is True

    def test_uuid_unique(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        c.uuid() !== c.uuid();
        """)
        assert result == 'true' or result is True

    def test_algorithms_list(self):
        result = self.koss.eval("""
        var a = require('koss:crypto').algorithms;
        Array.isArray(a) && a.includes('sha256') && a.includes('sha1') && a.includes('md5');
        """)
        assert result == 'true' or result is True

    def test_sign_returns_bytes(self):
        result = self.koss.eval("""
        var sig = require('koss:crypto').sign('mykey', 'data');
        sig instanceof Uint8Array && sig.length > 0;
        """)
        assert result == 'true' or result is True

    def test_verify_valid(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var sig = c.sign('key', 'msg');
        c.verify('key', 'msg', sig);
        """)
        assert result == 'true' or result is True

    def test_verify_tampered(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var sig = c.sign('key', 'msg');
        var tampered = new Uint8Array(sig.length);
        for (var i = 0; i < sig.length; i++) tampered[i] = sig[i] ^ 0xff;
        !c.verify('key', 'msg', tampered);
        """)
        assert result == 'true' or result is True

    def test_encrypt_returns_bytes(self):
        result = self.koss.eval("""
        var enc = require('koss:crypto').encrypt('sha256', 'key', 'plain');
        enc instanceof Uint8Array && enc.length > 0;
        """)
        assert result == 'true' or result is True

    def test_decrypt_equals_encrypt(self):
        result = self.koss.eval("""
        var c = require('koss:crypto');
        var enc = c.encrypt('sha256', 'k', 'data');
        var dec = c.decrypt('sha256', 'k', 'data');
        enc.length === dec.length;
        """)
        assert result == 'true' or result is True


class TestKossSystem:
    """Test koss:system module - platform info, hostname, pid, cwd, versions, cpus, etc."""

    def setup_method(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)

    def teardown_method(self):
        self.koss.destroy()

    def test_arch(self):
        result = self.koss.eval("""
        var a = require('koss:system').arch();
        typeof a === 'string' && a.length > 0;
        """)
        assert result == 'true' or result is True

    def test_platform(self):
        result = self.koss.eval("""
        var p = require('koss:system').platform();
        typeof p === 'string' && p.length > 0;
        """)
        assert result == 'true' or result is True

    def test_memory(self):
        result = self.koss.eval("""
        var m = require('koss:system').memory();
        typeof m === 'object' && 'total' in m && 'free' in m && 'used' in m;
        """)
        assert result == 'true' or result is True

    def test_version(self):
        result = self.koss.eval("typeof require('koss:system').version() === 'string'")
        assert result == 'true' or result is True

    def test_env(self):
        result = self.koss.eval("typeof require('koss:system').env() === 'object'")
        assert result == 'true' or result is True

    def test_hostname(self):
        result = self.koss.eval("""
        var h = require('koss:system').hostname();
        typeof h === 'string' && h.length > 0;
        """)
        assert result == 'true' or result is True

    def test_pid(self):
        result = self.koss.eval("""
        var p = require('koss:system').pid();
        typeof p === 'number';
        """)
        assert result == 'true' or result is True

    def test_cwd(self):
        result = self.koss.eval("""
        var c = require('koss:system').cwd();
        typeof c === 'string' && c.length > 0;
        """)
        assert result == 'true' or result is True

    def test_versions(self):
        result = self.koss.eval("""
        var v = require('koss:system').versions();
        typeof v === 'object';
        """)
        assert result == 'true' or result is True

    def test_cpus(self):
        result = self.koss.eval("""
        var c = require('koss:system').cpus();
        Array.isArray(c);
        """)
        assert result == 'true' or result is True

    def test_loadavg(self):
        result = self.koss.eval("""
        var l = require('koss:system').loadavg();
        Array.isArray(l) && l.length === 3;
        """)
        assert result == 'true' or result is True

    def test_uptime(self):
        result = self.koss.eval("""
        var u = require('koss:system').uptime();
        typeof u === 'number';
        """)
        assert result == 'true' or result is True

    def test_env_get_key(self):
        result = self.koss.eval("""
        var s = require('koss:system');
        var obj = s.env();
        typeof obj === 'object';
        """)
        assert result == 'true' or result is True


class TestKossData:
    """Test koss:data module - buffer, encoding, hex, base64."""

    def setup_method(self):
        self.koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)

    def teardown_method(self):
        self.koss.destroy()

    def test_encode_decode(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var bytes = d.encode('hello');
        var text = d.decode(bytes);
        text;
        """)
        assert str(result) == 'hello'

    def test_encode_returns_uint8array(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        d.encode('abc') instanceof Uint8Array;
        """)
        assert result == 'true' or result is True

    def test_decode_returns_string(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        typeof d.decode(new Uint8Array([65, 66]));
        """)
        assert result == 'string'

    def test_concat(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var a = d.encode('hello');
        var b = d.encode(' world');
        var c = d.concat(a, b);
        d.decode(c);
        """)
        assert str(result) == 'hello world'

    def test_concat_empty(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var c = d.concat();
        c.length === 0;
        """)
        assert result == 'true' or result is True

    def test_compare(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var a = d.encode('abc');
        var b = d.encode('abd');
        d.compare(a, b) < 0;
        """)
        assert result == 'true' or result is True

    def test_compare_equal(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var a = d.encode('same');
        var b = d.encode('same');
        d.compare(a, b) === 0;
        """)
        assert result == 'true' or result is True

    def test_compare_greater(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var a = d.encode('b');
        var b = d.encode('a');
        d.compare(a, b) > 0;
        """)
        assert result == 'true' or result is True

    def test_isEqual(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var a = d.encode('test');
        var b = d.encode('test');
        var c = d.encode('other');
        d.isEqual(a, b) && !d.isEqual(a, c);
        """)
        assert result == 'true' or result is True

    def test_hex(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        d.toHex(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
        """)
        assert str(result) == 'deadbeef'

    def test_from_hex(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var bytes = d.fromHex('deadbeef');
        bytes.length === 4 && bytes[0] === 0xde && bytes[1] === 0xad && bytes[2] === 0xbe && bytes[3] === 0xef;
        """)
        assert result == 'true' or result is True

    def test_hex_roundtrip(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var original = new Uint8Array([1, 2, 3, 4, 5]);
        var hex = d.toHex(original);
        var restored = d.fromHex(hex);
        d.isEqual(original, restored);
        """)
        assert result == 'true' or result is True

    def test_base64(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        d.toBase64(new Uint8Array([72, 101, 108, 108, 111]));
        """)
        assert str(result) == 'SGVsbG8='

    def test_from_base64(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var bytes = d.fromBase64('SGVsbG8=');
        bytes.length === 5 && bytes[0] === 72 && bytes[1] === 101;
        """)
        assert result == 'true' or result is True

    def test_base64_roundtrip(self):
        result = self.koss.eval("""
        var d = require('koss:data');
        var original = d.encode('roundtrip test');
        var b64 = d.toBase64(original);
        var restored = d.fromBase64(b64);
        d.decode(restored);
        """)
        assert str(result) == 'roundtrip test'


class TestKossFfi:
    """Test koss:ffi module - foreign function interface."""

    def test_ffi_module_importable(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("typeof require('koss:ffi').open === 'function'")
        assert result == 'true' or result is True
        koss.destroy()

    def test_ffi_has_all_exports(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("""
        var ffi = require('koss:ffi');
        typeof ffi.open === 'function' &&
        typeof ffi.dlopen === 'function' &&
        typeof ffi.malloc === 'function' &&
        typeof ffi.free === 'function' &&
        typeof ffi.addressOf === 'function' &&
        typeof ffi.createCallback === 'function' &&
        typeof ffi.strerror === 'function';
        """)
        assert result == 'true' or result is True
        koss.destroy()

    def test_ffi_dlopen_alias(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("""
        var ffi = require('koss:ffi');
        typeof ffi.dlopen === 'function' && typeof ffi.open === 'function';
        """)
        assert result == 'true' or result is True
        koss.destroy()

    def test_ffi_open_nonexistent_throws(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL,
                      stable=False)
        from kossjs_interface import JsError
        with pytest.raises(JsError):
            koss.eval("require('koss:ffi').open('/nonexistent_lib.so')")
        koss.destroy()

    def test_ffi_strerror(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("""
        typeof require('koss:ffi').strerror;
        """)
        assert result == 'function'
        koss.destroy()


class TestKossWorker:
    """Test koss:worker module - worker thread pool."""

    def test_worker_module_importable(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("typeof require('koss:worker').createPool === 'function'")
        assert result == 'true' or result is True
        koss.destroy()

    def test_worker_has_all_exports(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("""
        var w = require('koss:worker');
        typeof w.createPool === 'function' &&
        typeof w.post === 'function' &&
        typeof w.receive === 'function' &&
        typeof w.terminate === 'function';
        """)
        assert result == 'true' or result is True
        koss.destroy()

    def test_worker_create_pool(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_KOSS | KossJS.KOSS_BUILTIN_ALL,
                      capabilities=KossJS.KOSS_CAP_ALL, stable=False)
        result = koss.eval("""
        var pool = require('koss:worker').createPool(2);
        typeof pool === 'object' &&
        typeof pool.execute === 'function' &&
        typeof pool.post === 'function' &&
        typeof pool.receive === 'function' &&
        typeof pool.terminate === 'function' &&
        typeof pool.shutdown === 'function';
        """)
        assert result == 'true' or result is True
        koss.destroy()
