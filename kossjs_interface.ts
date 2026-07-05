/**
 * Copyright (c) 2026 TT23XR Studio
 *
 * KossJS TypeScript Interface — Embeddable JavaScript runtime
 * Mirrors kossjs_interface.py using koffi for C ABI bindings.
 */


declare const __dirname: string;
declare const process: any;
declare const Buffer: any;
declare const require: any;

import { existsSync } from 'fs';
import { join } from 'path';
import { platform } from 'os';

const koffi = require('koffi');

// Use 'void *' NOT 'string' — koffi 'string' replaces the C pointer, causing
// koss_free_result to double-free / corrupt heap. Python ctypes.c_char_p
// works because it preserves the original pointer in struct memory.
const KossResult = koffi.struct({ code: 'int32', value: 'void *' });

const RESULT_OK = 0;
const RESULT_ERROR = 1;
const RESULT_INVALID_ARG = 2;

// --- Capability flags (must match KossCapability in include/kossjs.h) ---

// 文件系统（6 个细粒度操作）
const FS_READ        = 1 << 0;
const FS_WRITE       = 1 << 1;
const FS_DELETE      = 1 << 2;
const FS_MKDIR       = 1 << 3;
const FS_RENAME      = 1 << 4;
const FS_CHMOD       = 1 << 5;

// 网络（5 个细粒度操作）
const NET_TCP_CLIENT = 1 << 6;
const NET_TCP_SERVER = 1 << 7;
const NET_UDP        = 1 << 8;
const NET_DNS        = 1 << 9;
const NET_FETCH      = 1 << 10;

// 加密（4 个细粒度操作）
const CRYPTO_HASH    = 1 << 11;
const CRYPTO_HMAC    = 1 << 12;
const CRYPTO_RANDOM  = 1 << 13;
const CRYPTO_PBKDF2  = 1 << 14;

// 内置 FFI（5 个细粒度操作）
const FFI_OPEN       = 1 << 15;
const FFI_CALL       = 1 << 16;
const FFI_ALLOC      = 1 << 17;
const FFI_CALLBACK   = 1 << 18;
const FFI_STRUCT     = 1 << 19;

// 其他模块（8 个操作）
const NATIVE_ADDON   = 1 << 20;
const WASM           = 1 << 21;
const SHARED_MEMORY  = 1 << 22;
const HIGHRES_TIME   = 1 << 23;
const SYSINFO        = 1 << 24;
const MODULE_LOAD    = 1 << 25;
const DYNAMIC_CODE   = 1 << 26;
const DEBUG_CAP      = 1 << 27;

// 组合常量
const KOSS_CAP_SANDBOX = 0;
const KOSS_CAP_ALL_FS = FS_READ | FS_WRITE | FS_DELETE | FS_MKDIR | FS_RENAME | FS_CHMOD;
const KOSS_CAP_ALL_NET = NET_TCP_CLIENT | NET_TCP_SERVER | NET_UDP | NET_DNS | NET_FETCH;
const KOSS_CAP_ALL_CRYPTO = CRYPTO_HASH | CRYPTO_HMAC | CRYPTO_RANDOM | CRYPTO_PBKDF2;
const KOSS_CAP_ALL_FFI = FFI_OPEN | FFI_CALL | FFI_ALLOC | FFI_CALLBACK | FFI_STRUCT;
const KOSS_CAP_ALL = 0xFFFFFFFF;

// 兼容别名
const KOSS_CAP_FS = KOSS_CAP_ALL_FS;
const KOSS_CAP_NET = KOSS_CAP_ALL_NET;
const KOSS_CAP_CRYPTO = KOSS_CAP_ALL_CRYPTO;
const KOSS_CAP_WORKER = 1 << 3;
const KOSS_CAP_EXTERNAL_LOADER = MODULE_LOAD;

// --- Builtin Flags (must match KossBuiltin in include/kossjs.h) ---
const KOSS_BUILTIN_NONE = 0;
const KOSS_BUILTIN_NODE = 1 << 0;
const KOSS_BUILTIN_BUN  = 1 << 1;
const KOSS_BUILTIN_DENO = 1 << 2;
const KOSS_BUILTIN_KOSS = 1 << 3;
const KOSS_BUILTIN_ALL  = 0xFFFFFFFF;

function findLibrary(): string {
  let candidates: string[] = [];
  if (platform() === 'win32') {
    candidates = [
      join(__dirname, 'kossjs.dll'),
      join(process.cwd(), 'kossjs.dll'),
      join(process.cwd(), 'src', '__tests__', 'kossjs.dll'),
    ];
  } else if (platform() === 'darwin') {
    candidates = [
      join(__dirname, 'kossjs.dylib'),
      join(process.cwd(), 'kossjs.dylib'),
      join(process.cwd(), 'src', '__tests__', 'kossjs.dylib'),
    ];
  } else {
    candidates = [
      join(__dirname, 'kossjs.so'),
      join(process.cwd(), 'kossjs.so'),
      join(process.cwd(), 'src', '__tests__', 'kossjs.so'),
    ];
  }
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  throw new Error('kossjs library not found');
}

export class JsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'JsError';
  }
}

export class KossJS {
  static readonly RESULT_OK = RESULT_OK;
  static readonly RESULT_ERROR = RESULT_ERROR;
  static readonly RESULT_INVALID_ARG = RESULT_INVALID_ARG;

  // 能力位常量
  static readonly FS_READ = FS_READ;
  static readonly FS_WRITE = FS_WRITE;
  static readonly FS_DELETE = FS_DELETE;
  static readonly FS_MKDIR = FS_MKDIR;
  static readonly FS_RENAME = FS_RENAME;
  static readonly FS_CHMOD = FS_CHMOD;
  static readonly NET_TCP_CLIENT = NET_TCP_CLIENT;
  static readonly NET_TCP_SERVER = NET_TCP_SERVER;
  static readonly NET_UDP = NET_UDP;
  static readonly NET_DNS = NET_DNS;
  static readonly NET_FETCH = NET_FETCH;
  static readonly CRYPTO_HASH = CRYPTO_HASH;
  static readonly CRYPTO_HMAC = CRYPTO_HMAC;
  static readonly CRYPTO_RANDOM = CRYPTO_RANDOM;
  static readonly CRYPTO_PBKDF2 = CRYPTO_PBKDF2;
  static readonly FFI_OPEN = FFI_OPEN;
  static readonly FFI_CALL = FFI_CALL;
  static readonly FFI_ALLOC = FFI_ALLOC;
  static readonly FFI_CALLBACK = FFI_CALLBACK;
  static readonly FFI_STRUCT = FFI_STRUCT;
  static readonly NATIVE_ADDON = NATIVE_ADDON;
  static readonly WASM = WASM;
  static readonly SHARED_MEMORY = SHARED_MEMORY;
  static readonly HIGHRES_TIME = HIGHRES_TIME;
  static readonly SYSINFO = SYSINFO;
  static readonly MODULE_LOAD = MODULE_LOAD;
  static readonly DYNAMIC_CODE = DYNAMIC_CODE;
  static readonly DEBUG_CAP = DEBUG_CAP;
  static readonly KOSS_CAP_SANDBOX = KOSS_CAP_SANDBOX;
  static readonly KOSS_CAP_ALL_FS = KOSS_CAP_ALL_FS;
  static readonly KOSS_CAP_ALL_NET = KOSS_CAP_ALL_NET;
  static readonly KOSS_CAP_ALL_CRYPTO = KOSS_CAP_ALL_CRYPTO;
  static readonly KOSS_CAP_ALL_FFI = KOSS_CAP_ALL_FFI;
  static readonly KOSS_CAP_ALL = KOSS_CAP_ALL;
  static readonly KOSS_CAP_FS = KOSS_CAP_FS;
  static readonly KOSS_CAP_NET = KOSS_CAP_NET;
  static readonly KOSS_CAP_CRYPTO = KOSS_CAP_CRYPTO;
  static readonly KOSS_CAP_WORKER = KOSS_CAP_WORKER;
  static readonly KOSS_CAP_EXTERNAL_LOADER = KOSS_CAP_EXTERNAL_LOADER;

  // Builtin flags
  static readonly KOSS_BUILTIN_NONE = KOSS_BUILTIN_NONE;
  static readonly KOSS_BUILTIN_NODE = KOSS_BUILTIN_NODE;
  static readonly KOSS_BUILTIN_BUN  = KOSS_BUILTIN_BUN;
  static readonly KOSS_BUILTIN_DENO = KOSS_BUILTIN_DENO;
  static readonly KOSS_BUILTIN_KOSS = KOSS_BUILTIN_KOSS;
  static readonly KOSS_BUILTIN_ALL  = KOSS_BUILTIN_ALL;

  private _lib: any;
  private _ptr: any;
  private _msvcrt: any;
  private _strlen: Function;
  private _memcpy: Function;
  private _callbacks: any[] = [];
  private _auditCallback: any = null;
  private _moduleLoaderCallbacks: any[] = [];
  private _classCallbacks: any[] = [];

  // Cached function bindings
  private _fnVersion: Function;
  private _fnDestroy: Function;
  private _fnEval: Function;
  private _fnFreeResult: Function;
  private _fnGetCapabilities: Function;
  private _fnRunFile: Function;
  private _fnRunModule: Function;
  private _fnRunString: Function;
  private _fnRunModuleString: Function;
  private _fnRunAsync: Function;
  private _fnTick: Function;
  private _fnSetGlobalString: Function;
  private _fnSetGlobalNumber: Function;
  private _fnSetGlobalBool: Function;
  private _fnSetGlobalNull: Function;
  private _fnSetGlobalUndefined: Function;
  private _fnSetGlobalJson: Function;
  private _fnRegisterFunction: Function;
  private _fnRegisterModuleLoader: Function;
  private _fnRegisterClass: Function;
  private _fnGetBinding: Function;
  private _fnSetAuditMask: Function;
  private _fnGetAuditMask: Function;
  private _fnEnableAuditDebug: Function;
  private _fnCheckSandbox: Function;
  private _fnFetch: Function;
  private _fnCreateWorkerPool: Function;
  private _fnWorkerPostMessage: Function;
  private _fnWorkerExecute: Function;
  private _fnWorkerTryRecv: Function;
  private _fnWorkerTerminate: Function;
  private _fnWorkerShutdown: Function;

  /**
   * Create a KossJS instance.
   *
   * @param libPath  - Path to the kossjs shared library. Auto-detected if omitted.
   * @param stable   - If true (default), disables FFI and Worker capabilities.
   * @param caps     - Capability bitmask. Defaults to KOSS_CAP_ALL.
   * @param builtins - Builtin module flags. Defaults to KOSS_BUILTIN_ALL.
   *
   * Examples:
   *   // Production (stable mode, FFI/Worker disabled)
   *   const koss = new KossJS();
   *
   *   // Only Node.js compat layer
   *   const koss = new KossJS(undefined, true, KOSS_CAP_ALL, KOSS_BUILTIN_NODE);
   *
   *   // Sandbox + Deno only
   *   const koss = new KossJS(undefined, true, KOSS_CAP_SANDBOX, KOSS_BUILTIN_DENO);
   */
  constructor(libPath?: string, stable: boolean = true, caps?: number, builtins?: number) {
    const dllPath = libPath || findLibrary();
    this._lib = koffi.load(dllPath);
    const effectiveCaps = caps ?? KOSS_CAP_ALL;
    const effectiveBuiltins = builtins ?? KOSS_BUILTIN_ALL;

    // 创建实例 (with module support)
    const createFn = this._lib.func('koss_create_with_modules_and_builtins', 'void *', ['string', 'uint32', 'uint32', 'bool']);
    this._ptr = createFn('.', effectiveCaps, effectiveBuiltins, stable);
    if (!this._ptr) throw new Error('Failed to create KossJS instance');

    // CRT
    const crtName = process.platform === 'win32' ? 'msvcrt' : 'libc.so.6';
    this._msvcrt = koffi.load(crtName);
    this._strlen = this._msvcrt.func('strlen', 'int32', ['void *']);
    this._memcpy = this._msvcrt.func('memcpy', 'void *', ['void *', 'void *', 'int32']);

    // 实例管理
    this._fnVersion = this._lib.func('koss_version', 'string', []);
    this._fnDestroy = this._lib.func('koss_destroy', 'void', ['void *']);
    this._fnGetCapabilities = this._lib.func('koss_get_capabilities', 'uint32', ['void *']);

    // 代码执行
    this._fnEval = this._lib.func('koss_eval', KossResult, ['void *', 'string']);
    this._fnFreeResult = this._lib.func('koss_free_result', 'void', [KossResult]);
    this._fnRunFile = this._lib.func('koss_run_file', KossResult, ['void *', 'string']);
    this._fnRunModule = this._lib.func('koss_run_module', KossResult, ['void *', 'string']);
    this._fnRunString = this._lib.func('koss_run_string', KossResult, ['void *', 'string']);
    this._fnRunModuleString = this._lib.func('koss_run_module_string', KossResult, ['void *', 'string']);
    this._fnRunAsync = this._lib.func('koss_run_async', KossResult, ['void *', 'string', 'uint64']);
    this._fnTick = this._lib.func('koss_tick', KossResult, ['void *']);

    // 全局变量
    this._fnSetGlobalString = this._lib.func('koss_set_global_string', KossResult, ['void *', 'string', 'string']);
    this._fnSetGlobalNumber = this._lib.func('koss_set_global_number', KossResult, ['void *', 'string', 'double']);
    this._fnSetGlobalBool = this._lib.func('koss_set_global_bool', KossResult, ['void *', 'string', 'bool']);
    this._fnSetGlobalNull = this._lib.func('koss_set_global_null', KossResult, ['void *', 'string']);
    this._fnSetGlobalUndefined = this._lib.func('koss_set_global_undefined', KossResult, ['void *', 'string']);
    this._fnSetGlobalJson = this._lib.func('koss_set_global_json', KossResult, ['void *', 'string', 'string']);

    // 注册
    this._fnRegisterFunction = this._lib.func('koss_register_function', KossResult, ['void *', 'string', 'void *']);
    this._fnRegisterModuleLoader = this._lib.func('koss_register_module_loader', KossResult, ['void *', 'void *']);
    this._fnRegisterClass = this._lib.func('koss_register_class', KossResult, ['void *', 'string', 'string', 'void *']);

    // 绑定
    this._fnGetBinding = this._lib.func('koss_get_binding', KossResult, ['void *', 'string']);

    // 审核与沙箱
    this._fnSetAuditMask = this._lib.func('koss_set_audit_mask', KossResult, ['void *', 'uint32']);
    this._fnGetAuditMask = this._lib.func('koss_get_audit_mask', 'uint32', ['void *']);
    this._fnEnableAuditDebug = this._lib.func('koss_enable_audit_debug', 'void', ['void *', 'bool']);
    this._fnCheckSandbox = this._lib.func('koss_check_sandbox', KossResult, ['void *', 'void *', 'void *']);

    // 网络
    this._fnFetch = this._lib.func('koss_fetch', KossResult, ['void *', 'string']);

    // Worker
    this._fnCreateWorkerPool = this._lib.func('koss_create_worker_pool', KossResult, ['void *', 'int32']);
    this._fnWorkerPostMessage = this._lib.func('koss_worker_post_message', KossResult, ['void *', 'int32', 'string']);
    this._fnWorkerExecute = this._lib.func('koss_worker_execute', KossResult, ['void *', 'int32', 'string']);
    this._fnWorkerTryRecv = this._lib.func('koss_worker_try_recv', KossResult, ['void *']);
    this._fnWorkerTerminate = this._lib.func('koss_worker_terminate', KossResult, ['void *', 'int32']);
    this._fnWorkerShutdown = this._lib.func('koss_worker_shutdown', KossResult, ['void *']);
  }

  /** Check if this instance is running in stable mode. */
  isStable(): boolean {
    const fn = this._lib.func('koss_is_stable', 'bool', ['void *']);
    return fn(this._ptr);
  }

  private _readCString(ptr: any): string {
    if (!ptr) return '';
    const len: number = this._strlen(ptr);
    if (len <= 0) return '';
    const buf = Buffer.alloc(len + 1);
    const addr = koffi.address ? koffi.address(buf) : BigInt(0);
    this._memcpy(addr, ptr, len + 1);
    const end = buf.indexOf(0);
    return buf.toString('utf-8', 0, end >= 0 ? end : len);
  }

  private _checkResult(result: any): string {
    const rawPtr = result.value;
    const value = this._readCString(rawPtr);
    const code: number = result.code;
    this._fnFreeResult(result);
    if (code === RESULT_OK) return value;
    if (code === RESULT_ERROR) throw new JsError(value);
    throw new Error('Invalid argument: ' + value);
  }

  private _ensurePtr(): void {
    if (!this._ptr) throw new Error('KossJS instance has been destroyed');
  }

  // --- 实例管理 ---

  version(): string {
    return this._fnVersion();
  }

  getCapabilities(): number {
    this._ensurePtr();
    return this._fnGetCapabilities(this._ptr);
  }

  /** Get the builtin module flags bitmask. */
  getBuiltins(): number {
    this._ensurePtr();
    const fn = this._lib.func('koss_get_builtins', 'uint32', ['void *']);
    return fn(this._ptr);
  }

  /** Check if a specific builtin flag is enabled. */
  isBuiltinEnabled(flag: number): boolean {
    this._ensurePtr();
    const fn = this._lib.func('koss_is_builtin_enabled', 'bool', ['void *', 'uint32']);
    return fn(this._ptr, flag);
  }

  destroy(): void {
    if (this._ptr) {
      this._fnDestroy(this._ptr);
      this._ptr = null;
    }
  }

  // --- 代码执行 ---

  eval(code: string): any {
    this._ensurePtr();
    const result = this._fnEval(this._ptr, code);
    const value = this._checkResult(result);
    if (value && (value[0] === '{' || value[0] === '[')) {
      try { return JSON.parse(value); } catch {}
    }
    return value;
  }

  runFile(path: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnRunFile(this._ptr, path));
  }

  runModule(path: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnRunModule(this._ptr, path));
  }

  runString(code: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnRunString(this._ptr, code));
  }

  runModuleString(code: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnRunModuleString(this._ptr, code));
  }

  runAsync(code: string, timeoutMs: number = 30000): string {
    this._ensurePtr();
    return this._checkResult(this._fnRunAsync(this._ptr, code, timeoutMs));
  }

  tick(): boolean {
    this._ensurePtr();
    const val = this._checkResult(this._fnTick(this._ptr));
    return val === '1';
  }

  // --- 全局变量 ---

  setGlobalString(name: string, value: string): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalString(this._ptr, name, value));
  }

  setGlobalNumber(name: string, value: number): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalNumber(this._ptr, name, value));
  }

  setGlobalBool(name: string, value: boolean): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalBool(this._ptr, name, value));
  }

  setGlobalNull(name: string): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalNull(this._ptr, name));
  }

  setGlobalUndefined(name: string): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalUndefined(this._ptr, name));
  }

  setGlobalJson(name: string, value: any): void {
    this._ensurePtr();
    this._checkResult(this._fnSetGlobalJson(this._ptr, name, JSON.stringify(value)));
  }

  setGlobal(name: string, value: any): void {
    if (value === null) {
      this.setGlobalNull(name);
    } else if (value === undefined) {
      this.setGlobalUndefined(name);
    } else if (typeof value === 'boolean') {
      this.setGlobalBool(name, value);
    } else if (typeof value === 'string') {
      this.setGlobalString(name, value);
    } else if (typeof value === 'number') {
      this.setGlobalNumber(name, value);
    } else {
      this.setGlobalJson(name, value);
    }
  }

  // --- 函数注册 ---

  registerFunction(name: string, func: (...args: string[]) => string | null | undefined): void {
    this._ensurePtr();
    const libc = this._msvcrt;
    const mallocFn = libc.func('malloc', 'void *', ['uint32']);

    const callbackType = koffi.proto('void * FuncCallback(int32_t argc, void *argv)');
    const wrapped = koffi.register(callbackType, koffi.FREE, (argc: number, argv: number) => {
      try {
        const args: string[] = [];
        for (let i = 0; i < argc; i++) {
          const strPtr = koffi.decode(argv + i * 4, 'void *');
          if (!strPtr) {
            args.push('');
          } else {
            args.push(this._readCString(strPtr));
          }
        }
        const result = func(...args);
        if (result === null || result === undefined) return 0;
        const encoded = Buffer.from(result, 'utf-8');
        const buf = mallocFn(encoded.length + 1);
        if (!buf) return 0;
        koffi.encode(buf, 'char', Array.from(encoded));
        koffi.encode(buf + encoded.length, 'char', [0]);
        return buf;
      } catch (e) {
        console.error('Callback error:', e);
        return 0;
      }
    });

    this._checkResult(this._fnRegisterFunction(this._ptr, name, wrapped));
    this._callbacks.push(wrapped);
  }

  // --- 模块加载器 ---

  registerModuleLoader(loaderFunc: (modulePath: string) => { type: string; code: string } | null): void {
    this._ensurePtr();
    const libc = this._msvcrt;
    const mallocFn = libc.func('malloc', 'void *', ['uint32']);

    const callbackType = koffi.proto('void * ModuleLoaderCallback(int32_t argc, void *argv)');
    const wrapped = koffi.register(callbackType, koffi.FREE, (argc: number, argv: number) => {
      try {
        if (argc < 1) return 0;
        const strPtr = koffi.decode(argv, 'void *');
        if (!strPtr) return 0;
        const modulePath = this._readCString(strPtr);

        const result = loaderFunc(modulePath);
        if (!result) return 0;

        const resultJson = JSON.stringify(result);
        const encoded = Buffer.from(resultJson, 'utf-8');
        const buf = mallocFn(encoded.length + 1);
        if (!buf) return 0;
        koffi.encode(buf, 'char', Array.from(encoded));
        koffi.encode(buf + encoded.length, 'char', [0]);
        return buf;
      } catch (e) {
        console.error('Module loader error:', e);
        return 0;
      }
    });

    this._checkResult(this._fnRegisterModuleLoader(this._ptr, wrapped));
    this._moduleLoaderCallbacks.push(wrapped);
  }

  // --- 类注册 ---

  registerClass(className: string, methods: Record<string, (...args: string[]) => string | null | undefined>): void {
    this._ensurePtr();
    const libc = this._msvcrt;
    const mallocFn = libc.func('malloc', 'void *', ['uint32']);

    const methodNames = Object.keys(methods);

    const callbackType = koffi.proto('void * ClassCallback(int32_t argc, void *argv)');
    const wrapped = koffi.register(callbackType, koffi.FREE, (argc: number, argv: number) => {
      try {
        const args: string[] = [];
        for (let i = 0; i < argc; i++) {
          const strPtr = koffi.decode(argv + i * 4, 'void *');
          if (!strPtr) {
            args.push('');
          } else {
            args.push(this._readCString(strPtr));
          }
        }
        if (!args.length) return 0;
        const methodName = args[0];
        const methodArgs = args.slice(1);
        if (methodName in methods) {
          const result = methods[methodName](...methodArgs);
          if (result === null || result === undefined) return 0;
          const encoded = Buffer.from(String(result), 'utf-8');
          const buf = mallocFn(encoded.length + 1);
          if (!buf) return 0;
          koffi.encode(buf, 'char', Array.from(encoded));
          koffi.encode(buf + encoded.length, 'char', [0]);
          return buf;
        }
        return 0;
      } catch (e) {
        console.error(`Class callback error (${className}):`, e);
        return 0;
      }
    });

    this._checkResult(this._fnRegisterClass(this._ptr, className, JSON.stringify(methodNames), wrapped));
    this._classCallbacks.push(wrapped);
  }

  // --- 绑定 ---

  getBinding(name: string): any {
    this._ensurePtr();
    const value = this._checkResult(this._fnGetBinding(this._ptr, name));
    return value ? JSON.parse(value) : {};
  }

  // --- 审核与沙箱 ---

  setAuditMask(mask: number): void {
    this._ensurePtr();
    this._checkResult(this._fnSetAuditMask(this._ptr, mask));
  }

  getAuditMask(): number {
    this._ensurePtr();
    return this._fnGetAuditMask(this._ptr);
  }

  enableAuditDebug(enable: boolean): void {
    this._ensurePtr();
    this._fnEnableAuditDebug(this._ptr, enable);
  }

  checkSandbox(func: ((target: string, args: string[], pwd: string | null) => boolean) | null): void {
    this._ensurePtr();

    if (!func) {
      const cbType = koffi.proto('bool AuditCallback(const char *target, void *args, int32_t argc, const char *pwd, void *userdata)');
      const nullCb = cbType(0);
      this._checkResult(this._fnCheckSandbox(this._ptr, nullCb, null));
      this._auditCallback = null;
      return;
    }

    const cbType = koffi.proto('bool AuditCallback(const char *target, void *args, int32_t argc, const char *pwd, void *userdata)');
    const wrapped = koffi.register(cbType, koffi.FREE, (target: any, argsPtr: any, argc: number, pwd: any, _userdata: any): boolean => {
      const targetStr = target ? target.toString('utf-8') : '';
      const args: string[] = [];
      for (let i = 0; i < argc; i++) {
        const strPtr = koffi.decode(argsPtr + i * 4, 'void *');
        if (!strPtr) {
          args.push('');
        } else {
          args.push(this._readCString(strPtr));
        }
      }
      const pwdStr = pwd ? pwd.toString('utf-8') : null;
      try {
        return func(targetStr, args, pwdStr);
      } catch {
        return false;
      }
    });

    this._auditCallback = wrapped;
    this._checkResult(this._fnCheckSandbox(this._ptr, wrapped, null));
  }

  // --- 网络 ---

  fetch(url: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnFetch(this._ptr, url));
  }

  // --- Worker ---

  createWorkerPool(size: number): string {
    this._ensurePtr();
    return this._checkResult(this._fnCreateWorkerPool(this._ptr, size));
  }

  workerPostMessage(workerId: number, data: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnWorkerPostMessage(this._ptr, workerId, data));
  }

  workerExecute(workerId: number, code: string): string {
    this._ensurePtr();
    return this._checkResult(this._fnWorkerExecute(this._ptr, workerId, code));
  }

  workerTryRecv(): string | null {
    this._ensurePtr();
    try {
      const val = this._checkResult(this._fnWorkerTryRecv(this._ptr));
      if (val === 'null' || !val) return null;
      return val;
    } catch {
      return null;
    }
  }

  workerTerminate(workerId: number): string {
    this._ensurePtr();
    return this._checkResult(this._fnWorkerTerminate(this._ptr, workerId));
  }

  workerShutdown(): string {
    this._ensurePtr();
    return this._checkResult(this._fnWorkerShutdown(this._ptr));
  }
}
