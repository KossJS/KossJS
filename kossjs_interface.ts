/**
 * KossJS TypeScript Interface — Embeddable JavaScript runtime
 * Mirrors kossjs_interface.py using koffi for C ABI bindings.
 *
 * Run: npx tsx src/__tests__/kossjs-ffi.test.ts
 */

declare const __dirname: string;
declare const process: any;
declare const Buffer: any;
declare const require: any;

import { existsSync } from 'fs';
import { join, dirname } from 'path';

const koffi = require('koffi');

const KossResult = koffi.struct({ code: 'int32', value: 'void *' });

const RESULT_OK = 0;
const RESULT_ERROR = 1;
const KOSS_CAP_ALL = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4);

function findLibrary(): string {
  const candidates = [
    join(__dirname, 'kossjs.dll'),
    join(process.cwd(), 'kossjs.dll'),
    join(process.cwd(), 'src', '__tests__', 'kossjs.dll'),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  throw new Error('kossjs.dll not found');
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
  static readonly KOSS_CAP_ALL = KOSS_CAP_ALL;

  private _lib: any;
  private _ptr: any;
  private _eval: Function;
  private _freeString: Function;
  private _destroy: Function;
  private _version: Function;

  constructor(libPath?: string) {
    const dllPath = libPath || findLibrary();
    this._lib = koffi.load(dllPath);

    this._eval = this._lib.func('koss_eval', KossResult, ['void *', 'string']);
    this._freeString = this._lib.func('koss_free_string', 'void', ['void *']);
    this._destroy = this._lib.func('koss_destroy', 'void', ['void *']);
    this._version = this._lib.func('koss_version', 'string', []);

    const createFn = this._lib.func('koss_create_with_modules_and_caps', 'void *', ['string', 'uint32']);
    this._ptr = createFn('.', KOSS_CAP_ALL);
    if (!this._ptr) throw new Error('Failed to create KossJS instance');
  }

  private _checkResult(result: any): string {
    const code: number = result.code;
    const valuePtr: number | null = result.value;
    const value: string = valuePtr ? koffi.decode('string', valuePtr) : '';
    if (valuePtr) {
      this._freeString(valuePtr);
    }
    if (code === RESULT_OK) return value;
    if (code === RESULT_ERROR) throw new JsError(value);
    throw new Error('Invalid argument: ' + value);
  }

  eval(code: string): any {
    const result = this._eval(this._ptr, code);
    const value = this._checkResult(result);
    if (value && (value[0] === '{' || value[0] === '[')) {
      try { return JSON.parse(value); } catch {}
    }
    return value;
  }

  version(): string {
    return this._version();
  }

  destroy(): void {
    if (this._ptr) {
      this._destroy(this._ptr);
      this._ptr = null;
    }
  }
}
