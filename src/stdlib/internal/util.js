'use strict';

const {
  ObjectDefineProperties,
  ObjectDefineProperty,
  ObjectFreeze,
  ObjectKeys,
  Symbol,
  Map,
  Promise,
  Function,
} = globalThis;

const customPromisify = Symbol('customPromisify');
const customInspectSymbol = Symbol('inspect');
const kIsEncodingSymbol = Symbol('kIsEncodingSymbol');
const kEmptyObject = Object.freeze({});
const encodingsMap = {
  'utf8': 1,
  'utf-8': 1,
  'ascii': 0,
  'latin1': 0,
  'binary': 0,
  'base64': 3,
  'hex': 2,
  'utf16le': 4,
  'utf-16le': 4,
  'ucs2': 4,
  'ucs-2': 4,
  'base64url': 5,
};

function normalizeEncoding(encoding) {
  if (!encoding) return undefined;
  const lower = encoding.toLowerCase();
  const map = {
    'utf8': 'utf8',
    'utf-8': 'utf8',
    'ascii': 'ascii',
    'latin1': 'latin1',
    'binary': 'latin1',
    'base64': 'base64',
    'hex': 'hex',
    'utf16le': 'utf16le',
    'utf-16le': 'utf16le',
    'ucs2': 'utf16le',
    'ucs-2': 'utf16le',
    'base64url': 'base64url',
  };
  return map[lower];
}

function isArrayBufferView(obj) {
  return obj && obj.buffer && obj.byteLength !== undefined;
}

function isUint8Array(obj) {
  return obj instanceof Uint8Array;
}

function isAnyArrayBuffer(obj) {
  return obj && (obj instanceof ArrayBuffer || obj instanceof SharedArrayBuffer);
}

function isTypedArray(obj) {
  return obj && obj.buffer && obj.constructor && obj.constructor.name === 'Uint8Array';
}

function isPromise(obj) {
  return obj instanceof Promise;
}

function isNativeError(obj) {
  return obj instanceof Error;
}

function promisify(original) {
  if (typeof original !== 'function') {
    throw new TypeError('The "original" argument must be of type Function');
  }

  const promisified = function(...args) {
    return new Promise((resolve, reject) => {
      original.call(this, ...args, (err, ...values) => {
        if (err) {
          reject(err);
        } else {
          resolve(values.length === 1 ? values[0] : values);
        }
      });
    });
  };

  ObjectDefineProperties(promisified, {
    __proto__: null,
    [customPromisify]: {
      value: original,
      configurable: true,
      writable: true,
    },
    length: {
      value: Math.max(original.length - 1, 0),
    },
    name: {
      value: `${original.name}__promisified__`,
    },
  });

  return promisified;
}

function callbackify(original) {
  if (typeof original !== 'function') {
    throw new TypeError('The "original" argument must be of type Function');
  }

  const callbackified = function(...args) {
    const promise = original.apply(this, args);
    if (!isPromise(promise)) {
      return Promise.resolve(promise).then(
        (value) => {
          return new Promise((resolve) => {
            args.pop()(null, value);
            resolve();
          });
        },
        (err) => {
          return new Promise((resolve, reject) => {
            args.pop()(err);
            reject(err);
          });
        }
      );
    }
    return promise.then(
      (value) => {
        return new Promise((resolve) => {
          const callback = args.pop();
          if (typeof callback === 'function') {
            callback(null, value);
          }
          resolve();
        });
      },
      (err) => {
        return new Promise((resolve, reject) => {
          const callback = args.pop();
          if (typeof callback === 'function') {
            callback(err);
          }
          reject(err);
        });
      }
    );
  };

  ObjectDefineProperties(callbackified, {
    __proto__: null,
    length: {
      value: Math.max(original.length, 0),
    },
    name: {
      value: `${original.name}__callbackified__`,
    },
  });

  return callbackified;
}

function getOptions(options, defaults = {}) {
  if (options === null || options === undefined) {
    return defaults;
  }
  if (typeof options === 'string') {
    return { ...defaults, encoding: options };
  }
  if (typeof options === 'object') {
    return { ...defaults, ...options };
  }
  return defaults;
}

function getValidatedPath(path, name = 'path') {
  if (path === null || path === undefined) {
    return path;
  }
  return path;
}

function getValidatedFd(fd, name = 'fd') {
  if (typeof fd !== 'number') {
    throw new TypeError(`The '${name}' argument must be of type number`);
  }
  return fd;
}

function handleErrorFromBinding(ctx) {}

function stringToFlags(flags) {
  if (typeof flags === 'number') {
    return flags;
  }
  
  const flagMap = {
    'r': 0,
    'r+': 2,
    'w': 512,
    'w+': 578,
    'a': 1024,
    'a+': 1536,
  };
  
  return flagMap[flags] || 0;
}

function parseFileMode(mode, name, defaultMode = 0o666) {
  if (mode === undefined) {
    return defaultMode;
  }
  if (typeof mode === 'number') {
    return mode;
  }
  if (typeof mode === 'string') {
    return parseInt(mode, 8);
  }
  return defaultMode;
}

function copyObject(obj) {
  return { ...obj };
}

function defineLazyProperties(obj, module, properties) {
  properties.forEach((prop) => {
    let value;
    ObjectDefineProperty(obj, prop, {
      __proto__: null,
      enumerable: true,
      configurable: true,
      get() {
        if (!value) {
          value = require(module)[prop];
        }
        return value;
      },
    });
  });
}

function uncurryThis(fn) {
  return function(...args) {
    return fn.call(this, ...args);
  };
}

module.exports = {
  customPromisify,
  customInspectSymbol,
  kIsEncodingSymbol,
  kEmptyObject,
  normalizeEncoding,
  isArrayBufferView,
  isUint8Array,
  isAnyArrayBuffer,
  isTypedArray,
  isPromise,
  isNativeError,
  promisify,
  callbackify,
  getOptions,
  getValidatedPath,
  getValidatedFd,
  handleErrorFromBinding,
  stringToFlags,
  parseFileMode,
  copyObject,
  defineLazyProperties,
  uncurryThis,
  encodingsMap,
};