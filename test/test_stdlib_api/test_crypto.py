import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_crypto_random_bytes(koss: KossJS):
    result = koss.eval("require('crypto').randomBytes(16).length")
    assert result == '16'


def test_crypto_random_bytes_zero(koss: KossJS):
    result = koss.eval("require('crypto').randomBytes(0).length")
    assert result == '0'


def test_crypto_create_hash_sha256(koss: KossJS):
    result = koss.eval("require('crypto').createHash('sha256').update('hello').digest('hex')")
    assert result == '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'


def test_crypto_create_hash_sha1(koss: KossJS):
    result = koss.eval("require('crypto').createHash('sha1').update('hello').digest('hex')")
    assert result == 'aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d'


def test_crypto_create_hash_md5(koss: KossJS):
    result = koss.eval("require('crypto').createHash('md5').update('hello').digest('hex')")
    assert result == '5d41402abc4b2a76b9719d911017c592'


def test_crypto_create_hash_chain(koss: KossJS):
    result = koss.eval("require('crypto').createHash('sha256').update('a').update('b').digest('hex')")
    assert result == 'fb8e20fc2e4c3f248c60c39bd652f3c1347298bb977b8b4d5903b85055620603'


def test_crypto_random_uuid(koss: KossJS):
    result = koss.eval("typeof require('crypto').randomUUID()")
    assert result == 'string'


def test_crypto_random_uuid_format(koss: KossJS):
    result = koss.eval("""
        var uuid = require('crypto').randomUUID();
        uuid.length === 36 && uuid[8] === '-' && uuid[13] === '-'
    """)
    assert result == 'true'


def test_crypto_timing_safe_equal(koss: KossJS):
    result = koss.eval("""
        var crypto = require('crypto');
        crypto.timingSafeEqual([1,2,3], [1,2,3])
    """)
    assert result == 'true'


def test_crypto_timing_safe_equal_diff(koss: KossJS):
    result = koss.eval("""
        var crypto = require('crypto');
        crypto.timingSafeEqual([1,2,3], [1,2,4])
    """)
    assert result == 'false'


def test_crypto_get_hashes(koss: KossJS):
    result = koss.eval("require('crypto').getHashes().length")
    assert result == '3'
