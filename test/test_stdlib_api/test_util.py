import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_util_format(koss: KossJS):
    result = koss.eval("require('util').format('%s=%d', 'x', 1)")
    assert result == 'x=1'


def test_util_format_multiple(koss: KossJS):
    result = koss.eval("require('util').format('%s %s', 'a', 'b', 'c')")
    assert result == 'a b c'


def test_util_format_no_specifiers(koss: KossJS):
    result = koss.eval("require('util').format('hello', 'world')")
    assert result == 'hello world'


def test_util_format_percent(koss: KossJS):
    result = koss.eval("require('util').format('100%%')")
    assert result == '100%'


def test_util_types(koss: KossJS):
    result = koss.eval("require('util').types.isArray([1,2,3])")
    assert result == 'true'


def test_util_types_is_string(koss: KossJS):
    result = koss.eval("require('util').types.isString('hello')")
    assert result == 'true'


def test_util_types_is_number(koss: KossJS):
    result = koss.eval("require('util').types.isNumber(42)")
    assert result == 'true'


def test_util_types_is_boolean(koss: KossJS):
    result = koss.eval("require('util').types.isBoolean(true)")
    assert result == 'true'


def test_util_types_is_null(koss: KossJS):
    result = koss.eval("require('util').types.isNull(null)")
    assert result == 'true'


def test_util_types_is_undefined(koss: KossJS):
    result = koss.eval("require('util').types.isUndefined(undefined)")
    assert result == 'true'


def test_util_types_is_function(koss: KossJS):
    result = koss.eval("require('util').types.isFunction(function(){})")
    assert result == 'true'


def test_util_types_is_object(koss: KossJS):
    result = koss.eval("require('util').types.isObject({})")
    assert result == 'true'


def test_util_types_is_reg_exp(koss: KossJS):
    result = koss.eval("require('util').types.isRegExp(/test/)")
    assert result == 'true'


def test_util_types_is_date(koss: KossJS):
    result = koss.eval("require('util').types.isDate(new Date())")
    assert result == 'true'


def test_util_deprecate(koss: KossJS):
    result = koss.eval("var fn = require('util').deprecate(function(){return 42;}, 'test'); typeof fn")
    assert result == 'function'


def test_util_inherits(koss: KossJS):
    result = koss.eval("""
        var util = require('util');
        function A() {}
        function B() {}
        util.inherits(B, A);
        var b = new B();
        b instanceof A
    """)
    assert result == 'true'


def test_util_promisify(koss: KossJS):
    result = koss.eval("typeof require('util').promisify(function(cb){cb(null, 42)})")
    assert result == 'function'


def test_util_inspect_object(koss: KossJS):
    result = koss.eval("typeof require('util').inspect({a: 1})")
    assert result == 'string'


def test_util_inspect_string(koss: KossJS):
    result = koss.eval("require('util').inspect('hello')")
    assert "'hello'" in result


def test_util_inspect_null(koss: KossJS):
    result = koss.eval("require('util').inspect(null)")
    assert result == 'null'


def test_util_inspect_number(koss: KossJS):
    result = koss.eval("require('util').inspect(42)")
    assert result == '42'


def test_util_inspect_function(koss: KossJS):
    result = koss.eval("require('util').inspect(function foo() {})")
    assert 'Function' in result
    assert 'foo' in result


def test_util_strip_vt(koss: KossJS):
    result = koss.eval("require('util').stripVTControlCharacters('\\x1b[31mhello\\x1b[0m')")
    assert result == 'hello'


def test_util_debuglog(koss: KossJS):
    result = koss.eval("typeof require('util').debuglog('test')")
    assert result == 'function'


def test_util_get_system_error_name(koss: KossJS):
    result = koss.eval("typeof require('util').getSystemErrorName")
    assert result == 'function'
