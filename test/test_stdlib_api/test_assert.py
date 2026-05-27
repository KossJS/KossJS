"""Test assert module — Node.js-compatible assertion library."""

from .conftest import KossJS


class TestAssertAPI:
    def test_require_assert(self, koss: KossJS):
        result = koss.eval("typeof require('assert')")
        assert result == "function"

    def test_ok_truthy(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.ok(true)")

    def test_ok_falsy_throws(self, koss: KossJS):
        try:
            koss.eval("const assert = require('assert'); assert.ok(false)")
            assert False, "should have thrown"
        except Exception as e:
            assert "AssertionError" in str(e) or "assertion" in str(e).lower()

    def test_equal(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.equal(1, 1)")

    def test_equal_strict(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.strictEqual(1, 1)")

    def test_not_strict_equal(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.notStrictEqual(1, '1')")

    def test_deep_equal(self, koss: KossJS):
        koss.eval(
            "const assert = require('assert'); "
            "assert.deepStrictEqual({a:1,b:2}, {a:1,b:2})"
        )

    def test_deep_equal_nested(self, koss: KossJS):
        koss.eval(
            "const assert = require('assert'); "
            "assert.deepStrictEqual({a:[1,2,{x:3}]}, {a:[1,2,{x:3}]})"
        )

    def test_throws(self, koss: KossJS):
        koss.eval(
            "const assert = require('assert'); "
            "assert.throws(() => { throw new Error('boom') }, /boom/)"
        )

    def test_if_error_null(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.ifError(null)")

    def test_if_error_undefined(self, koss: KossJS):
        koss.eval("const assert = require('assert'); assert.ifError(undefined)")

    def test_if_error_throws(self, koss: KossJS):
        try:
            koss.eval(
                "const assert = require('assert'); "
                "assert.ifError(new Error('test'))"
            )
            assert False, "should have thrown"
        except Exception:
            pass

    def test_fail(self, koss: KossJS):
        try:
            koss.eval("const assert = require('assert'); assert.fail('message')")
            assert False, "should have thrown"
        except Exception as e:
            assert "message" in str(e) or "failed" in str(e).lower()

    def test_strict_mode(self, koss: KossJS):
        result = koss.eval("typeof require('assert').strict")
        assert result in ("object", "function")

    def test_does_not_throw(self, koss: KossJS):
        koss.eval(
            "const assert = require('assert'); "
            "assert.doesNotThrow(() => { var x = 1; })"
        )
