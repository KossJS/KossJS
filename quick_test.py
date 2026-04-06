from kossjs_interface import KossJS

print("=== Testing fetch ===")
koss = KossJS()

# Test 1: Check fetch exists
print("Test 1 - fetch exists:", koss.eval("typeof fetch"))

# Test 2: Get response and then get status
try:
    result = koss.eval("""
    (function() {
        var r = fetch('https://example.com', '{}');
        return 'status=' + r.status + ', ok=' + r.ok;
    })()
    """)
    print("Test 2 - fetch response:", result)
except Exception as e:
    print("Test 2 error:", e)

# Test 3: Get body
try:
    result = koss.eval("""
    (function() {
        var r = fetch('https://example.com', '{}');
        return r.status;
    })()
    """)
    print("Test 3 - fetch status:", result)
except Exception as e:
    print("Test 3 error:", e)

# Test 4: Get headers
try:
    result = koss.eval("""
    (function() {
        var r = fetch('https://example.com', '{}');
        return r.headers.get('content-type');
    })()
    """)
    print("Test 4 - content-type:", result)
except Exception as e:
    print("Test 4 error:", e)

koss.destroy()
