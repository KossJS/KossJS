"""Test process module — global process object."""

from .conftest import KossJS


class TestProcessAPI:
    def test_require_process(self, koss: KossJS):
        result = koss.eval("typeof require('process')")
        assert result == "object"

    def test_process_exists(self, koss: KossJS):
        result = koss.eval("typeof process")
        assert result == "object"

    def test_arch(self, koss: KossJS):
        result = koss.eval("process.arch")
        assert isinstance(result, str) and len(result) > 0

    def test_platform(self, koss: KossJS):
        result = koss.eval("process.platform")
        assert isinstance(result, str)

    def test_version(self, koss: KossJS):
        result = koss.eval("process.version")
        assert isinstance(result, str)

    def test_versions(self, koss: KossJS):
        result = koss.eval("typeof process.versions")
        assert result == "object"

    def test_cwd(self, koss: KossJS):
        result = koss.eval("process.cwd()")
        assert isinstance(result, str) and len(result) > 0

    def test_argv(self, koss: KossJS):
        result = koss.eval("Array.isArray(process.argv)")
        assert result == "true"

    def test_env_exists(self, koss: KossJS):
        result = koss.eval("typeof process.env")
        assert result == "object"

    def test_pid(self, koss: KossJS):
        result = koss.eval("typeof process.pid")
        assert result == "number"

    def test_next_tick(self, koss: KossJS):
        # nextTick should be a function
        result = koss.eval("typeof process.nextTick")
        assert result == "function"

    def test_exit_code(self, koss: KossJS):
        result = koss.eval("process.exitCode === undefined || typeof process.exitCode === 'number'")
        assert result == "true"

    def test_stdout(self, koss: KossJS):
        result = koss.eval("typeof process.stdout")
        assert result in ("object", "undefined")

    def test_stderr(self, koss: KossJS):
        result = koss.eval("typeof process.stderr")
        assert result in ("object", "undefined")

    def test_stdin(self, koss: KossJS):
        result = koss.eval("typeof process.stdin")
        assert result in ("object", "undefined")

    def test_title(self, koss: KossJS):
        result = koss.eval("typeof process.title")
        assert result == "string"

    def test_memory_usage(self, koss: KossJS):
        result = koss.eval("typeof process.memoryUsage === 'function'")
        assert result == "true"
