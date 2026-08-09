// Copyright (C) 2026 TT23XR Studio with AGPL v3.0 license

"use strict";

var Buffer;
try {
  Buffer = require('koss:buffer').Buffer;
} catch (e) {
  Buffer = globalThis.Buffer;
}
var TextEncoder = globalThis.TextEncoder;
var TextDecoder = globalThis.TextDecoder;

const kNoOp = () => {};
const kSymbol = Symbol;
const kObject = Object;
const kArray = Array;
const kString = String;
const kNumber = Number;
const kRegExp = RegExp;
const kBoolean = Boolean;
const kUndefined = undefined;
const kNull = null;
const kTypeError = TypeError;

function inherits(ctor, superCtor) {
  if (superCtor) {
    ctor.super_ = superCtor;
    ctor.prototype = kObject.create(superCtor.prototype, {
      constructor: {
        value: ctor,
        enumerable: false,
        writable: true,
        configurable: true,
      },
    });
  }
}

function inheritsDeep(target, source) {
  inherits(target, source);
  const descriptors = kObject.getOwnPropertyDescriptors(source);
  for (const key of kObject.keys(descriptors)) {
    if (key === "prototype" || key === "constructor") continue;
    const desc = descriptors[key];
    if (desc.value && typeof desc.value === "function") {
      if (target.prototype && !target.prototype[key]) {
        target.prototype[key] = desc.value;
      }
    }
    kObject.defineProperty(target, key, desc);
  }
}

function inheritsPrototype(ctor, superCtor) {
  if (!superCtor || !superCtor.prototype) return;
  const protoProps = kObject.getOwnPropertyNames(superCtor.prototype);
  for (const prop of protoProps) {
    if (prop === "constructor") continue;
    const desc = kObject.getOwnPropertyDescriptor(superCtor.prototype, prop);
    if (!kObject.getOwnPropertyDescriptor(ctor.prototype, prop)) {
      kObject.defineProperty(ctor.prototype, prop, desc);
    }
  }
}

function isBoolean(value) {
  return typeof value === "boolean" || value instanceof kBoolean;
}

function isBuffer(value) {
  return Buffer.isBuffer(value);
}

function isDate(value) {
  return value instanceof Date;
}

function isDeepStrictEqual(a, b) {
  return require("assert").deepStrictEqual(a, b), true;
}

function isEmptyBuffer(value) {
  return isBuffer(value) && value.length === 0;
}

function isFunction(value) {
  return typeof value === "function";
}

function isNull(value) {
  return value === kNull;
}

function isNumber(value) {
  return typeof value === "number" || value instanceof kNumber;
}

function isObject(value) {
  return typeof value === "object" && value !== kNull && !isBuffer(value) && !isDate(value);
}

function isPrimitive(value) {
  const t = typeof value;
  return t === "string" || t === "number" || t === "boolean" || t === "symbol" || value === kNull || value === kUndefined;
}

function isRegExp(value) {
  return value instanceof kRegExp;
}

function isString(value) {
  return typeof value === "string" || value instanceof kString;
}

function isSymbol(value) {
  return typeof value === "symbol" || value instanceof kSymbol;
}

function isUndefined(value) {
  return value === kUndefined;
}

function _formatValue(value) {
  if (value === kNull) return 'null';
  if (value === kUndefined) return 'undefined';
  if (typeof value === 'string') return value;
  if (typeof value === 'function') return inspect(value);
  if (typeof value === 'object' && value !== kNull) return inspect(value);
  return kString(value);
}

function format(f) {
  const args = kArray.from(arguments);
  if (typeof f !== 'string') {
    const parts = [];
    for (let i = 0; i < args.length; i++) parts.push(_formatValue(args[i]));
    return parts.join(' ');
  }
  const rest = args.slice(1);
  let argIndex = 0;
  let out = '';
  let i = 0;
  while (i < f.length) {
    const ch = f[i];
    if (ch !== '%') {
      out += ch;
      i++;
      continue;
    }
    const spec = f[++i];
    if (spec === '%') {
      out += '%';
      i++;
      continue;
    }
    if (spec === 's') {
      out += argIndex < rest.length ? kString(rest[argIndex++]) : '%s';
      i++;
      continue;
    }
    if (spec === 'd' || spec === 'i') {
      const v = argIndex < rest.length ? rest[argIndex++] : NaN;
      out += kNumber(v).toString();
      i++;
      continue;
    }
    if (spec === 'f') {
      const vf = argIndex < rest.length ? rest[argIndex++] : NaN;
      out += parseFloat(vf).toString();
      i++;
      continue;
    }
    if (spec === 'j') {
      const vj = argIndex < rest.length ? rest[argIndex++] : kUndefined;
      try {
        out += JSON.stringify(vj);
      } catch (_) {
        out += '[Circular]';
      }
      i++;
      continue;
    }
    if (spec === 'o' || spec === 'O') {
      const vo = argIndex < rest.length ? rest[argIndex++] : kUndefined;
      out += inspect(vo);
      i++;
      continue;
    }
    out += '%' + spec;
    i++;
  }
  for (let j = argIndex; j < rest.length; j++) {
    out += ' ' + _formatValue(rest[j]);
  }
  return out;
}

function formatWithOptions(options, ...args) {
  return format(...args);
}

function _inspectValue(value, depth) {
  if (value === kNull) return 'null';
  if (value === kUndefined) return 'undefined';
  const t = typeof value;
  if (t === 'string') return "'" + value + "'";
  if (t === 'number') return kString(value);
  if (t === 'boolean') return kString(value);
  if (t === 'symbol') return kString(value);
  if (t === 'function') {
    const name = value.name || value.displayName || 'anonymous';
    return '[Function: ' + name + ']';
  }
  if (t === 'object') {
    if (value instanceof kRegExp) return kString(value);
    if (value instanceof Date) {
      return typeof value.toISOString === 'function' ? value.toISOString() : kString(value);
    }
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer && Buffer.isBuffer(value)) {
      let hex = '';
      for (let i = 0; i < value.length && i < 8; i++) {
        hex += (value[i] < 16 ? '0' : '') + value[i].toString(16);
      }
      return '<Buffer ' + hex + '>'; 
    }
    if (kArray.isArray(value)) {
      if (value.length === 0) return '[]';
      const items = [];
      for (let i = 0; i < value.length; i++) items.push(_inspectValue(value[i], depth + 1));
      return '[' + items.join(', ') + ']';
    }
    const keys = kObject.keys(value);
    if (keys.length === 0) return '{}';
    const entries = [];
    for (let j = 0; j < keys.length; j++) {
      entries.push(keys[j] + ': ' + _inspectValue(value[keys[j]], depth + 1));
    }
    return '{ ' + entries.join(', ') + ' }';
  }
  return kString(value);
}

function inspect(value, options, depth) {
  return _inspectValue(value, 0);
}

function debug(namespace) {
  return function debug() {
    if (typeof console !== 'undefined' && console.error) {
      console.error('DEBUG:' + namespace, kArray.from(arguments).join(' '));
    }
  };
}

function debuglog(set) {
  return function debuglog() {
    if (typeof console !== 'undefined' && console.error) {
      console.error('DEBUG:' + set, kArray.from(arguments).join(' '));
    }
  };
}

function deprecate(fn, msg, code) {
  if (typeof fn !== 'function') {
    throw new kTypeError('The "fn" argument must be of type Function');
  }
  let warned = false;
  const deprecated = function () {
    if (!warned) {
      warned = true;
      if (typeof console !== 'undefined' && console.error) {
        console.error('DeprecationWarning: ' + (msg || ''));
      }
    }
    return fn.apply(this, arguments);
  };
  deprecated.deprecation = true;
  return deprecated;
}

function log(...args) {
  console.log(...args);
}

function error(...args) {
  console.error(...args);
}

function promisify(original) {
  if (typeof original !== "function") {
    throw new kTypeError("The \"original\" argument must be of type Function");
  }
  if (original[kkPromisifySymbol]) {
    return original[kkPromisifySymbol];
  }
  const fn = function (...args) {
    return new Promise((resolve, reject) => {
      original.call(this, ...args, (err, ...values) => {
        if (err) {
          reject(err);
        } else {
          resolve(values.length <= 1 ? values[0] : values);
        }
      });
    });
  };
  Object.setPrototypeOf(fn, kObject.getPrototypeOf(original));
  try {
    const names = kObject.getOwnPropertyNames(original);
    for (const name of names) {
      if (!kObject.getOwnPropertyDescriptor(fn, name)) {
        const desc = kObject.getOwnPropertyDescriptor(original, name);
        if (typeof desc.value === "function") {
          kObject.defineProperty(fn, name, desc);
        }
      }
    }
  } catch (_) {}
  fn[kkPromisifySymbol] = fn;
  Object.defineProperty(fn, "name", {
    value: `promisify(${original.name || original.displayName || ""})`,
    configurable: true,
  });
  return fn;
}

const kkPromisifySymbol = kSymbol("nodejs.util.promisify.custom");

function callbackify(fn) {
  if (typeof fn !== "function") {
    throw new kTypeError("The \"fn\" argument must be of type Function");
  }
  if (fn[kkCallbackifySymbol]) {
    return fn[kkCallbackifySymbol];
  }
  const cbFn = function (...args) {
    const lastArg = args[args.length - 1];
    const cb = typeof lastArg === "function" ? lastArg : kNoOp;
    if (typeof lastArg === "function") {
      args.pop();
    }
    const promise = fn.apply(this, args);
    if (promise && typeof promise.then === "function") {
      promise.then((value) => cb(kNull, value), (err) => cb(err));
    }
  };
  Object.setPrototypeOf(cbFn, kObject.getPrototypeOf(fn));
  try {
    const names = kObject.getOwnPropertyNames(fn);
    for (const name of names) {
      if (!kObject.getOwnPropertyDescriptor(cbFn, name)) {
        const desc = kObject.getOwnPropertyDescriptor(fn, name);
        if (typeof desc.value === "function") {
          kObject.defineProperty(cbFn, name, desc);
        }
      }
    }
  } catch (_) {}
  cbFn[kkCallbackifySymbol] = cbFn;
  Object.defineProperty(cbFn, "name", {
    value: `callbackify(${fn.name || fn.displayName || ""})`,
    configurable: true,
  });
  return cbFn;
}

function stripVTControlCharacters(str) {
  if (typeof str !== "string") {
    throw new kTypeError("The \"str\" argument must be of type string");
  }
  return str.replace(
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=<>]/g,
    ""
  );
}

function getCallSites(options) {
  const originalLimit = Error.stackTraceLimit;
  Error.stackTraceLimit = (options && options.limit) || 10;
  const err = new Error();
  Error.stackTraceLimit = originalLimit;
  const sites = [];
  const lines = (err.stack || "").split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (i === 0 || line.includes("node:internal")) continue;
    sites.push({ stack: line });
  }
  return sites;
}

class AbortSignal {
  constructor() {
    this.aborted = false;
    this.onabort = null;
    this[kkAbortListeners] = [];
  }
  addEventListener(type, listener) {
    if (type === "abort") {
      this[kkAbortListeners].push(listener);
    }
  }
  removeEventListener(type, listener) {
    if (type === "abort") {
      const idx = this[kkAbortListeners].indexOf(listener);
      if (idx !== -1) this[kkAbortListeners].splice(idx, 1);
    }
  }
  dispatchEvent(event) {
    if (event.type === "abort") {
      this.aborted = true;
      if (typeof this.onabort === "function") {
        this.onabort(event);
      }
      for (const fn of this[kkAbortListeners]) {
        fn(event);
      }
    }
    return true;
  }
  throwIfAborted() {
    if (this.aborted) {
      throw new DOMException("The operation was aborted", "AbortError");
    }
  }
}

const kkAbortListeners = kSymbol("abortListeners");

class AbortController {
  constructor() {
    this.signal = new AbortSignal();
  }
  abort(reason) {
    if (this.signal.aborted) return;
    const event = { type: "abort", reason };
    this.signal.dispatchEvent(event);
  }
  toString() {
    return "[object AbortController]";
  }
}

class DOMException extends Error {
  constructor(message = "", name = "Error") {
    super(message);
    this.name = name;
  }
}

const querystring = {
  parse(str, sep = "&", eq = "=", options) {
    const obj = {};
    if (typeof str !== "string") return obj;
    const decode = options && options.decodeURIComponent ? options.decodeURIComponent : decodeURIComponent;
    const parts = str.split(sep);
    for (const part of parts) {
      if (!part) continue;
      const idx = part.indexOf(eq);
      let key, value;
      if (idx === -1) {
        key = decode(part);
        value = "";
      } else {
        key = decode(part.substring(0, idx));
        value = decode(part.substring(idx + 1));
      }
      if (obj[key] === kUndefined) {
        obj[key] = value;
      } else if (kArray.isArray(obj[key])) {
        obj[key].push(value);
      } else {
        obj[key] = [obj[key], value];
      }
    }
    return obj;
  },
  stringify(obj, sep = "&", eq = "=", options) {
    if (typeof obj !== "object" || obj === kNull) return "";
    const encode = options && options.encodeURIComponent ? options.encodeURIComponent : encodeURIComponent;
    const parts = [];
    const keys = kObject.keys(obj);
    for (const key of keys) {
      const val = obj[key];
      if (kArray.isArray(val)) {
        for (const item of val) {
          parts.push(encode(key) + eq + encode(item));
        }
      } else {
        parts.push(encode(key) + eq + encode(val));
      }
    }
    return parts.join(sep);
  },
  escape(str) {
    return encodeURIComponent(str)
      .replace(/[!'()*]/g, (c) => "%" + c.charCodeAt(0).toString(16).toUpperCase())
      .replace(/%20/g, "+");
  },
  unescape(str) {
    return decodeURIComponent(str.replace(/\+/g, " "));
  },
  encode(obj, sep, eq, options) {
    return querystring.stringify(obj, sep, eq, options);
  },
  decode(str, sep, eq, options) {
    return querystring.parse(str, sep, eq, options);
  },
};

const types = {
  isArray: kArray.isArray,
  isArrayBuffer: function (v) { return v instanceof ArrayBuffer; },
  isArrayBufferView: function (v) { return ArrayBuffer.isView(v); },
  isBigInt64Array: function (v) { return Object.prototype.toString.call(v) === '[object BigInt64Array]'; },
  isBigUint64Array: function (v) { return Object.prototype.toString.call(v) === '[object BigUint64Array]'; },
  isBoolean: function (v) { return typeof v === 'boolean'; },
  isDataView: function (v) { return Object.prototype.toString.call(v) === '[object DataView]'; },
  isDate: function (v) { return v instanceof Date; },
  isFloat32Array: function (v) { return Object.prototype.toString.call(v) === '[object Float32Array]'; },
  isFloat64Array: function (v) { return Object.prototype.toString.call(v) === '[object Float64Array]'; },
  isFunction: function (v) { return typeof v === 'function'; },
  isGeneratorFunction: function (v) { return v && Object.prototype.toString.call(v) === '[object GeneratorFunction]'; },
  isInt8Array: function (v) { return Object.prototype.toString.call(v) === '[object Int8Array]'; },
  isInt16Array: function (v) { return Object.prototype.toString.call(v) === '[object Int16Array]'; },
  isInt32Array: function (v) { return Object.prototype.toString.call(v) === '[object Int32Array]'; },
  isMap: function (v) { return Object.prototype.toString.call(v) === '[object Map]'; },
  isNativeError: function (v) { return v instanceof Error; },
  isNull: function (v) { return v === kNull; },
  isNumber: function (v) { return typeof v === 'number'; },
  isObject: function (v) { return typeof v === 'object' && v !== kNull; },
  isPromise: function (v) { return v instanceof Promise; },
  isRegExp: function (v) { return v instanceof kRegExp; },
  isSet: function (v) { return Object.prototype.toString.call(v) === '[object Set]'; },
  isSharedArrayBuffer: function (v) { return v instanceof SharedArrayBuffer; },
  isString: function (v) { return typeof v === 'string'; },
  isSymbol: function (v) { return typeof v === 'symbol'; },
  isUint8Array: function (v) { return Object.prototype.toString.call(v) === '[object Uint8Array]'; },
  isUint8ClampedArray: function (v) { return Object.prototype.toString.call(v) === '[object Uint8ClampedArray]'; },
  isUint16Array: function (v) { return Object.prototype.toString.call(v) === '[object Uint16Array]'; },
  isUint32Array: function (v) { return Object.prototype.toString.call(v) === '[object Uint32Array]'; },
  isUndefined: function (v) { return v === kUndefined; },
  isWeakMap: function (v) { return Object.prototype.toString.call(v) === '[object WeakMap]'; },
  isWeakSet: function (v) { return Object.prototype.toString.call(v) === '[object WeakSet]'; },
};

const _systemErrorNames = {
  1: 'EPERM', 2: 'ENOENT', 3: 'ESRCH', 4: 'EINTR', 5: 'EIO', 6: 'ENXIO', 7: 'E2BIG',
  8: 'ENOEXEC', 9: 'EBADF', 10: 'ECHILD', 11: 'EAGAIN', 12: 'ENOMEM', 13: 'EACCES',
  14: 'EFAULT', 15: 'ENOTBLK', 16: 'EBUSY', 17: 'EEXIST', 18: 'EXDEV', 19: 'ENODEV',
  20: 'ENOTDIR', 21: 'EISDIR', 22: 'EINVAL', 23: 'ENFILE', 24: 'EMFILE', 25: 'ENOTTY',
  26: 'ETXTBSY', 27: 'EFBIG', 28: 'ENOSPC', 29: 'ESPIPE', 30: 'EROFS', 31: 'EMLINK',
  32: 'EPIPE', 33: 'EDOM', 34: 'ERANGE', 35: 'EDEADLK', 36: 'ENAMETOOLONG',
  37: 'ENOLCK', 38: 'ENOSYS', 39: 'ENOTEMPTY', 40: 'ELOOP', 41: 'ENOMSG',
  42: 'EIDRM', 43: 'ECHRNG', 44: 'EL2NSYNC', 45: 'EL3HLT', 46: 'EL3RST',
  47: 'ELNRNG', 48: 'EUNATCH', 49: 'ENOCSI', 50: 'EL2HLT', 51: 'EBADE',
  52: 'EBADR', 53: 'EXFULL', 54: 'ENOANO', 55: 'EBADRQC', 56: 'EBADSLT',
  57: 'EDEADLOCK', 59: 'EBFONT', 60: 'ENOSTR', 61: 'ENODATA', 62: 'ETIME',
  63: 'ENOSR', 64: 'ENONET', 65: 'ENOPKG', 66: 'EREMOTE', 67: 'ENOLINK',
  68: 'EADV', 69: 'ESRMNT', 70: 'ECOMM', 71: 'EPROTO', 72: 'EMULTIHOP',
  73: 'EDOTDOT', 74: 'EBADMSG', 75: 'EOVERFLOW', 76: 'ENOTUNIQ', 77: 'EBADFD',
  78: 'EREMCHG', 79: 'ELIBACC', 80: 'ELIBBAD', 81: 'ELIBSCN', 82: 'ELIBMAX',
  83: 'ELIBEXEC', 84: 'EILSEQ', 85: 'ERESTART', 86: 'ESTRPIPE', 87: 'EUSERS',
  88: 'ENOTSOCK', 89: 'EDESTADDRREQ', 90: 'EMSGSIZE', 91: 'EPROTOTYPE',
  92: 'ENOPROTOOPT', 93: 'EPROTONOSUPPORT', 94: 'ESOCKTNOSUPPORT',
  95: 'EOPNOTSUPP', 96: 'EPFNOSUPPORT', 97: 'EAFNOSUPPORT', 98: 'EADDRINUSE',
  99: 'EADDRNOTAVAIL', 100: 'ENETDOWN', 101: 'ENETUNREACH', 102: 'ENETRESET',
  103: 'ECONNABORTED', 104: 'ECONNRESET', 105: 'ENOBUFS', 106: 'EISCONN',
  107: 'ENOTCONN', 108: 'ESHUTDOWN', 109: 'ETOOMANYREFS', 110: 'ETIMEDOUT',
  111: 'ECONNREFUSED', 112: 'EHOSTDOWN', 113: 'EHOSTUNREACH', 114: 'EALREADY',
  115: 'EINPROGRESS', 116: 'ESTALE', 117: 'EUCLEAN', 118: 'ENOTNAM', 119: 'ENAVAIL',
  120: 'EISNAM', 121: 'EREMOTEIO', 122: 'EDQUOT', 123: 'ENOMEDIUM',
  124: 'EMEDIUMTYPE', 125: 'ECANCELED', 126: 'ENOKEY', 127: 'EKEYEXPIRED',
  128: 'EKEYREVOKED', 129: 'EKEYREJECTED', 130: 'EOWNERDEAD', 131: 'ENOTRECOVERABLE',
  132: 'ERFKILL', 133: 'EHWPOISON',
};

function getSystemErrorName(errno) {
  return _systemErrorNames[errno] || 'Unknown';
}

module.exports = {
  inherits,
  inheritsDeep,
  inheritsPrototype,
  TextEncoder,
  TextDecoder,
  format,
  formatWithOptions,
  inspect,
  debug,
  debuglog,
  deprecate,
  error,
  isBoolean,
  isBuffer,
  isDate,
  isDeepStrictEqual,
  isEmptyBuffer,
  isFunction,
  isNull,
  isNumber,
  isObject,
  isPrimitive,
  isRegExp,
  isString,
  isSymbol,
  isUndefined,
  log,
  promisify,
  callbackify,
  stripVTControlCharacters,
  getCallSites,
  AbortController,
  AbortSignal,
  querystring,
  types,
  getSystemErrorName,
};
