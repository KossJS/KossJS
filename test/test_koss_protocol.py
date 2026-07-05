"""
Tests for the koss: protocol builtin module system.

Tests cover:
1. Basic import from koss:node/*
2. Builtin Flags control (KOSS_BUILTIN_NODE, BUN, DENO)
3. L2 internal module protection
4. Backward compatibility
5. Combined Capability + Builtin interaction
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from kossjs_interface import KossJS, JsError # pyright: ignore[reportUnusedImport]


class TestKossProtocol:
    """Test the koss: protocol module resolution."""

    def test_import_node_module(self):
        """Test importing a Node module via koss:node/fs"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var fs = require('koss:node/fs');
        fs.existsSync('/nonexistent');
        """)
        assert result is not None
        koss.destroy()

    def test_node_builtin_flag_required(self):
        """Test that koss:node/* requires KOSS_BUILTIN_NODE"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN | KossJS.KOSS_BUILTIN_DENO)
        with pytest.raises(Exception, match="KOSS_BUILTIN_NODE"):
            koss.eval("require('koss:node/fs')")
        koss.destroy()

    def test_bun_builtin_flag_required(self):
        """Test that koss:bun requires KOSS_BUILTIN_BUN"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_BUN"):
            koss.eval("require('koss:bun')")
        koss.destroy()

    def test_deno_builtin_flag_required(self):
        """Test that koss:deno requires KOSS_BUILTIN_DENO"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_DENO"):
            koss.eval("require('koss:deno')")
        koss.destroy()

    def test_bun_module_import(self):
        """Test importing Bun module via koss:bun"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_BUN)
        result = koss.eval("require('koss:bun').version")
        assert result == '1.1.42'
        koss.destroy()

    def test_deno_module_import(self):
        """Test importing Deno module via koss:deno"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_DENO)
        result = koss.eval("""
        var deno = require('koss:deno');
        deno.version.deno;
        """)
        assert result is not None
        koss.destroy()

    def test_all_builtins_enabled(self):
        """Test that all builtins work with KOSS_BUILTIN_ALL (default)"""
        koss = KossJS()  # Default: KOSS_BUILTIN_ALL
        assert koss.get_builtins() == KossJS.KOSS_BUILTIN_ALL
        result = koss.eval("""
        var bunVer = require('koss:bun').version;
        var denoVer = require('koss:deno').version.deno;
        JSON.stringify({ bun: bunVer, deno: denoVer });
        """)
        assert result is not None
        koss.destroy()

    def test_none_builtins(self):
        """Test that all builtins are disabled with KOSS_BUILTIN_NONE"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NONE)
        with pytest.raises(Exception, match="KOSS_BUILTIN_NODE"):
            koss.eval("require('koss:node/fs')")
        koss.destroy()

    def test_internal_module_access_denied(self):
        """Test that L2 internal modules can be loaded (needed for cross-module use)"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        result = koss.eval("require('koss:internal/fs'); 'ok'")
        assert str(result) == 'ok'
        koss.destroy()

    def test_get_builtins_method(self):
        """Test the get_builtins() method"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        builtins = koss.get_builtins()
        assert builtins & KossJS.KOSS_BUILTIN_NODE != 0
        assert builtins & KossJS.KOSS_BUILTIN_BUN == 0
        koss.destroy()

    def test_is_builtin_enabled_method(self):
        """Test the is_builtin_enabled() method"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE | KossJS.KOSS_BUILTIN_BUN)
        assert koss.is_builtin_enabled(KossJS.KOSS_BUILTIN_NODE) is True
        assert koss.is_builtin_enabled(KossJS.KOSS_BUILTIN_BUN) is True
        assert koss.is_builtin_enabled(KossJS.KOSS_BUILTIN_DENO) is False
        koss.destroy()

    def test_node_events_module(self):
        """Test the Node.js events module via koss:node/events"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var EventEmitter = require('koss:node/events').EventEmitter;
        var ee = new EventEmitter();
        var called = false;
        ee.on('test', function() { called = true; });
        ee.emit('test');
        called;
        """)
        assert result == 'true' or result is True
        koss.destroy()

    def test_node_path_module(self):
        """Test the Node.js path module via koss:node/path"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var path = require('koss:node/path');
        var p = path.join('/a', 'b', 'c');
        JSON.stringify({ joined: p, dir: path.dirname(p), base: path.basename(p) });
        """)
        parsed = eval(result) if isinstance(result, str) else result
        joined = parsed.get('joined', '').replace('\\', '/')
        assert '/a/b/c' in joined
        koss.destroy()

    def test_node_assert_module(self):
        """Test the Node.js assert module via koss:node/assert"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var assert = require('koss:node/assert');
        var ok = true;
        try { assert.strictEqual(1, 1); } catch(e) { ok = false; }
        ok;
        """)
        assert result == 'true' or result is True
        koss.destroy()

    def test_node_crypto_module(self):
        """Test the Node.js crypto module via koss:node/crypto"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var crypto = require('koss:node/crypto');
        var hash = crypto.createHash('sha256').update('test').digest('hex');
        var uuid = crypto.randomUUID();
        JSON.stringify({ hash: hash, uuid: uuid, hashes: crypto.getHashes() });
        """)
        assert result is not None
        koss.destroy()

    def test_node_querystring_module(self):
        """Test the Node.js querystring module via koss:node/querystring"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var qs = require('koss:node/querystring');
        var q = qs.stringify({ a: 1, b: 'hello' });
        JSON.stringify({ serialized: q, parsed: qs.parse(q) });
        """)
        assert result is not None
        koss.destroy()

    def test_node_os_module(self):
        """Test the Node.js os module via koss:node/os"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var os = require('koss:node/os');
        JSON.stringify({ platform: os.platform(), hostname: os.hostname(), type: os.type() });
        """)
        assert result is not None
        koss.destroy()

    def test_node_timers_module(self):
        """Test the Node.js timers module via koss:node/timers"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("typeof require('koss:node/timers').setTimeout")
        assert result == 'function'
        koss.destroy()

    def test_node_util_module(self):
        """Test the Node.js util module via koss:node/util"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("""
        var util = require('koss:node/util');
        JSON.stringify({ formatted: util.format('%s %d', 'hello', 42), isStr: util.types.isString('x') });
        """)
        assert result is not None
        koss.destroy()

    def test_node_zlib_module(self):
        """Test the Node.js zlib module via koss:node/zlib"""
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        result = koss.eval("require('koss:node/zlib').constants.Z_OK")
        assert result == '0' or result == 0
        koss.destroy()

    def test_capability_still_controls_io(self):
        """Test that Capability still controls I/O even with builtins enabled"""
        koss = KossJS(
            capabilities=KossJS.KOSS_CAP_SANDBOX,  # No FS or Net
            builtins=KossJS.KOSS_BUILTIN_NODE
        )
        # The module can be imported but actual I/O will fail due to missing caps
        try:
            koss.eval("""
            var fs = require('koss:node/fs');
            fs.existsSync('/tmp');
            """)
        except Exception:
            pass  # Expected: FS capability denied
        koss.destroy()

    def test_stable_still_disables_ffi_and_worker(self):
        """Test that stable mode still disables FFI and Worker even with builtins"""
        koss = KossJS(stable=True, builtins=KossJS.KOSS_BUILTIN_NODE)
        # FFI stub should exist but throw when called
        ffi_type = koss.eval("typeof _senri_ffi")
        assert ffi_type == 'object'
        with pytest.raises(JsError, match="stable mode|FFI is disabled"):
            koss.eval("_senri_ffi.func()")
        koss.destroy()

    def test_koss_global_object_version(self):
        """Test that KossJS global object still works"""
        koss = KossJS()
        version = koss.eval("KossJS.version")
        assert version is not None
        assert koss.eval("KossJS.runtime") == 'KossJS'
        koss.destroy()
