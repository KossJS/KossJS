"""
KossJS Python Interface - Embeddable JavaScript runtime for Python
"""

import ctypes
import os
import sys
from pathlib import Path
from typing import Any
from collections.abc import Callable


class KossResult(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int32),
        ("value", ctypes.c_char_p),
    ]


class KossJS:
    RESULT_OK = 0
    RESULT_ERROR = 1
    RESULT_INVALID_ARG = 2

    def __init__(self, lib_path: str | None = None):
        if lib_path is None:
            lib_path = self._find_library()
        
        self._lib = ctypes.CDLL(lib_path)
        self._setup_prototypes()
        self._ptr = self._lib.koss_create()
        
        if not self._ptr:
            raise RuntimeError("Failed to create KossJS instance")
    
    def _find_library(self) -> str:
        base_dir = Path(__file__).parent
        if sys.platform == "win32":
            return str(base_dir / "target" / "debug" / "kossjs.dll")
        elif sys.platform == "darwin":
            return str(base_dir / "target" / "debug" / "libkossjs.dylib")
        else:
            return str(base_dir / "target" / "debug" / "libkossjs.so")
    
    def _setup_prototypes(self):
        lib = self._lib
        
        lib.koss_create.restype = ctypes.c_void_p
        lib.koss_create.argtypes = []
        
        lib.koss_destroy.argtypes = [ctypes.c_void_p]
        
        lib.koss_eval.restype = KossResult
        lib.koss_eval.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_run_file.restype = KossResult
        lib.koss_run_file.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_run_string.restype = KossResult
        lib.koss_run_string.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_set_global_string.restype = KossResult
        lib.koss_set_global_string.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_set_global_number.restype = KossResult
        lib.koss_set_global_number.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_double]
        
        lib.koss_set_global_bool.restype = KossResult
        lib.koss_set_global_bool.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_bool]
        
        lib.koss_register_function.restype = KossResult
        lib.koss_register_function.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_free_string.argtypes = [ctypes.c_void_p]
        lib.koss_free_result.argtypes = [KossResult]
        
        lib.koss_version.restype = ctypes.c_char_p
        lib.koss_version.argtypes = []
    
    def _check_result(self, result: KossResult) -> str:
        raw_value = result.value
        if raw_value:
            if isinstance(raw_value, bytes):
                value = raw_value.decode("utf-8")
            else:
                value = bytes(raw_value).decode("utf-8")
        else:
            value = ""
        self._lib.koss_free_result(result)
        
        if result.code == self.RESULT_OK:
            return value
        elif result.code == self.RESULT_ERROR:
            raise JsError(value)
        else:
            raise ValueError(f"Invalid argument: {value}")
    
    def eval(self, code: str) -> str:
        """Evaluate JavaScript code and return the result as a string."""
        result = self._lib.koss_eval(self._ptr, code.encode("utf-8"))
        return self._check_result(result)
    
    def run_file(self, path: str) -> str:
        """Execute a JavaScript file and return the result."""
        result = self._lib.koss_run_file(self._ptr, path.encode("utf-8"))
        return self._check_result(result)
    
    def run_string(self, code: str) -> str:
        """Execute a JavaScript string and return the result."""
        result = self._lib.koss_run_string(self._ptr, code.encode("utf-8"))
        return self._check_result(result)
    
    def set_global(self, name: str, value: Any) -> None:
        """Set a global variable in the JavaScript context."""
        name_bytes = name.encode("utf-8")
        
        if isinstance(value, str):
            result = self._lib.koss_set_global_string(self._ptr, name_bytes, value.encode("utf-8"))
        elif isinstance(value, (int, float)):
            result = self._lib.koss_set_global_number(self._ptr, name_bytes, float(value))
        elif isinstance(value, bool):
            result = self._lib.koss_set_global_bool(self._ptr, name_bytes, value)
        else:
            raise TypeError(f"Unsupported type: {type(value)}")
        
        self._check_result(result)
    
    def register_function(self, name: str, func: Callable) -> None:
        """Register a Python function callable from JavaScript."""
        if sys.platform == "win32":
            libc = ctypes.CDLL('msvcrt.dll')
        else:
            libc = ctypes.CDLL(ctypes.util.find_library('c'))
        
        libc.malloc.argtypes = [ctypes.c_size_t]
        libc.malloc.restype = ctypes.c_void_p
        libc.free.argtypes = [ctypes.c_void_p]
        
        if not hasattr(self, "_callback_allocations"):
            self._callback_allocations = []
        
        def wrapper(argc: int, argv: ctypes.c_void_p) -> ctypes.c_void_p:
            try:
                args = []
                for i in range(argc):
                    str_ptr = ctypes.cast(argv + i * ctypes.sizeof(ctypes.c_char_p), ctypes.POINTER(ctypes.c_char_p))[0]
                    args.append(ctypes.string_at(str_ptr).decode("utf-8"))
                result = func(*args)
                if result is None:
                    return None
                encoded = result.encode('utf-8')
                size = len(encoded) + 1
                buf = libc.malloc(size)
                if not buf:
                    return None
                ctypes.memmove(buf, encoded, len(encoded))
                ctypes.memset(ctypes.c_void_p(buf + len(encoded)), 0, 1)
                self._callback_allocations.append(buf)
                return buf
            except Exception as e:
                import traceback
                print(f"Callback error: {e}")
                traceback.print_exc()
                return None
        
        CALLBACK_TYPE = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p)
        wrapped = CALLBACK_TYPE(wrapper)
        name_bytes = name.encode("utf-8")
        result = self._lib.koss_register_function(self._ptr, name_bytes, wrapped)
        self._check_result(result)
        
        if not hasattr(self, "_callbacks"):
            self._callbacks = []
        self._callbacks.append(wrapped)
    
    def register_fetch(self) -> None:
        """Register the built-in fetch function using Python's urllib."""
        import urllib.request
        import json as json_module
        
        def fetch_impl(url: str, method: str = "GET", headers_str: str = "{}", body: str = "") -> str:
            try:
                headers = json_module.loads(headers_str) if headers_str else {}
                
                req = urllib.request.Request(url, data=body.encode() if body else None, method=method)
                for k, v in headers.items():
                    req.add_header(k, v)
                
                with urllib.request.urlopen(req, timeout=30) as response:
                    body_bytes = response.read()
                    status_code = response.status
                    status_text = response.reason
                    
                    response_headers = {}
                    for k, v in response.getheaders():
                        response_headers[k] = v
                    
                    result = {
                        "ok": status_code >= 200 and status_code < 300,
                        "status": status_code,
                        "statusText": status_text,
                        "body": body_bytes.decode("utf-8", errors="replace"),
                        "headers": response_headers
                    }
                    return json_module.dumps(result)
            except urllib.error.HTTPError as e:
                body = e.read().decode("utf-8", errors="replace")
                result = {
                    "ok": False,
                    "status": e.code,
                    "statusText": e.reason,
                    "body": body,
                    "headers": dict(e.headers)
                }
                return json_module.dumps(result)
            except Exception as e:
                result = {
                    "ok": False,
                    "status": 0,
                    "statusText": str(e),
                    "body": "",
                    "headers": {}
                }
                return json_module.dumps(result)
        
        self.register_function("__fetch", fetch_impl)
        
        fetch_js = '''
        function fetch(url, options) {
            options = options || {};
            const headers = options.headers || {};
            const headersJson = JSON.stringify(headers);
            const body = options.body || "";
            const method = options.method || "GET";
            
            const result = __fetch(url, method, headersJson, body);
            const response = JSON.parse(result);
            const _body = response.body;
            
            response.text = function() { return _body || ""; };
            response.json = function() { 
                try { return JSON.parse(_body || "null"); } 
                catch(e) { return null; }
            };
            
            return response;
        }
        '''
        self.eval(fetch_js)
    
    def version(self) -> str:
        """Get the KossJS version string."""
        return self._lib.koss_version().decode("utf-8")
    
    def destroy(self) -> None:
        """Destroy the JavaScript instance and free memory."""
        if self._ptr:
            self._lib.koss_destroy(self._ptr)
            self._ptr = None
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()
        return False
    
    def __del__(self):
        self.destroy()


class JsError(Exception):
    """Exception raised when JavaScript code throws an error."""
    pass
