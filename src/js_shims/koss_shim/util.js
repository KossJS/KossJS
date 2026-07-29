// Copyright (C) 2026 TT23XR Studio with AGPL v3.0 license

"use strict";

const { Buffer } = require("buffer");
const { format: fmt, inspect, debuglog: dl, deprecate } = require("util");
const { TextEncoder, TextDecoder } = require("text-encoding");

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

function format(...args) {
  return fmt(...args);
}

function formatWithOptions(options, ...args) {
  return fmt(...args);
}

function debug(namespace) {
  return dl(namespace);
}

function debuglog(set) {
  return dl(set);
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
};
