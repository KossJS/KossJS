from typing import Literal
from collections.abc import Callable
import pytest
from kossjs_interface import KossJS


class TestRegisterFunctionDotted:
    def test_math_max(self, koss: KossJS):
        def math_max(a: str, b: str) -> str:
            return str(max(int(a), int(b)))
        koss.register_function("Math.max", math_max)
        assert koss.eval("Math.max(10, 20)") == "20"

    def test_math_min(self, koss: KossJS):
        def math_min(a: str, b: str) -> str:
            return str(min(int(a), int(b)))
        koss.register_function("Math.min", math_min)
        assert koss.eval("Math.min(10, 20)") == "10"

    def test_nested_namespace(self, koss: KossJS):
        def utils_format(s: str) -> str:
            return s.upper()
        def utils_repeat(s: str) -> str:
            return s * 2
        koss.register_function("utils.format", utils_format)
        koss.register_function("utils.repeat", utils_repeat)
        assert koss.eval("utils.format('hello')") == "HELLO"
        assert koss.eval("utils.repeat('ab')") == "abab"

    def test_nested_deep(self, koss: KossJS):
        def deep_func() -> Literal['deep_ok']:
            return "deep_ok"
        koss.register_function("a.b.c.deep", deep_func)
        assert koss.eval("a.b.c.deep()") == "deep_ok"


class TestRegisterClass:
    def test_class_exists(self, koss: KossJS):
        koss.register_class("MyCalc", {"add": lambda *a: str(sum(int(x) for x in a))})
        assert koss.eval("typeof MyCalc") == "function"

    def test_new_instance(self, koss: KossJS):
        koss.register_class("MyCalc", {"add": lambda *a: str(sum(int(x) for x in a))})
        assert koss.eval("var c = new MyCalc(); typeof c") == "object"

    def test_class_method_add(self, koss: KossJS):
        koss.register_class("MyCalc", {"add": lambda *a: str(sum(int(x) for x in a))})
        assert koss.eval("var c = new MyCalc(); c.add(10, 20, 30)") == "60"

    @pytest.mark.parametrize("method,args,expected", [
        ("increment", "5", "6"),
        ("add", "3, 4", "7"),
        ("multiply", "6, 7", "42"),
    ])
    def test_class_methods(self, koss: KossJS, method: str, args: str, expected: str):
        methods: dict[str, Callable[[tuple[str, ...] | list[str]], str]] = {
            "increment": lambda *a: str(int(a[0]) + 1) if a else "1", # pyright: ignore[reportUnknownArgumentType, reportUnknownLambdaType]
            "add": lambda *a: str(sum(int(x) for x in a)), # pyright: ignore[reportUnknownArgumentType, reportUnknownLambdaType, reportUnknownVariableType]
            "multiply": lambda *a: str(int(a[0]) * int(a[1])) if len(a) >= 2 else "0", # pyright: ignore[reportUnknownArgumentType, reportUnknownLambdaType]
        }
        koss.register_class("MyCounter", methods)
        assert koss.eval(f"var c = new MyCounter(); c.{method}({args})") == expected

    def test_multiple_instances(self, koss: KossJS):
        methods: dict[str, Callable[[tuple[str, ...] | list[str]], str]] = {"increment": lambda *a: str(int(a[0]) + 1) if a else "1"} # pyright: ignore[reportUnknownArgumentType, reportUnknownLambdaType]
        koss.register_class("MyCounter", methods)
        result = koss.eval("""
        var c1 = new MyCounter();
        var c2 = new MyCounter();
        c1.increment(1); c2.increment(10);
        """)
        assert result is not None



