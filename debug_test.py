from kossjs_interface import KossJS

print("=== Testing module loader ===")
koss = KossJS()

# Test 1: First let's see if require works
print("\n--- Test 1: require function ---")
try:
    result = koss.eval("typeof require")
    print("typeof require:", result)
except Exception as e:
    print("Error:", e)

# Test 2: Try to require path
print("\n--- Test 2: require path ---")
try:
    result = koss.eval("require('path')")
    print("require('path'):", result)
except Exception as e:
    print("Error:", e)

# Test 3: Try to call path methods directly 
print("\n--- Test 3: path methods ---")
try:
    result = koss.eval("path.basename('/foo/bar.txt')")
    print("path.basename:", result)
except Exception as e:
    print("Error:", e)

koss.destroy()
