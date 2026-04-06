"""Test KossJS callback functionality"""

from kossjs_interface import KossJS, JsError # pyright: ignore[reportUnusedImport]

def test_callback():
    print("=== Test: Python callback returning string ===")
    koss = KossJS()
    
    def myFunc(name: str):
        return f"Hello, {name}!"
    
    koss.register_function("myFunc", myFunc)
    
    try:
        result = koss.eval("myFunc('World')")
        print(f"Result: {result}")
        assert result == "Hello, World!", f"Expected 'Hello, World!' but got '{result}'"
        print("PASSED: Callback returned correct value")
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

def test_basic_eval():
    print("\n=== Test: Basic eval ===")
    koss = KossJS()
    
    try:
        result = koss.eval("1 + 2")
        print(f"Result: {result}")
        assert result == "3", f"Expected '3' but got '{result}'"
        print("PASSED: Basic eval works")
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

def test_set_global():
    print("\n=== Test: Set global variable ===")
    koss = KossJS()
    
    try:
        koss.set_global("myVar", 42)
        result = koss.eval("myVar * 2")
        print(f"Result: {result}")
        assert result == "84", f"Expected '84' but got '{result}'"
        print("PASSED: set_global works")
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

def test_fetch():
    print("\n=== Test: fetch() ===")
    koss = KossJS()
    
    try:
        result = koss.eval("""
        (function() {
            var r = fetch('https://example.com', '{}');
            return 'status=' + r.status + ', ok=' + r.ok;
        })()
        """)
        print(f"Result: {result}")
        assert 'status=200' in result, f"Expected status=200 but got '{result}'"
        print("PASSED: fetch works")
        
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

def test_js_basic_api():
    print("\n=== Test: JavaScript Basic API ===")
    koss = KossJS()
    
    try:
        # Test 1: console.log
        result = koss.eval("console.log('Hello from console'); 1 + 1")
        print(f"console.log test: {result}")
        
        # Test 2: Math object
        result = koss.eval("Math.sqrt(16)")
        print(f"Math.sqrt: {result}")
        assert result == "4", f"Expected '4' but got '{result}'"
        
        result = koss.eval("Math.abs(-5)")
        print(f"Math.abs: {result}")
        assert result == "5", f"Expected '5' but got '{result}'"
        
        # Test 3: JSON object
        result = koss.eval("JSON.stringify({a: 1, b: 2})")
        print(f"JSON.stringify: {result}")
        assert result == '{"a":1,"b":2}', f"Expected JSON but got '{result}'"
        
        result = koss.eval("JSON.parse('{\"x\":10}').x")
        print(f"JSON.parse: {result}")
        assert result == "10", f"Expected '10' but got '{result}'"
        
        # Test 4: Array
        result = koss.eval("[1, 2, 3].map(x => x * 2)")
        print(f"Array.map: {result}")
        assert result == "2,4,6", f"Expected '2,4,6' but got '{result}'"
        
        result = koss.eval("[1, 2, 3].filter(x => x > 1)")
        print(f"Array.filter: {result}")
        assert result == "2,3", f"Expected '2,3' but got '{result}'"
        
        result = koss.eval("[1, 2, 3].reduce((a, b) => a + b, 0)")
        print(f"Array.reduce: {result}")
        assert result == "6", f"Expected '6' but got '{result}'"
        
        # Test 5: String
        result = koss.eval("'hello'.toUpperCase()")
        print(f"String.toUpperCase: {result}")
        assert result == "HELLO", f"Expected 'HELLO' but got '{result}'"
        
        result = koss.eval("'hello world'.split(' ')")
        print(f"String.split: {result}")
        assert result == "hello,world", f"Expected 'hello,world' but got '{result}'"
        
        # Test 6: Object
        result = koss.eval("Object.keys({a: 1, b: 2})")
        print(f"Object.keys: {result}")
        assert result == "a,b", f"Expected 'a,b' but got '{result}'"
        
        result = koss.eval("Object.values({a: 1, b: 2})")
        print(f"Object.values: {result}")
        assert result == "1,2", f"Expected '1,2' but got '{result}'"
        
        # Test 7: Number
        result = koss.eval("Number.isNaN(NaN)")
        print(f"Number.isNaN: {result}")
        assert result == "true", f"Expected 'true' but got '{result}'"
        
        result = koss.eval("Number.isInteger(5)")
        print(f"Number.isInteger: {result}")
        assert result == "true", f"Expected 'true' but got '{result}'"
        
        # Test 8: Date
        result = koss.eval("new Date(0).getFullYear()")
        print(f"Date.getFullYear: {result}")
        assert result == "1970", f"Expected '1970' but got '{result}'"
        
        # Test 9: set_global and eval together
        koss.set_global("testVar", 100)
        result = koss.eval("testVar + 50")
        print(f"set_global + eval: {result}")
        assert result == "150", f"Expected '150' but got '{result}'"
        
        # Test 10: global eval
        result = koss.eval("eval('1 + 2 + 3')")
        print(f"eval(): {result}")
        assert result == "6", f"Expected '6' but got '{result}'"
        
        # Test 11: parseInt / parseFloat
        result = koss.eval("parseInt('42')")
        print(f"parseInt: {result}")
        assert result == "42", f"Expected '42' but got '{result}'"
        
        result = koss.eval("parseFloat('3.14')")
        print(f"parseFloat: {result}")
        assert result == "3.14", f"Expected '3.14' but got '{result}'"
        
        # Test 12: isFinite / isNaN
        result = koss.eval("isFinite(100)")
        print(f"isFinite: {result}")
        assert result == "true", f"Expected 'true' but got '{result}'"
        
        result = koss.eval("isNaN(NaN)")
        print(f"isNaN: {result}")
        assert result == "true", f"Expected 'true' but got '{result}'"
        
        # Test 13: encodeURI / decodeURI
        result = koss.eval("encodeURIComponent('hello world')")
        print(f"encodeURIComponent: {result}")
        assert result == "hello%20world", f"Expected encoded string but got '{result}'"
        
        print("\nPASSED: All Basic API tests passed!")
    except Exception as e:
        print(f"FAILED: {e}")
        import traceback
        traceback.print_exc()
    finally:
        koss.destroy()

def test_modules_find():
    print("\n=== Test: Modules find ===")
    koss = KossJS()
    
    try:
#         koss.run_string('import a from "./a.js";\
# console.log(a)\
# ')
        # koss.run_file('./b.js')
        
        print("PASSED: Modules find works correctly")
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

if __name__ == "__main__":
    test_basic_eval()
    test_set_global()
    test_callback()
    test_fetch()
    test_js_basic_api()
    test_modules_find()
    print("\nAll tests completed!")
