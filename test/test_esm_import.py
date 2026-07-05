"""
Tests for ES module import with koss: protocol specifiers.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from kossjs_interface import KossJS


def esm_import(koss, code):
    """Run ESM import code, return object keys of __koss_esm_result."""
    koss.run_module_string(code)
    return koss.eval("Object.keys(__koss_esm_result || {})")


def esm_type(koss, code, prop):
    """Run ESM import and return typeof a property of __koss_esm_result."""
    koss.run_module_string(code)
    return koss.eval(f"typeof __koss_esm_result.{prop}")


class TestESMImportKossNode:
    def test_import_koss_node_fs(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import fs from 'koss:node/fs';\n")
        assert "readFileSync" in keys
        assert "writeFileSync" in keys
        koss.destroy()

    def test_import_koss_node_fs_readFileSync_type(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        t = esm_type(koss, "import fs from 'koss:node/fs';\n", "readFileSync")
        assert t == "function"
        koss.destroy()

    def test_import_koss_node_path(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import path from 'koss:node/path';\n")
        assert "join" in keys
        assert "resolve" in keys
        assert "extname" in keys
        koss.destroy()

    def test_import_koss_node_url(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import url from 'koss:node/url';\n")
        assert "URL" in keys
        assert "parse" in keys
        koss.destroy()

    def test_import_koss_node_util(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import util from 'koss:node/util';\n")
        assert "format" in keys
        assert "inspect" in keys
        koss.destroy()

    def test_import_koss_node_events(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import events from 'koss:node/events';\n")
        assert "EventEmitter" in keys
        koss.destroy()

    def test_import_koss_node_stream(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import stream from 'koss:node/stream';\n")
        assert "Readable" in keys
        assert "Writable" in keys
        koss.destroy()

    def test_import_koss_node_string_decoder(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import sd from 'koss:node/string_decoder';\n")
        assert "StringDecoder" in keys
        koss.destroy()

    def test_import_koss_node_timers(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import timers from 'koss:node/timers';\n")
        assert "setTimeout" in keys
        assert "setInterval" in keys
        koss.destroy()

    def test_import_koss_node_assert(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import assert from 'koss:node/assert';\n")
        assert "strictEqual" in keys
        koss.destroy()

    def test_import_koss_node_buffer(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import buf from 'koss:node/buffer';\n")
        assert "Buffer" in keys
        koss.destroy()

    def test_import_koss_node_process(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        # process module exports process object directly (module.exports = p)
        koss.run_module_string("import p from 'koss:node/process';\n")
        r = koss.eval("typeof __koss_esm_result")
        assert r == "object"
        koss.destroy()

    def test_import_koss_node_os(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import os from 'koss:node/os';\n")
        assert "platform" in keys
        assert "arch" in keys
        koss.destroy()

    def test_import_koss_node_crypto(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import crypto from 'koss:node/crypto';\n")
        assert "randomBytes" in keys
        assert "createHash" in keys
        koss.destroy()

    def test_import_koss_node_zlib(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import zlib from 'koss:node/zlib';\n")
        assert "gzipSync" in keys
        assert "gunzipSync" in keys
        koss.destroy()

    def test_import_koss_node_querystring(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import qs from 'koss:node/querystring';\n")
        assert "stringify" in keys
        assert "parse" in keys
        koss.destroy()

    def test_import_koss_node_constants(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import c from 'koss:node/constants';\n")
        assert "fs" in keys
        assert "os" in keys

    def test_import_koss_node_http(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import http from 'koss:node/http';\n")
        assert "createServer" in keys
        koss.destroy()

    def test_import_koss_node_net(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import net from 'koss:node/net';\n")
        assert "createServer" in keys
        assert "Socket" in keys
        koss.destroy()


class TestESMImportKossBun:
    def test_import_koss_bun(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import Bun from 'koss:bun';\n")
        assert "version" in keys
        koss.destroy()

    def test_import_koss_bun_disabled(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NODE)
        with pytest.raises(Exception):
            koss.run_module_string("import Bun from 'koss:bun';\n")
        koss.destroy()


class TestESMImportKossDeno:
    def test_import_koss_deno(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import Deno from 'koss:deno';\n")
        assert "version" in keys
        koss.destroy()


class TestESMImportKossIo:
    def test_import_koss_io(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import io from 'koss:io';\n")
        assert "read" in keys
        assert "write" in keys
        koss.destroy()


class TestESMImportKossCrypto:
    def test_import_koss_crypto(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import crypto from 'koss:crypto';\n")
        assert "hash" in keys
        assert "randomBytes" in keys
        koss.destroy()


class TestESMImportKossSystem:
    def test_import_koss_system(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import system from 'koss:system';\n")
        assert "hostname" in keys
        koss.destroy()


class TestESMImportKossData:
    def test_import_koss_data(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import data from 'koss:data';\n")
        assert "fromHex" in keys
        assert "fromBase64" in keys
        koss.destroy()


class TestESMImportKossInternal:
    def test_import_koss_internal_fs(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import fs from 'koss:internal/fs';\n")
        assert "readFileSync" in keys
        koss.destroy()

    def test_import_koss_internal_net(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import net from 'koss:internal/net';\n")
        assert "tcpConnect" in keys
        koss.destroy()

    def test_import_koss_internal_crypto(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import c from 'koss:internal/crypto';\n")
        assert "hash" in keys
        koss.destroy()

    def test_import_koss_internal_stream(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import s from 'koss:internal/stream';\n")
        assert "createReadStream" in keys or "ReadStream" in keys
        koss.destroy()


class TestESMImportNodePrefix:
    def test_import_node_fs(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import fs from 'node:fs';\n")
        assert "readFileSync" in keys
        koss.destroy()

    def test_import_node_path(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import path from 'node:path';\n")
        assert "join" in keys
        koss.destroy()

    def test_import_node_events(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import events from 'node:events';\n")
        assert "EventEmitter" in keys
        koss.destroy()

    def test_import_node_buffer(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import buf from 'node:buffer';\n")
        assert "Buffer" in keys
        koss.destroy()

    def test_import_node_util(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        keys = esm_import(koss, "import util from 'node:util';\n")
        assert "format" in keys
        koss.destroy()


class TestESMImportFailure:
    def test_import_koss_unknown_module(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        with pytest.raises(Exception):
            koss.run_module_string("import x from 'koss:unknown_xyz';\n")
        koss.destroy()

    def test_import_disabled_builtin(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_NONE)
        with pytest.raises(Exception):
            koss.run_module_string("import fs from 'koss:node/fs';\n")
        koss.destroy()


class TestESMImportMixed:
    def test_import_multiple_koss_modules(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        koss.run_module_string(
            "import path from 'koss:node/path';\n"
            "import fs from 'koss:node/fs';\n"
            "import buf from 'koss:node/buffer';\n"
        )
        r = koss.eval("Object.keys(__koss_esm_result || {})")
        assert "Buffer" in r
        koss.destroy()

    def test_import_koss_require_interop(self):
        koss = KossJS(builtins=KossJS.KOSS_BUILTIN_ALL)
        koss.run_module_string("import path from 'koss:node/path';\n")
        result = koss.eval("typeof require('koss:node/path').join")
        assert result == "function"
        koss.destroy()
