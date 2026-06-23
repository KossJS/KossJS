import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_zlib_require(koss: KossJS):
    result = koss.eval("typeof require('zlib')")
    assert result == 'object'


def test_zlib_gzip_sync(koss: KossJS):
    result = koss.eval("""
        var z = require('zlib');
        var B = require('buffer').Buffer;
        var buf = B.from('hello');
        var compressed = z.gzipSync(buf);
        compressed.length > 0 && typeof compressed === 'object'
    """)
    assert result == 'true'


def test_zlib_gunzip_sync(koss: KossJS):
    result = koss.eval("""
        var z = require('zlib');
        var B = require('buffer').Buffer;
        var buf = B.from('hello world');
        var compressed = z.gzipSync(buf);
        var decompressed = z.gunzipSync(compressed);
        decompressed.toString() === 'hello world'
    """)
    assert result == 'true'


def test_zlib_deflate_sync(koss: KossJS):
    result = koss.eval("""
        var z = require('zlib');
        var B = require('buffer').Buffer;
        var buf = B.from('test data');
        var compressed = z.deflateSync(buf);
        compressed.length > 0
    """)
    assert result == 'true'


def test_zlib_inflate_sync(koss: KossJS):
    result = koss.eval("""
        var z = require('zlib');
        var B = require('buffer').Buffer;
        var buf = B.from('test data');
        var compressed = z.deflateSync(buf);
        var decompressed = z.inflateSync(compressed);
        decompressed.toString() === 'test data'
    """)
    assert result == 'true'


def test_zlib_gzip_async(koss: KossJS):
    result = koss.eval("typeof require('zlib').gzip")
    assert result == 'function'


def test_zlib_constants(koss: KossJS):
    result = koss.eval("typeof require('zlib').constants")
    assert result == 'object'


def test_zlib_empty(koss: KossJS):
    result = koss.eval("""
        var z = require('zlib');
        var B = require('buffer').Buffer;
        var compressed = z.gzipSync(B.from(''));
        var decompressed = z.gunzipSync(compressed);
        decompressed.toString() === ''
    """)
    assert result == 'true'


def test_dgram_require(koss: KossJS):
    result = koss.eval("typeof require('dgram')")
    assert result == 'object'


def test_dgram_create_socket(koss: KossJS):
    result = koss.eval("typeof require('dgram').createSocket")
    assert result == 'function'


def test_dgram_create_udp4(koss: KossJS):
    result = koss.eval("var s = require('dgram').createSocket('udp4'); typeof s")
    assert result == 'object'


def test_dgram_socket_type(koss: KossJS):
    result = koss.eval("""
        var s = require('dgram').createSocket('udp4');
        s.type === 'udp4'
    """)
    assert result == 'true'


def test_dgram_socket_address(koss: KossJS):
    result = koss.eval("""
        var s = require('dgram').createSocket('udp4');
        typeof s.address === 'function'
    """)
    assert result == 'true'
