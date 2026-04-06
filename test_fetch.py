from kossjs_interface import KossJS

print("=== Test: fetch() ===")
koss = KossJS()

try:
    result = koss.eval("""
    (function() {
        var r = fetch('https://example.com');
        return 'status=' + r.status + ', ok=' + r.ok;
    })()
    """)
    print(f"Result: {result}")
    print("PASSED: fetch works")
except Exception as e:
    print(f"FAILED: {e}")
    import traceback
    traceback.print_exc()
finally:
    koss.destroy()