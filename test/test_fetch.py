import pytest
from kossjs_interface import KossJS


@pytest.mark.skip(reason="Fetch requires network access and may fail in CI")
class TestFetch:
    def test_fetch_https(self, koss: KossJS):
        """Test fetch with HTTPS URL"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com', '{}');
            return 'status=' + r.status + ', ok=' + r.ok;
        })()
        """)
        assert "status=200" in result

    def test_fetch_status_code(self, koss: KossJS):
        """Verify fetch returns numeric status"""
        result = koss.run_async("(async function() { var r = await fetch('https://example.com', '{}'); return r.status; })()")
        assert result == "200"

    @pytest.mark.skip(reason="HTTP may be blocked")
    def test_fetch_http(self, koss: KossJS):
        result = koss.run_async("(async function() { var r = await fetch('http://example.com'); return r.status; })()")
        assert result is not None
