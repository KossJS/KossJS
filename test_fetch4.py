from kossjs_interface import KossJS
import time

print("=== Test: fetch() with HTTP ===")
koss = KossJS()

try:
    result = koss.eval("fetch('http://example.com').status")
    print(f"Status: {result}")
except Exception as e:
    print(f"FAILED: {e}")
finally:
    koss.destroy()