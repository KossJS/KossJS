"""
KossJS Python Interface - Embeddable JavaScript runtime for Python
"""

import ctypes
import ctypes.util
import os # pyright: ignore[reportUnusedImport]
import sys
import json
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

    # Capability flags (must match KossCapability in include/kossjs.h)
    # 文件系统（6 个细粒度操作）
    FS_READ = 1 << 0
    FS_WRITE = 1 << 1
    FS_DELETE = 1 << 2
    FS_MKDIR = 1 << 3
    FS_RENAME = 1 << 4
    FS_CHMOD = 1 << 5

    # 网络（5 个细粒度操作）
    NET_TCP_CLIENT = 1 << 6
    NET_TCP_SERVER = 1 << 7
    NET_UDP = 1 << 8
    NET_DNS = 1 << 9
    NET_FETCH = 1 << 10

    # 加密（4 个细粒度操作）
    CRYPTO_HASH = 1 << 11
    CRYPTO_HMAC = 1 << 12
    CRYPTO_RANDOM = 1 << 13
    CRYPTO_PBKDF2 = 1 << 14

    # 内置 FFI（5 个细粒度操作）
    FFI_OPEN = 1 << 15
    FFI_CALL = 1 << 16
    FFI_ALLOC = 1 << 17
    FFI_CALLBACK = 1 << 18
    FFI_STRUCT = 1 << 19

    # 其他模块（8 个操作）
    NATIVE_ADDON = 1 << 20
    WASM = 1 << 21
    SHARED_MEMORY = 1 << 22
    HIGHRES_TIME = 1 << 23
    SYSINFO = 1 << 24
    MODULE_LOAD = 1 << 25
    DYNAMIC_CODE = 1 << 26
    DEBUG_CAP = 1 << 27

    # 组合常量
    KOSS_CAP_SANDBOX = 0
    KOSS_CAP_ALL_FS = FS_READ | FS_WRITE | FS_DELETE | FS_MKDIR | FS_RENAME | FS_CHMOD
    KOSS_CAP_ALL_NET = NET_TCP_CLIENT | NET_TCP_SERVER | NET_UDP | NET_DNS | NET_FETCH
    KOSS_CAP_ALL_CRYPTO = CRYPTO_HASH | CRYPTO_HMAC | CRYPTO_RANDOM | CRYPTO_PBKDF2
    KOSS_CAP_ALL_FFI = FFI_OPEN | FFI_CALL | FFI_ALLOC | FFI_CALLBACK | FFI_STRUCT
    KOSS_CAP_ALL = 0xFFFFFFFF

    # 兼容别名（用于旧宿主代码过渡）
    KOSS_CAP_FS = KOSS_CAP_ALL_FS
    KOSS_CAP_NET = KOSS_CAP_ALL_NET
    KOSS_CAP_CRYPTO = KOSS_CAP_ALL_CRYPTO
    KOSS_CAP_WORKER = 1 << 3
    KOSS_CAP_EXTERNAL_LOADER = MODULE_LOAD

    def __init__(
        self,
        lib_path: str | None = None,
        with_modules: bool = False,
        root_dir: str | None = None,
        capabilities: int | None = None,
        stable: bool = True,
    ):
        """
        Create a KossJS instance.

        :param lib_path: Path to the kossjs shared library. Auto-detected if None.
        :param with_modules: Enable ES module loading.
        :param root_dir: Base directory for module resolution.
        :param capabilities: Capability bitmask. None defaults to KOSS_CAP_ALL.
        :param stable: If True (default), disables FFI and Worker capabilities.
                       If False, enables these experimental features and prints
                       warnings to stderr. Production environments should keep
                       the default.

        Examples:
            # Production (stable mode, FFI/Worker disabled)
            runtime = KossJS(stable=True)  # default

            # Development/debugging (unstable mode, FFI/Worker enabled)
            runtime = KossJS(stable=False)
        """
        self._ptr: ctypes.c_void_p | None = None
        if lib_path is None:
            lib_path = self._find_library()
        
        self._lib = ctypes.CDLL(lib_path)
        self._setup_prototypes()

        if sys.platform == "win32":
            self._libc = ctypes.CDLL('msvcrt.dll')
        else:
            self._libc = ctypes.CDLL(ctypes.util.find_library('c'))
        self._libc.malloc.argtypes = [ctypes.c_size_t]
        self._libc.malloc.restype = ctypes.c_void_p
        
        caps = capabilities if capabilities is not None else self.KOSS_CAP_ALL

        # Use with_modules to enable module loading from stdlib
        if with_modules and root_dir:
            self._ptr = self._lib.koss_create_with_modules_and_caps(
                root_dir.encode("utf-8"), caps, stable
            )
        else:
            # Still use with_modules but with current directory
            self._ptr = self._lib.koss_create_with_modules_and_caps(b".", caps, stable)
        
        if not self._ptr:
            raise RuntimeError("Failed to create KossJS instance")
        
        # External module loader: called as a fallback when the module is not
        # found in the embedded stdlib (e.g. custom modules on disk).
        self.register_module_loader()
        # self.register_fetch()
    
    def _find_library(self) -> str:
        base_dir = Path(__file__).parent
        if sys.platform == "win32":
            return str(base_dir / "kossjs.dll")
        elif sys.platform == "darwin":
            return str(base_dir / "kossjs.dylib")
        else:
            return str(base_dir / "kossjs.so")
    
    def _setup_prototypes(self):
        lib = self._lib
        
        lib.koss_create_with_caps.restype = ctypes.c_void_p
        lib.koss_create_with_caps.argtypes = [ctypes.c_uint32, ctypes.c_bool]

        lib.koss_create_with_modules_and_caps.restype = ctypes.c_void_p
        lib.koss_create_with_modules_and_caps.argtypes = [ctypes.c_char_p, ctypes.c_uint32, ctypes.c_bool]

        lib.koss_is_stable.restype = ctypes.c_bool
        lib.koss_is_stable.argtypes = [ctypes.c_void_p]

        lib.koss_get_capabilities.restype = ctypes.c_uint32
        lib.koss_get_capabilities.argtypes = [ctypes.c_void_p]

        lib.koss_set_audit_mask.restype = KossResult
        lib.koss_set_audit_mask.argtypes = [ctypes.c_void_p, ctypes.c_uint32]

        lib.koss_get_audit_mask.restype = ctypes.c_uint32
        lib.koss_get_audit_mask.argtypes = [ctypes.c_void_p]
        
        lib.koss_destroy.argtypes = [ctypes.c_void_p]
        
        lib.koss_eval.restype = KossResult
        lib.koss_eval.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        
        lib.koss_run_file.restype = KossResult
        lib.koss_run_file.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_run_module.restype = KossResult
        lib.koss_run_module.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_run_string.restype = KossResult
        lib.koss_run_string.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_run_module_string.restype = KossResult
        lib.koss_run_module_string.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_set_global_string.restype = KossResult
        lib.koss_set_global_string.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        
        lib.koss_set_global_number.restype = KossResult
        lib.koss_set_global_number.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_double]
        
        lib.koss_set_global_bool.restype = KossResult
        lib.koss_set_global_bool.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_bool]
        
        lib.koss_set_global_null.restype = KossResult
        lib.koss_set_global_null.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_set_global_undefined.restype = KossResult
        lib.koss_set_global_undefined.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_set_global_json.restype = KossResult
        lib.koss_set_global_json.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        
        lib.koss_register_function.restype = KossResult
        lib.koss_register_function.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_void_p]
        
        lib.koss_free_string.argtypes = [ctypes.c_void_p]
        lib.koss_free_result.argtypes = [KossResult]
        
        lib.koss_version.restype = ctypes.c_char_p
        lib.koss_version.argtypes = []
        
        lib.koss_get_binding.restype = KossResult
        lib.koss_get_binding.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        
        lib.koss_fetch.restype = KossResult
        lib.koss_fetch.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

        lib.koss_run_async.restype = KossResult
        lib.koss_run_async.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint64]

        lib.koss_tick.restype = KossResult
        lib.koss_tick.argtypes = [ctypes.c_void_p]

        lib.koss_create_worker_pool.restype = KossResult
        lib.koss_create_worker_pool.argtypes = [ctypes.c_void_p, ctypes.c_int32]

        lib.koss_worker_post_message.restype = KossResult
        lib.koss_worker_post_message.argtypes = [ctypes.c_void_p, ctypes.c_int32, ctypes.c_char_p]

        lib.koss_worker_execute.restype = KossResult
        lib.koss_worker_execute.argtypes = [ctypes.c_void_p, ctypes.c_int32, ctypes.c_char_p]

        lib.koss_worker_try_recv.restype = KossResult
        lib.koss_worker_try_recv.argtypes = [ctypes.c_void_p]

        lib.koss_worker_terminate.restype = KossResult
        lib.koss_worker_terminate.argtypes = [ctypes.c_void_p, ctypes.c_int32]

        lib.koss_worker_shutdown.restype = KossResult
        lib.koss_worker_shutdown.argtypes = [ctypes.c_void_p]

        lib.koss_register_module_loader.restype = KossResult
        lib.koss_register_module_loader.argtypes = [ctypes.c_void_p, ctypes.c_void_p]

        lib.koss_register_class.restype = KossResult
        lib.koss_register_class.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_void_p]

        lib.koss_enable_audit_debug.restype = None
        lib.koss_enable_audit_debug.argtypes = [ctypes.c_void_p, ctypes.c_bool]
    
    @property
    def is_stable(self) -> bool:
        """Check if this instance is running in stable mode."""
        return self._lib.koss_is_stable(self._ptr)

    def get_capabilities(self) -> int:
        """Get the capability bitmask for this instance."""
        return self._lib.koss_get_capabilities(self._ptr)

    def _get_binding(self, name: str) -> dict[str, Any]:
        """Get internal binding info from Rust."""
        result = self._lib.koss_get_binding(self._ptr, name.encode("utf-8"))
        value = self._check_result(result)
        return json.loads(value) if value else {}
    
    def _check_result(self, result: KossResult) -> str:
        if self._ptr is None:
            raise RuntimeError("KossJS instance has been destroyed")
        raw_value = result.value
        if raw_value:
            if isinstance(raw_value, bytes):
                value = raw_value.decode("utf-8", errors="replace")
            else:
                value = bytes(raw_value).decode("utf-8", errors="replace")
        else:
            value = ""
        self._lib.koss_free_result(result)
        
        if result.code == self.RESULT_OK:
            return value
        elif result.code == self.RESULT_ERROR:
            raise JsError(value)
        else:
            raise ValueError(f"Invalid argument: {value}")
    
    def eval(self, code: str) -> Any:
        """Evaluate JavaScript code synchronously and return the result.
        
        If the result is a JSON object/array string, it is automatically parsed.
        Primitive values (strings, numbers, booleans) are returned as plain strings.
        """
        import json
        value = self._check_result(self._lib.koss_eval(self._ptr, code.encode("utf-8")))
        if value and value[0] in ('{', '['):
            try:
                return json.loads(value)
            except json.JSONDecodeError:
                pass
        return value

    def run_async(self, code: str, timeout_ms: int = 30000) -> str:
        """Evaluate JavaScript code and drive the async event loop to completion.
        
        Use this for code that uses await/async (e.g., fetch() which returns a Promise).
        The event loop processes async I/O and drains microtasks for up to timeout_ms.
        """
        result = self._lib.koss_run_async(self._ptr, code.encode("utf-8"), timeout_ms)
        return self._check_result(result)

    def tick(self) -> bool:
        """Run one iteration of the event loop. Returns True if pending async ops remain."""
        result = self._lib.koss_tick(self._ptr)
        val = self._check_result(result)
        return val == "1"

    def create_worker_pool(self, size: int) -> str:
        """Create a pool of worker threads for parallel execution."""
        result = self._lib.koss_create_worker_pool(self._ptr, size)
        return self._check_result(result)

    def worker_post_message(self, worker_id: int, data: str) -> str:
        """Post a JSON message to a worker thread."""
        result = self._lib.koss_worker_post_message(self._ptr, worker_id, data.encode("utf-8"))
        return self._check_result(result)

    def worker_execute(self, worker_id: int, code: str) -> str:
        """Execute JavaScript code on a worker thread."""
        result = self._lib.koss_worker_execute(self._ptr, worker_id, code.encode("utf-8"))
        return self._check_result(result)

    def worker_try_recv(self) -> str | None:
        """Try to receive a message from any worker (non-blocking). Returns None if no message."""
        result = self._lib.koss_worker_try_recv(self._ptr)
        try:
            val = self._check_result(result)
            if val == "null" or not val:
                return None
            return val
        except Exception:
            import logging
            logging.getLogger("KossJS").warning("worker_try_recv: failed to parse result", exc_info=True)
            return None

    def worker_terminate(self, worker_id: int) -> str:
        """Terminate a specific worker thread."""
        result = self._lib.koss_worker_terminate(self._ptr, worker_id)
        return self._check_result(result)

    def worker_shutdown(self) -> str:
        """Shut down all worker threads and clean up the pool."""
        result = self._lib.koss_worker_shutdown(self._ptr)
        return self._check_result(result)
    
    def run_file(self, path: str) -> str:
        """Execute a JavaScript file and return the result."""
        result = self._lib.koss_run_file(self._ptr, path.encode("utf-8"))
        return self._check_result(result)
    
    def run_module(self, path: str) -> str:
        """Execute a JavaScript module and return the result."""
        result = self._lib.koss_run_module(self._ptr, path.encode("utf-8"))
        return self._check_result(result)

    def run_string(self, code: str) -> str:
        """Execute a JavaScript string and return the result."""
        result = self._lib.koss_run_string(self._ptr, code.encode("utf-8"))
        return self._check_result(result)
    
    def run_module_string(self, code: str) -> str:
        """Execute a JavaScript module string code and return the result."""
        result = self._lib.koss_run_module_string(self._ptr, code.encode("utf-8"))
        return self._check_result(result)

    def set_global(self, name: str, value: Any) -> None:
        """Set a global variable in the JavaScript context.

        Supports: str, int, float, bool, None, list, dict, and JavaScript undefined.
        Lists and dicts are serialized to JSON for object/array support.
        """
        name_bytes = name.encode("utf-8")

        if value is None:
            result = self._lib.koss_set_global_null(self._ptr, name_bytes)
        elif isinstance(value, bool):
            # bool must be checked before int because bool is subclass of int
            result = self._lib.koss_set_global_bool(self._ptr, name_bytes, value)
        elif isinstance(value, str):
            if value == "__undefined__":
                result = self._lib.koss_set_global_undefined(self._ptr, name_bytes)
            else:
                result = self._lib.koss_set_global_string(self._ptr, name_bytes, value.encode("utf-8"))
        elif isinstance(value, (int, float)):
            result = self._lib.koss_set_global_number(self._ptr, name_bytes, float(value))
        elif isinstance(value, (list, dict)):
            json_str = json.dumps(value, ensure_ascii=False).encode("utf-8")
            result = self._lib.koss_set_global_json(self._ptr, name_bytes, json_str)
        else:
            json_str = json.dumps(value, ensure_ascii=False).encode("utf-8")
            result = self._lib.koss_set_global_json(self._ptr, name_bytes, json_str)

        self._check_result(result)
    
    def register_function(self, name: str, func: Callable[..., Any]) -> None:
        """Register a Python function callable from JavaScript.

        Supports dotted paths for mounting to nested objects:
          register_function("Math.max", fn)  -> globalThis.Math.max = fn
          register_function("console.log", fn) -> globalThis.console.log = fn
        """
        libc = self._libc

        def wrapper(argc: int, argv: ctypes.c_void_p) -> ctypes.c_void_p | None:
            try:
                args: list[str] = []
                for i in range(argc):
                    str_ptr: int = ctypes.cast(argv + i * ctypes.sizeof(ctypes.c_char_p), ctypes.POINTER(ctypes.c_char_p))[0] # pyright: ignore[reportOperatorIssue, reportUnknownArgumentType]
                    if not str_ptr:
                        args.append("")
                    else:
                        args.append(ctypes.string_at(str_ptr).decode("utf-8", errors="replace"))
                result = func(*args)
                if result is None:
                    return None
                encoded = result.encode('utf-8')
                size = len(encoded) + 1
                buf: ctypes.c_void_p = libc.malloc(size)
                if not buf:
                    return None
                ctypes.memmove(buf, encoded, len(encoded))
                ctypes.memset(ctypes.c_void_p(int(buf) + len(encoded)), 0, 1) # pyright: ignore[reportOperatorIssue, reportUnknownArgumentType]
                return buf
            except Exception as e:
                import traceback
                print(f"Callback error: {e}")
                traceback.print_exc()
                return None

        CALLBACK_TYPE = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p)
        wrapped: ctypes._CFunctionType = CALLBACK_TYPE(wrapper) # pyright: ignore[reportPrivateUsage]
        name_bytes = name.encode("utf-8")
        result = self._lib.koss_register_function(self._ptr, name_bytes, wrapped)
        self._check_result(result)

        if not hasattr(self, "_callbacks"):
            self._callbacks: list[ctypes._CFunctionType] = [] # pyright: ignore[reportPrivateUsage]
        self._callbacks.append(wrapped)
    
    # def register_fetch(self) -> None:
    #     """Register the built-in fetch function using Python's urllib."""
    #     import urllib.request
    #     import json as json_module
        
    #     def fetch_impl(url: str, method: str = "GET", headers_str: str = "{}", body: str = "") -> str: # pyright: ignore[reportRedeclaration]
    #         try:
    #             headers: dict[str, Any] = json_module.loads(headers_str) if headers_str else {}
                
    #             req = urllib.request.Request(url, data=body.encode() if body else None, method=method)
    #             for k, v in headers.items():
    #                 req.add_header(k, v)
                
    #             with urllib.request.urlopen(req, timeout=30) as response:
    #                 body_bytes = response.read()
    #                 status_code = response.status
    #                 status_text = response.reason
                    
    #                 response_headers = {}
    #                 for k, v in response.getheaders():
    #                     response_headers[k] = v
                    
    #                 result: dict[str, Any] = {
    #                     "ok": status_code >= 200 and status_code < 300,
    #                     "status": status_code,
    #                     "statusText": status_text,
    #                     "body": body_bytes.decode("utf-8", errors="replace"),
    #                     "headers": response_headers
    #                 }
    #                 return json_module.dumps(result)
    #         except urllib.error.HTTPError as e: # pyright: ignore[reportUnknownVariableType, reportAttributeAccessIssue, reportUnknownMemberType]
    #             body: str = e.read().decode("utf-8", errors="replace") # pyright: ignore[reportUnknownMemberType, reportUnknownVariableType]
    #             result: dict[str, Any] = {
    #                 "ok": False,
    #                 "status": e.code, # pyright: ignore[reportUnknownMemberType]
    #                 "statusText": e.reason, # pyright: ignore[reportUnknownMemberType]
    #                 "body": body,
    #                 "headers": dict(e.headers) # pyright: ignore[reportUnknownMemberType, reportUnknownArgumentType]
    #             }
    #             return json_module.dumps(result)
    #         except Exception as e:
    #             result = {
    #                 "ok": False,
    #                 "status": 0,
    #                 "statusText": str(e),
    #                 "body": "",
    #                 "headers": {}
    #             }
    #             return json_module.dumps(result)
        
    #     self.register_function("__fetch", fetch_impl)
        
    #     fetch_js = '''
    #     function fetch(url, options) {
    #         options = options || {};
    #         const headers = options.headers || {};
    #         const headersJson = JSON.stringify(headers);
    #         const body = options.body || "";
    #         const method = options.method || "GET";
            
    #         const result = __fetch(url, method, headersJson, body);
    #         const response = JSON.parse(result);
    #         const _body = response.body;
            
    #         response.text = function() { return _body || ""; };
    #         response.json = function() { 
    #             try { return JSON.parse(_body || "null"); } 
    #             catch(e) { return null; }
    #         };
            
    #         return response;
    #     }
    #     '''
    #     self.eval(fetch_js)
    
    # def register_fetch(self, fetch_func=None) -> None:
    #     """Register the built-in fetch function and load the fetch module."""
    #     import urllib.request
    #     import json as json_module
        
    #     if fetch_func is None:
    #         def default_fetch(url: str, options: str = "") -> str:
    #             try:
    #                 opts: dict[str, Any] = json_module.loads(options) if options else {}
    #             except:
    #                 opts = {}
                
    #             headers: dict[str, Any] = opts.get('headers', {})
    #             method: str = opts.get('method', 'GET')
    #             body: str = opts.get('body', '')
                
    #             req = urllib.request.Request(url, data=body.encode() if body else None, method=method)
    #             for k, v in headers.items():
    #                 req.add_header(k, v)
                
    #             try:
    #                 with urllib.request.urlopen(req, timeout=30) as response:
    #                     body_bytes = response.read()
    #                     status_code = response.status
    #                     status_text = response.reason
                        
    #                     response_headers = {}
    #                     for k, v in response.getheaders():
    #                         response_headers[k] = v
                        
    #                     result: dict[str, Any] = {
    #                         "ok": status_code >= 200 and status_code < 300,
    #                         "status": status_code,
    #                         "statusText": status_text,
    #                         "body": body_bytes.decode("utf-8", errors="replace"),
    #                         "headers": response_headers
    #                     }
    #                     return json_module.dumps(result)
    #             except urllib.error.HTTPError as e:
    #                 body = e.read().decode("utf-8", errors="replace")
    #                 result = {
    #                     "ok": False,
    #                     "status": e.code,
    #                     "statusText": e.reason,
    #                     "body": body,
    #                     "headers": dict(e.headers)
    #                 }
    #                 return json_module.dumps(result)
    #             except Exception as e:
    #                 result = {
    #                     "ok": False,
    #                     "status": 0,
    #                     "statusText": str(e),
    #                     "body": "",
    #                     "headers": {}
    #                 }
    #                 return json_module.dumps(result)
            
    #         fetch_func = default_fetch
        
    #     self.register_function("__fetch", fetch_func)
        
    #     # Use simple fetch implementation
    #     fetch_module_path = str(Path(__file__).parent / "src" / "fetch" / "simple-fetch.js")
    #     if Path(fetch_module_path).exists():
    #         self.run_module(fetch_module_path)
    #     else:
    #         raise FileNotFoundError(f"Fetch module not found at: {fetch_module_path}")
    
    def register_module_loader(self) -> None:
        """Register the module loader that loads Node.js stdlib modules."""
        import json
        
        libc = self._libc
        
        def wrapper(argc: int, argv: ctypes.c_void_p) -> ctypes.c_void_p | None:
            try:
                if argc < 1:
                    return None
                
                # Get the string pointer safely
                argv_array = ctypes.cast(argv, ctypes.POINTER(ctypes.c_char_p))
                str_ptr = argv_array[0]
                if not str_ptr:
                    return None
                    
                module_path = ctypes.string_at(str_ptr).decode("utf-8", errors="replace")
                
                result: dict[str, Any] | None = koss_module_loader(module_path)
                
                if result is None:
                    # Return empty object instead of null
                    return None
                
                result_json = json.dumps(result)
                encoded = result_json.encode('utf-8')
                size = len(encoded) + 1
                buf: ctypes.c_void_p = libc.malloc(size)
                if not buf:
                    return None
                ctypes.memmove(buf, encoded, len(encoded))
                ctypes.memset(ctypes.c_void_p(int(buf) + len(encoded)), 0, 1)
                return buf
            except Exception as e:
                import traceback
                print(f"Module loader error: {e}")
                traceback.print_exc()
                return None
        
        CALLBACK_TYPE = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p)
        wrapped: ctypes._CFunctionType = CALLBACK_TYPE(wrapper) # pyright: ignore[reportPrivateUsage]
        
        result = self._lib.koss_register_module_loader(self._ptr, wrapped)
        self._check_result(result)
        
        if not hasattr(self, "_module_loader_callback"):
            self._module_loader_callback: list[ctypes._CFunctionType] = [] # pyright: ignore[reportPrivateUsage]
        self._module_loader_callback.append(wrapped)
    
    def register_class(self, class_name: str, methods: dict[str, Callable[..., Any]]) -> None:
        """Register a Python class/methods object as a JavaScript class.

        Args:
            class_name: Name for the JavaScript constructor (e.g. "MyClass")
            methods: Dict of {method_name: callable} to expose on instances

        The created JS class supports `new` keyword:
            const obj = new MyClass();
            obj.method1(arg1, arg2);

        Each method call invokes the Python callable with string arguments.
        The method should return a string (or None for undefined).
        """
        libc = self._libc

        # Build method dispatch list
        method_names = list(methods.keys())

        # Create a C callback that dispatches by method name
        # The callback receives: (method_name: str, ...args)
        def class_callback(argc: int, argv: ctypes.c_void_p) -> ctypes.c_void_p | None:
            args: list[str] = []
            try:
                for i in range(argc):
                    str_ptr: int = ctypes.cast(argv + i * ctypes.sizeof(ctypes.c_char_p), ctypes.POINTER(ctypes.c_char_p))[0] # pyright: ignore[reportOperatorIssue, reportUnknownArgumentType]
                    if not str_ptr:
                        args.append("")
                    else:
                        args.append(ctypes.string_at(str_ptr).decode("utf-8", errors="replace"))
                if not args:
                    return None
                method_name: str = args[0]
                method_args = args[1:]
                if method_name in methods:
                    result = methods[method_name](*method_args)
                    if result is None:
                        return None
                    encoded = str(result).encode('utf-8')
                    size = len(encoded) + 1
                    buf: ctypes.c_void_p = libc.malloc(size)
                    if not buf:
                        return None
                    ctypes.memmove(buf, encoded, len(encoded))
                    ctypes.memset(ctypes.c_void_p(int(buf) + len(encoded)), 0, 1)
                    return buf
                return None
            except Exception as e:
                method_name_str: str = args[0] if args else "?"
                import traceback
                print(f"Class callback error ({class_name}.{method_name_str}): {e}")
                traceback.print_exc()
                return None

        CALLBACK_TYPE = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p)
        wrapped: ctypes._CFunctionType = CALLBACK_TYPE(class_callback) # pyright: ignore[reportPrivateUsage]

        methods_json = json.dumps(method_names, ensure_ascii=False).encode("utf-8")
        name_bytes = class_name.encode("utf-8")
        result = self._lib.koss_register_class(self._ptr, name_bytes, methods_json, wrapped)
        self._check_result(result)

        if not hasattr(self, "_class_callbacks"):
            self._class_callbacks: list[ctypes._CFunctionType] = [] # pyright: ignore[reportPrivateUsage]
        self._class_callbacks.append(wrapped)

    def version(self) -> str:
        """Get the KossJS version string."""
        return self._lib.koss_version().decode("utf-8")

    def set_audit_mask(self, mask: int) -> None:
        """设置审核掩码（只能审核已授予的能力位）"""
        result = self._lib.koss_set_audit_mask(self._ptr, mask)
        self._check_result(result)

    def get_audit_mask(self) -> int:
        """获取当前审核掩码"""
        return self._lib.koss_get_audit_mask(self._ptr)

    def check_sandbox(self, func: Callable[[str, list[str], str | None], bool] | None = None) -> None:
        """Register or clear the synchronous audit callback.

        The callback receives (target, args_list, pwd) and returns True to allow
        the operation or False to block it (which throws KossSecurityError).

        Pass None to clear the audit callback.
        """
        if not hasattr(self, '_AUDIT_CALLBACK'):
            self._AUDIT_CALLBACK = ctypes.CFUNCTYPE(
                ctypes.c_bool,
                ctypes.c_char_p,
                ctypes.POINTER(ctypes.c_char_p),
                ctypes.c_int,
                ctypes.c_char_p,
                ctypes.c_void_p,
            )
            self._lib.koss_check_sandbox.restype = KossResult
            self._lib.koss_check_sandbox.argtypes = [ctypes.c_void_p, self._AUDIT_CALLBACK, ctypes.c_void_p]

        if func is None:
            null_cb = self._AUDIT_CALLBACK(0)
            self._check_result(self._lib.koss_check_sandbox(self._ptr, null_cb, None))
            self._audit_callback = None
            return

        def wrapper(target: bytes, args: ctypes.POINTER(ctypes.c_char_p), argc: int, pwd: bytes, userdata: ctypes.c_void_p) -> ctypes.c_bool:  # type: ignore[reportUnknownParameterType]
            target_s = target.decode("utf-8", errors="replace") if target else ""
            values: list[str] = []
            for i in range(argc):
                raw: bytes = args[i]  # type: ignore[reportUnknownVariableType]
                values.append(raw.decode("utf-8", errors="replace") if raw else "")  # type: ignore[reportUnknownMemberType, reportUnknownArgumentType]
            pwd_s = pwd.decode("utf-8", errors="replace") if pwd else None
            try:
                return ctypes.c_bool(bool(func(target_s, values, pwd_s)))
            except Exception:
                return ctypes.c_bool(False)

        cb = self._AUDIT_CALLBACK(wrapper)  # type: ignore[reportUnknownArgumentType]
        self._audit_callback = cb
        self._check_result(self._lib.koss_check_sandbox(self._ptr, cb, None))

    def enable_audit_debug(self, enable: bool) -> None:
        """Enable or disable audit debug mode.

        When debug mode is enabled:
        - Sync/async callback exceptions are output to stderr
        - Audit denial reasons include additional error information
        - Async audit timeouts or hangs log warnings
        - Rejection reentry logs current depth and configured max depth

        Production environments should disable debug mode to avoid information leakage.
        """
        self._lib.koss_enable_audit_debug(self._ptr, bool(enable))

    def destroy(self) -> None:
        """Destroy the JavaScript instance and free memory."""
        if self._ptr and hasattr(self, '_lib') and self._lib:
            self._lib.koss_destroy(self._ptr)
            self._ptr = None
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb): # pyright: ignore[reportUnknownParameterType, reportMissingParameterType]
        self.destroy()
        return False
    
    def __del__(self):
        self.destroy()


class JsError(Exception):
    """Exception raised when JavaScript code throws an error."""
    pass

def koss_module_loader(module_path: str) -> dict[str, Any] | None:
    """Load a Node.js module from the stdlib.
    
    This function is called by the JS require() when it can't find a module
    in the regular module cache. It loads modules from src/stdlib.
    Supports sub-modules like path/posix -> src/stdlib/path/posix.js
    """
    base_dir = Path(__file__).parent.resolve()
    # Walk up from test/ to project root to find src/stdlib/
    stdlib_dir = base_dir.parent / "src" / "stdlib"
    
    # Handle node: prefix
    if module_path.startswith("node:"):
        module_path = module_path[5:]
    
    # List of built-in modules
    builtin_modules = [
        'assert', 'buffer', 'child_process', 'cluster', 'console', 'constants',
        'crypto', 'dgram', 'dns', 'domain', 'events', 'fs', 'http', 'https',
        'module', 'net', 'os', 'path', 'process', 'punycode', 'querystring',
        'readline', 'repl', 'stream', 'string_decoder', 'sys', 'timers',
        'tls', 'trace_events', 'tty', 'url', 'util', 'v8', 'vm', 'wasi',
        'worker_threads', 'zlib', 'async_hooks', 'diagnostics_channel',
        'perf_hooks', 'http2', 'http3', 'sqlite', 'test', 'wasi', 'sea'
    ]
    
    # Check if it's a sub-module (e.g., "path/posix")
    if "/" in module_path:
        parts = module_path.split("/", 1)
        base_module = parts[0]
        sub_path = parts[1]
        sanitized_base = os.path.basename(base_module)
        sanitized_sub = os.path.basename(sub_path)
        if sanitized_base != base_module:
            return None
        # Try: stdlib/path/posix.js
        js_file = stdlib_dir / sanitized_base / f"{sanitized_sub}.js"
        if js_file.exists():
            resolved = js_file.resolve()
            if not str(resolved).startswith(str(stdlib_dir.resolve())):
                return None
            with open(js_file, 'r', encoding='utf-8') as f:
                return {"type": "module", "code": f.read()}
        # Try: stdlib/path/posix/index.js
        js_file = stdlib_dir / sanitized_base / sanitized_sub / "index.js"
        if js_file.exists():
            resolved = js_file.resolve()
            if not str(resolved).startswith(str(stdlib_dir.resolve())):
                return None
            with open(js_file, 'r', encoding='utf-8') as f:
                return {"type": "module", "code": f.read()}
        # Fall through to module not found
    
    # Check if it's a built-in module (full name match)
    if module_path in builtin_modules:
        # Try to load from stdlib
        sanitized = os.path.basename(module_path)
        if sanitized != module_path:
            return None
        js_file = stdlib_dir / f"{sanitized}.js"
        if js_file.exists():
            resolved = js_file.resolve()
            if not str(resolved).startswith(str(stdlib_dir.resolve())):
                return None
            with open(js_file, 'r', encoding='utf-8') as f:
                return {"type": "module", "code": f.read()}
        
        # Try as directory with index.js
        js_file = stdlib_dir / sanitized / "index.js"
        if js_file.exists():
            resolved = js_file.resolve()
            if not str(resolved).startswith(str(stdlib_dir.resolve())):
                return None
            with open(js_file, 'r', encoding='utf-8') as f:
                return {"type": "module", "code": f.read()}
    
    # Module not found
    return None
