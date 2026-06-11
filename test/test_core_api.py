import pytest # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS


class TestCoreAPI:
    def test_basic_eval(self, koss: KossJS):
        assert koss.eval("1 + 2") == "3"

    def test_string_concat(self, koss: KossJS):
        assert koss.eval("'hello' + ' ' + 'world'") == "hello world"

    def test_template_literal(self, koss: KossJS):
        assert koss.eval("`${1 + 2}`") == "3"

    def test_math_sqrt(self, koss: KossJS):
        assert koss.eval("Math.sqrt(16)") == "4"

    def test_math_abs(self, koss: KossJS):
        assert koss.eval("Math.abs(-5)") == "5"

    def test_json_stringify(self, koss: KossJS):
        assert koss.eval("JSON.stringify({a: 1, b: 2})") == {"a": 1, "b": 2}

    def test_json_parse(self, koss: KossJS):
        assert koss.eval("JSON.parse('{\"x\":10}').x") == "10"

    def test_array_map(self, koss: KossJS):
        assert koss.eval("[1, 2, 3].map(x => x * 2)") == [2, 4, 6]

    def test_array_filter(self, koss: KossJS):
        assert koss.eval("[1, 2, 3].filter(x => x > 1)") == [2, 3]

    def test_array_reduce(self, koss: KossJS):
        assert koss.eval("[1, 2, 3].reduce((a, b) => a + b, 0)") == "6"

    def test_string_upper(self, koss: KossJS):
        assert koss.eval("'hello'.toUpperCase()") == "HELLO"

    def test_string_split(self, koss: KossJS):
        assert koss.eval("'hello world'.split(' ')") == ["hello", "world"]

    def test_object_keys(self, koss: KossJS):
        assert koss.eval("Object.keys({a: 1, b: 2})") == ["a", "b"]

    def test_object_values(self, koss: KossJS):
        assert koss.eval("Object.values({a: 1, b: 2})") == [1, 2]

    def test_is_nan(self, koss: KossJS):
        assert koss.eval("Number.isNaN(NaN)") == "true"

    def test_is_integer(self, koss: KossJS):
        assert koss.eval("Number.isInteger(5)") == "true"

    def test_date(self, koss: KossJS):
        assert koss.eval("new Date(0).getFullYear()") == "1970"

    def test_eval_function(self, koss: KossJS):
        assert koss.eval("eval('1 + 2 + 3')") == "6"

    def test_parse_int(self, koss: KossJS):
        assert koss.eval("parseInt('42')") == "42"

    def test_parse_float(self, koss: KossJS):
        assert koss.eval("parseFloat('3.14')") == "3.14"

    def test_is_finite(self, koss: KossJS):
        assert koss.eval("isFinite(100)") == "true"

    def test_is_nan_global(self, koss: KossJS):
        assert koss.eval("isNaN(NaN)") == "true"

    def test_encode_uri(self, koss: KossJS):
        assert koss.eval("encodeURIComponent('hello world')") == "hello%20world"


class TestSetGlobal:
    def test_set_global_string(self, koss: KossJS):
        koss.set_global("my_string", "hello")
        assert koss.eval("my_string") == "hello"

    def test_set_global_number(self, koss: KossJS):
        koss.set_global("my_num", 42)
        assert koss.eval("my_num") == "42"

    def test_set_global_float(self, koss: KossJS):
        koss.set_global("my_float", 3.14)
        assert koss.eval("my_float") == "3.14"

    def test_set_global_bool_true(self, koss: KossJS):
        koss.set_global("my_bool", True)
        assert koss.eval("my_bool") == "true"

    def test_set_global_null(self, koss: KossJS):
        koss.set_global("my_null", None)
        assert koss.eval("my_null === null") == "true"

    def test_set_global_array(self, koss: KossJS):
        koss.set_global("my_arr", [1, 2, 3])
        assert koss.eval("JSON.stringify(my_arr)") == [1, 2, 3]

    def test_set_global_object(self, koss: KossJS):
        koss.set_global("my_obj", {"name": "test", "value": 99})
        assert koss.eval("my_obj.name") == "test"

    def test_set_global_nested(self, koss: KossJS):
        koss.set_global("nested", {"a": {"b": [1, 2, {"c": 3}]}})
        assert koss.eval("JSON.stringify(nested)") == {"a": {"b": [1, 2, {"c": 3}]}}

    def test_global_used_in_expression(self, koss: KossJS):
        koss.set_global("x", 100)
        assert koss.eval("x + 50") == "150"


class TestCallback:
    def test_callback_returns_string(self, koss: KossJS):
        def greet(name: str) -> str:
            return f"Hello, {name}!"
        koss.register_function("greet", greet)
        assert koss.eval("greet('World')") == "Hello, World!"

    def test_callback_addition(self, koss: KossJS):
        def add(a: str, b: str) -> str:
            return str(int(a) + int(b))
        koss.register_function("add", add)
        assert koss.eval("add(3, 4)") == "7"

    def test_callback_no_return(self, koss: KossJS):
        def noop() -> None:
            return None
        koss.register_function("noop", noop)
        assert koss.eval("noop()") == "undefined"

    def test_callback_chaining(self, koss: KossJS):
        def double_(x: str) -> str:
            return str(int(x) * 2)
        koss.register_function("double", double_)
        assert koss.eval("double(double(5))") == "20"


class TestKossJSGlobal:
    def test_kossjs_object_exists(self, koss: KossJS):
        assert koss.eval("typeof KossJS") == "object"

    def test_kossjs_version_is_string(self, koss: KossJS):
        assert koss.eval("typeof KossJS.version") == "string"

    def test_kossjs_version_format(self, koss: KossJS):
        result = koss.eval("KossJS.version")
        assert isinstance(result, str)
        assert len(result) > 0

    def test_kossjs_runtime_value(self, koss: KossJS):
        assert koss.eval("KossJS.runtime") == "KossJS"

    def test_kossjs_is_frozen(self, koss: KossJS):
        assert koss.eval("Object.isFrozen(KossJS)") == "true"

    def test_kossjs_has_no_prototype(self, koss: KossJS):
        assert koss.eval("Object.getPrototypeOf(KossJS)") == "null"

    def test_kossjs_version_readonly(self, koss: KossJS):
        koss.eval("KossJS.version = 'modified'")
        assert koss.eval("KossJS.version") != "modified"

    def test_kossjs_runtime_readonly(self, koss: KossJS):
        koss.eval("KossJS.runtime = 'modified'")
        assert koss.eval("KossJS.runtime") == "KossJS"

    def test_kossjs_cannot_add_property(self, koss: KossJS):
        koss.eval("KossJS.newProp = 123")
        assert koss.eval("KossJS.newProp") == "undefined"

    def test_kossjs_cannot_delete_property(self, koss: KossJS):
        assert koss.eval("delete KossJS.version") == "false"

    def test_kossjs_global_readonly(self, koss: KossJS):
        koss.eval("KossJS = {}")
        assert koss.eval("typeof KossJS") == "object"
        assert koss.eval("KossJS.runtime") == "KossJS"
