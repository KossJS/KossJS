"""Test Node/Bun/Deno compatibility layer additions: timers/promises, stream/promises,
stream/consumers, console module, util.types, Bun utilities, Deno sync/Env APIs."""

import pytest
from kossjs_interface import KossJS


def _true(koss: KossJS, code: str):
    result = koss.eval(code)
    return str(result).strip() == "true"


class TestNodeModules:
    """New Node.js builtin modules."""

    @pytest.fixture(autouse=True)
    def _setup(self):
        self.koss = KossJS(capabilities=KossJS.KOSS_CAP_ALL, builtins=KossJS.KOSS_BUILTIN_ALL)
        yield
        self.koss.destroy()

    def test_timers_promises(self):
        assert _true(self.koss, """
        var t = require('timers/promises');
        typeof t.setTimeout === 'function' && typeof t.setImmediate === 'function' &&
        typeof t.scheduler === 'object';
        """)

    def test_timers_promises_settimeout(self):
        assert _true(self.koss, """
        var t = require('timers/promises');
        typeof t.setTimeout(1, 'x').then === 'function';
        """)

    def test_stream_promises(self):
        assert _true(self.koss, """
        var s = require('stream/promises');
        typeof s.pipeline === 'function' && typeof s.finished === 'function';
        """)

    def test_stream_consumers(self):
        assert _true(self.koss, """
        var c = require('stream/consumers');
        typeof c.text === 'function' && typeof c.json === 'function' &&
        typeof c.buffer === 'function' && typeof c.blob === 'function';
        """)

    def test_console_module(self):
        assert _true(self.koss, """
        var c = require('console');
        typeof c.log === 'function' && typeof c.error === 'function';
        """)

    def test_util_types_bigint(self):
        assert _true(self.koss, """
        var util = require('util');
        util.types.isBigInt64Array(new BigInt64Array(2)) &&
        util.types.isBigUint64Array(new BigUint64Array(2)) &&
        util.types.isUint8Array(new Uint8Array(1)) &&
        util.types.isDataView(new DataView(new ArrayBuffer(4)));
        """)

    def test_util_types_collections(self):
        assert _true(self.koss, """
        var util = require('util');
        util.types.isMap(new Map()) && util.types.isSet(new Set()) &&
        util.types.isWeakMap(new WeakMap()) && util.types.isPromise(Promise.resolve(1));
        """)


class TestBunCompat:
    """Bun compatibility layer additions."""

    @pytest.fixture(autouse=True)
    def _setup(self):
        self.koss = KossJS(capabilities=KossJS.KOSS_CAP_ALL, builtins=KossJS.KOSS_BUILTIN_BUN + KossJS.KOSS_BUILTIN_KOSS)
        yield
        self.koss.destroy()

    def test_bun_gzip_sync(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var zlib = require('koss:zlib');
        var gz = Bun.gzipSync(new TextEncoder().encode('bun gzip'));
        var back = zlib.gunzipSync(gz);
        new TextDecoder().decode(back) === 'bun gzip';
        """)

    def test_bun_nanoseconds(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        typeof Bun.nanoseconds() === 'number' && Bun.nanoseconds() > 0;
        """)

    def test_bun_deep_equals(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        Bun.deepEquals({a:1,b:[1,2]}, {a:1,b:[1,2]}) && !Bun.deepEquals({a:1}, {a:2});
        """)

    def test_bun_escape_html(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        Bun.escapeHTML('<b>&quot;x&quot;</b>') === '&lt;b&gt;&amp;quot;x&amp;quot;&lt;/b&gt;';
        """)

    def test_bun_string_width(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        Bun.stringWidth('abc') === 3;
        """)

    def test_bun_glob(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var g = new Bun.Glob('*.js');
        g.match('app.js') && !g.match('app.ts');
        """)

    def test_bun_cookie_map(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var cm = new Bun.CookieMap('a=1; b=2');
        cm.get('a') === '1' && cm.get('b') === '2';
        """)

    def test_bun_cookie_to_string(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var c = new Bun.Cookie('sid', 'abc', { path: '/', httpOnly: true });
        String(c).indexOf('sid=abc') !== -1 && String(c).indexOf('HttpOnly') !== -1;
        """)

    def test_bun_concat_array_buffers(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var a = new Uint8Array([1,2]).buffer;
        var b = new Uint8Array([3,4]).buffer;
        var out = new Uint8Array(Bun.concatArrayBuffers([a, b]));
        out.length === 4 && out[0] === 1 && out[3] === 4;
        """)

    def test_bun_file_url_path(self):
        assert _true(self.koss, """
        var Bun = require('koss:bun');
        var p = Bun.fileURLToPath('file:///tmp/test.txt');
        p.indexOf('tmp') !== -1;
        """)


class TestDenoCompat:
    """Deno compatibility layer additions."""

    @pytest.fixture(autouse=True)
    def _setup(self):
        self.koss = KossJS(capabilities=KossJS.KOSS_CAP_ALL, builtins=KossJS.KOSS_BUILTIN_DENO + KossJS.KOSS_BUILTIN_KOSS)
        yield
        self.koss.destroy()

    def test_deno_env(self):
        assert _true(self.koss, """
        var Deno = require('koss:deno');
        typeof Deno.Env.get === 'function' && typeof Deno.Env.set === 'function' &&
        typeof Deno.Env.delete === 'function';
        """)

    def test_deno_sync_fs(self):
        assert _true(self.koss, """
        var Deno = require('koss:deno');
        Deno.writeTextFileSync('_deno_sync.txt', 'sync data');
        Deno.readTextFileSync('_deno_sync.txt') === 'sync data' &&
        typeof Deno.statSync('_deno_sync.txt') === 'object';
        """)

    def test_deno_real_path(self):
        assert _true(self.koss, """
        var Deno = require('koss:deno');
        var p = Deno.realPath('.');
        typeof p === 'string' && p.length > 0;
        """)

    def test_deno_kill_exists(self):
        assert _true(self.koss, """
        var Deno = require('koss:deno');
        typeof Deno.kill === 'function';
        """)
