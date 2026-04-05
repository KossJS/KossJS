"""Test KossJS callback functionality"""

from kossjs_interface import KossJS, JsError

def test_callback():
    print("=== Test: Python callback returning string ===")
    koss = KossJS()
    
    def myFunc(name):
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
        koss.register_fetch()
        
        result = koss.eval("fetch('https://jsonplaceholder.typicode.com/posts/1').status")
        print(f"Status: {result}")
        assert result == "200", f"Expected '200' but got '{result}'"
        print("PASSED: fetch works")
        
        body = koss.eval("fetch('https://jsonplaceholder.typicode.com/posts/1').text()")
        print(f"Body starts with: {body[:50]}...")
        
    except Exception as e:
        print(f"FAILED: {e}")
    finally:
        koss.destroy()

if __name__ == "__main__":
    test_basic_eval()
    test_set_global()
    test_callback()
    test_fetch()
    print("\nAll tests completed!")
