"""Test os module — operating system information."""

from .conftest import KossJS


class TestOSAPI:
    def test_require_os(self, koss: KossJS):
        result = koss.eval("typeof require('os')")
        assert result == "object"

    def test_arch(self, koss: KossJS):
        result = koss.eval("require('os').arch()")
        assert isinstance(result, str) and len(result) > 0

    def test_platform(self, koss: KossJS):
        result = koss.eval("require('os').platform()")
        assert isinstance(result, str)

    def test_type(self, koss: KossJS):
        result = koss.eval("require('os').type()")
        assert isinstance(result, str)

    def test_release(self, koss: KossJS):
        result = koss.eval("require('os').release()")
        assert isinstance(result, str)

    def test_cpus(self, koss: KossJS):
        result = koss.eval("Array.isArray(require('os').cpus())")
        assert result == "true"

    def test_freemem(self, koss: KossJS):
        result = koss.eval("typeof require('os').freemem()")
        assert result == "number"

    def test_totalmem(self, koss: KossJS):
        result = koss.eval("typeof require('os').totalmem()")
        assert result == "number"

    def test_homedir(self, koss: KossJS):
        result = koss.eval("require('os').homedir()")
        assert isinstance(result, str)

    def test_hostname(self, koss: KossJS):
        result = koss.eval("require('os').hostname()")
        assert isinstance(result, str)

    def test_tmpdir(self, koss: KossJS):
        result = koss.eval("require('os').tmpdir()")
        assert isinstance(result, str)

    def test_loadavg(self, koss: KossJS):
        result = koss.eval("Array.isArray(require('os').loadavg())")
        assert result == "true"

    def test_uptime(self, koss: KossJS):
        result = koss.eval("typeof require('os').uptime()")
        assert result == "number"

    def test_endianness(self, koss: KossJS):
        result = koss.eval("require('os').endianness()")
        assert result in ("LE", "BE")

    def test_eol(self, koss: KossJS):
        result = koss.eval("require('os').EOL")
        assert isinstance(result, str) and len(result) > 0

    def test_dev_null(self, koss: KossJS):
        result = koss.eval("require('os').devNull")
        assert isinstance(result, str)

    def test_constants(self, koss: KossJS):
        result = koss.eval("typeof require('os').constants")
        assert result == "object"

    def test_constants_signals(self, koss: KossJS):
        result = koss.eval("typeof require('os').constants.signals")
        assert result == "object"

    def test_constants_errno(self, koss: KossJS):
        result = koss.eval("typeof require('os').constants.errno")
        assert result == "object"

    def test_user_info(self, koss: KossJS):
        result = koss.eval("typeof require('os').userInfo()")
        assert result == "object"

    def test_version(self, koss: KossJS):
        result = koss.eval("require('os').version()")
        assert isinstance(result, str)

    def test_machine(self, koss: KossJS):
        result = koss.eval("require('os').machine()")
        assert isinstance(result, str)

    def test_network_interfaces(self, koss: KossJS):
        result = koss.eval("typeof require('os').networkInterfaces()")
        assert result == "object"

    def test_available_parallelism(self, koss: KossJS):
        result = koss.eval("typeof require('os').availableParallelism()")
        assert result == "number"
