from kossjs_interface import KossJS

print("=== Test: fetch() debug ===")
koss = KossJS()

try:
    result = koss.eval("""
    (function() {
        try {
            var responseJson = __koss_fetch('http://example.com', JSON.stringify({
                method: 'GET',
                headers: {},
                body: null
            }));
            return 'raw: ' + responseJson;
        } catch(e) {
            return 'error: ' + e.message;
        }
    })()
    """)
    print(f"Result: {result}")
except Exception as e:
    print(f"FAILED: {e}")
finally:
    koss.destroy()