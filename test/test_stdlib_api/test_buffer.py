"""Test buffer module — Buffer class (FastBuffer extending Uint8Array)."""

from .conftest import KossJS


class TestBufferAPI:
    def test_require_buffer(self, koss: KossJS):
        result = koss.eval("typeof require('buffer').Buffer")
        assert result == "function"

    def test_buffer_from_string(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from('hello').toString()"
        )
        assert result == "hello"

    def test_buffer_from_array(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from([65, 66, 67]).toString()"
        )
        assert result == "ABC"

    def test_buffer_alloc(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.alloc(4, 'a').toString()"
        )
        assert result == "aaaa"

    def test_buffer_alloc_zero_filled(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.alloc(3).toString('hex')"
        )
        assert result == "000000"

    def test_buffer_alloc_unsafe(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.allocUnsafe(4).length"
        )
        assert result == "4"

    def test_buffer_byte_length(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.byteLength('hello')"
        )
        assert result == "5"

    def test_buffer_byte_length_utf8(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.byteLength('你好', 'utf8')"
        )
        assert result == "6"

    def test_buffer_concat(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.concat([Buffer.from('ab'), Buffer.from('cd')]).toString()"
        )
        assert result == "abcd"

    def test_buffer_compare(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.compare(Buffer.from('abc'), Buffer.from('abd'))"
        )
        assert int(result) < 0

    def test_buffer_is_buffer(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.isBuffer(Buffer.from('x'))"
        )
        assert result == "true"

    def test_buffer_is_not_buffer(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.isBuffer('not a buffer')"
        )
        assert result == "false"

    def test_buffer_is_encoding(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.isEncoding('utf8')"
        )
        assert result == "true"

    def test_buffer_to_string_base64(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from('hello').toString('base64')"
        )
        assert result == "aGVsbG8="

    def test_buffer_to_string_hex(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from('ab').toString('hex')"
        )
        assert result == "6162"

    def test_buffer_slice(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from('hello world').slice(6, 11).toString()"
        )
        assert result == "world"

    def test_buffer_copy(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "const dest = Buffer.alloc(5); "
            "Buffer.from('hello').copy(dest); "
            "dest.toString()"
        )
        assert result == "hello"

    def test_buffer_fill(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.alloc(4).fill('z').toString()"
        )
        assert result == "zzzz"

    def test_buffer_write(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "const b = Buffer.alloc(10); "
            "b.write('hello'); "
            "b.toString('utf8', 0, 5)"
        )
        assert result == "hello"

    def test_buffer_length(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.alloc(42).length"
        )
        assert result == "42"

    def test_buffer_indexing(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from([10, 20, 30])[1]"
        )
        assert result == "20"

    def test_btoa(self, koss: KossJS):
        result = koss.eval(
            "const {btoa} = require('buffer'); "
            "btoa('hello')"
        )
        assert result == "aGVsbG8="

    def test_atob(self, koss: KossJS):
        result = koss.eval(
            "const {atob} = require('buffer'); "
            "atob('aGVsbG8=')"
        )
        assert result == "hello"

    def test_buffer_constants(self, koss: KossJS):
        result = koss.eval(
            "const {constants} = require('buffer'); "
            "typeof constants"
        )
        assert result == "object"

    def test_buffer_k_max_length(self, koss: KossJS):
        result = koss.eval(
            "const {kMaxLength} = require('buffer'); "
            "typeof kMaxLength"
        )
        assert result == "number"

    def test_buffer_write_uint8(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "const b = Buffer.alloc(4); "
            "b.writeUInt8(0xFF, 0); "
            "b[0]"
        )
        assert result == "255"

    def test_buffer_read_uint8(self, koss: KossJS):
        result = koss.eval(
            "const {Buffer} = require('buffer'); "
            "Buffer.from([10, 20]).readUInt8(1)"
        )
        assert result == "20"
