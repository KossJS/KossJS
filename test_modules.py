from kossjs_interface import KossJS

print("=== Testing Node.js modules ===")
koss = KossJS()

# Test 1: URL
print("\n--- Test 1: URL module ---")
try:
    result = koss.eval("""
    (function() {
        const { URL } = require('url');
        var u = new URL('https://example.com/path?query=1#hash');
        return 'href=' + u.href + ', host=' + u.host;
    })()
    """)
    print("Result:", result)
except Exception as e:
    print("Error:", e)

# Test 2: querystring
print("\n--- Test 2: querystring module ---")
try:
    result = koss.eval("""
    (function() {
        const qs = require('querystring');
        var parsed = qs.parse('foo=bar&baz=qux');
        return JSON.stringify(parsed);
    })()
    """)
    print("Result:", result)
except Exception as e:
    print("Error:", e)

# Test 3: path
print("\n--- Test 3: path module ---")
try:
    result = koss.eval("""
    (function() {
        const path = require('path');
        return 'basename=' + path.basename('/foo/bar/baz.txt') + ', ext=' + path.extname('/foo/bar/baz.txt');
    })()
    """)
    print("Result:", result)
except Exception as e:
    print("Error:", e)

# Test 4: events
print("\n--- Test 4: events module ---")
try:
    result = koss.eval("""
    (function() {
        const { EventEmitter } = require('events');
        var emitter = new EventEmitter();
        var count = 0;
        emitter.on('event', function() { count++; });
        emitter.emit('event');
        emitter.emit('event');
        return 'count=' + count;
    })()
    """)
    print("Result:", result)
except Exception as e:
    print("Error:", e)

# Test 5: assert
print("\n--- Test 5: assert module ---")
try:
    result = koss.eval("""
    (function() {
        const assert = require('assert');
        assert.strictEqual(1 + 1, 2);
        return 'assert passed';
    })()
    """)
    print("Result:", result)
except Exception as e:
    print("Error:", e)

koss.destroy()
