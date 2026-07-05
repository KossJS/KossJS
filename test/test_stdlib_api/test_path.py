"""Test path module — POSIX path utilities."""

from .conftest import KossJS


class TestPathAPI:
    def test_require_path(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/path')")
        assert result == "object"

    def test_basename(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').basename('/foo/bar/baz.txt')")
        assert result == "baz.txt"

    def test_basename_with_ext(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').basename('/foo/bar/baz.txt', '.txt')")
        assert result == "baz"

    def test_dirname(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').dirname('/foo/bar/baz.txt')")
        assert result in ("/foo/bar", "\\foo\\bar")

    def test_extname(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').extname('file.txt')")
        assert result == ".txt"

    def test_extname_none(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').extname('file')")
        assert result == ""

    def test_join(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').join('/foo', 'bar', 'baz')")
        assert "foo" in result and "bar" in result and "baz" in result

    def test_join_empty(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').join()")
        assert result == "."

    def test_normalize(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').normalize('/foo/bar/../baz')")
        assert result.endswith("foo/baz") or result.endswith("foo\\baz")

    def test_is_absolute_unix(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').isAbsolute('/foo/bar')")
        assert result == "true"

    def test_is_absolute_relative(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').isAbsolute('foo/bar')")
        assert result == "false"

    def test_resolve(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').resolve('/foo', 'bar')")
        assert "foo" in result and "bar" in result

    def test_relative(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').relative('/data/test', '/data/test/foo/bar')")
        assert "foo" in result and "bar" in result

    def test_parse(self, koss: KossJS):
        result = koss.eval(
            "var p = require('koss:node/path').parse('/home/user/file.txt'); "
            "JSON.stringify({root: p.root, dir: p.dir, base: p.base, ext: p.ext, name: p.name})"
        )
        # koss.eval auto-parses JSON, so result is a dict
        assert isinstance(result, dict)
        assert "home" in result["dir"]

    def test_format(self, koss: KossJS):
        result = koss.eval(
            "require('koss:node/path').format({dir: '/home/user', base: 'file.txt'})"
        )
        assert "home" in result and "file.txt" in result

    def test_sep(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').sep")
        assert result in ("/", "\\")

    def test_delimiter(self, koss: KossJS):
        result = koss.eval("require('koss:node/path').delimiter")
        assert isinstance(result, str)

    def test_posix(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/path').posix")
        assert result == "object"

    def test_win32(self, koss: KossJS):
        result = koss.eval("typeof require('koss:node/path').win32")
        assert result == "object"
