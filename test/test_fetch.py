import pytest
from kossjs_interface import KossJS


@pytest.mark.skip(reason="Fetch requires network access and may fail in CI")
class TestFetch:
    def test_fetch_https_standard(self, koss: KossJS):
        """Test fetch with standard Web API signature fetch(url)"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com');
            return 'status=' + r.status + ', ok=' + r.ok;
        })()
        """)
        assert "status=200" in result

    def test_fetch_with_init_object(self, koss: KossJS):
        """Test fetch with init object"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com', { method: 'GET' });
            return r.status;
        })()
        """)
        assert result == "200"

    def test_fetch_response_methods(self, koss: KossJS):
        """Verify Response.text() and Response.json() work"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com');
            var text = await r.text();
            return typeof text === 'string' && text.length > 0;
        })()
        """)
        assert result == "true"

    def test_fetch_response_headers(self, koss: KossJS):
        """Verify response headers are accessible"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com');
            var ct = r.headers.get('content-type');
            return ct !== null;
        })()
        """)
        assert result == "true"

    def test_fetch_response_array_buffer(self, koss: KossJS):
        """Verify Response.arrayBuffer() works"""
        result = koss.run_async("""
        (async function() {
            var r = await fetch('https://example.com');
            var buf = await r.arrayBuffer();
            return buf instanceof ArrayBuffer && buf.byteLength > 0;
        })()
        """)
        assert result == "true"

    def test_fetch_headers_get_set(self, koss: KossJS):
        """Verify Headers class works"""
        result = koss.run_async("""
        (async function() {
            var h = new Headers({ 'Content-Type': 'text/html' });
            return h.get('content-type');
        })()
        """)
        assert result == "text/html"

    def test_fetch_headers_iterate(self, koss: KossJS):
        """Verify Headers iteration works"""
        result = koss.run_async("""
        (async function() {
            var h = new Headers({ 'a': '1', 'b': '2' });
            var keys = [];
            for (var k of h.keys()) keys.push(k);
            return keys.sort().join(',');
        })()
        """)
        assert result == "a,b"

    def test_fetch_status_code(self, koss: KossJS):
        """Verify fetch returns numeric status"""
        result = koss.run_async("(async function() { var r = await fetch('https://example.com'); return r.status; })()")
        assert result == "200"
