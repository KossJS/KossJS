// koss:node/util - Node.js util module (L3)

const { Buffer } = require('koss:node/buffer');

function format(fmt, ...args) {
  if (typeof fmt !== 'string') return inspect(fmt);
  let str = '';
  let argIndex = 0;
  for (let i = 0; i < fmt.length; i++) {
    if (fmt[i] === '%' && i + 1 < fmt.length) {
      const spec = fmt[i + 1];
      i++;
      switch (spec) {
        case 's': str += String(argIndex < args.length ? args[argIndex++] : ''); break;
        case 'd': str += String(argIndex < args.length ? Math.floor(Number(args[argIndex++])) : 0); break;
        case 'i': str += String(argIndex < args.length ? parseInt(args[argIndex++]) : 0); break;
        case 'f': str += String(argIndex < args.length ? parseFloat(args[argIndex++]) : 0); break;
        case 'j': str += argIndex < args.length ? JSON.stringify(args[argIndex++]) : 'undefined'; break;
        case 'o': case 'O': str += argIndex < args.length ? inspect(args[argIndex++]) : 'undefined'; break;
        case '%': str += '%'; break;
        default: str += '%' + spec; break;
      }
    } else {
      str += fmt[i];
    }
  }
  while (argIndex < args.length) {
    str += (str.length > 0 ? ' ' : '') + String(args[argIndex++]);
  }
  return str;
}

function inspect(obj, options) {
  const depth = options?.depth !== undefined ? options.depth : 2;
  const maxArrayLength = options?.maxArrayLength !== undefined ? options.maxArrayLength : 100;

  function formatValue(val, d, seen) {
    if (val === null) return 'null';
    if (val === undefined) return 'undefined';
    if (typeof val === 'boolean' || typeof val === 'number') return String(val);
    if (typeof val === 'string') return `'${val.replace(/'/g, "\\'").replace(/\n/g, '\\n')}'`;
    if (typeof val === 'function') return `[Function: ${val.name || 'anonymous'}]`;
    if (typeof val === 'symbol') return val.toString();
    if (typeof val !== 'object') return String(val);

    if (seen.has(val)) return '[Circular]';
    seen.add(val);

    if (d <= 0) return typeof val === 'object' ? (Array.isArray(val) ? `[Array(${val.length})]` : '[Object]') : String(val);

    if (Buffer.isBuffer(val)) return `<Buffer ${Array.from(val.slice(0, 20)).map(b => b.toString(16).padStart(2, '0')).join(' ')}${val.length > 20 ? ' ... ' + (val.length - 20) + ' more bytes' : ''}>`;

    if (Array.isArray(val)) {
      const items = val.slice(0, maxArrayLength).map(v => formatValue(v, d - 1, seen)).join(', ');
      const suffix = val.length > maxArrayLength ? ` ... ${val.length - maxArrayLength} more items` : '';
      return `[ ${items}${suffix} ]`;
    }

    if (val instanceof Date) return val.toISOString();
    if (val instanceof Map) return `Map(${val.size}) { ${Array.from(val.entries()).slice(0, 10).map(([k, v]) => `${formatValue(k, d - 1, seen)} => ${formatValue(v, d - 1, seen)}`).join(', ')} }`;
    if (val instanceof Set) return `Set(${val.size}) { ${Array.from(val).slice(0, 10).map(v => formatValue(v, d - 1, seen)).join(', ')} }`;
    if (val instanceof Error) return val.stack || `${val.name}: ${val.message}`;
    if (val instanceof RegExp) return val.toString();

    const keys = Object.keys(val);
    const entries = keys.map(k => `${k}: ${formatValue(val[k], d - 1, seen)}`);
    return `{ ${entries.join(', ')} }`;
  }

  return formatValue(obj, depth, new WeakSet());
}

function deprecate(fn, msg) {
  let warned = false;
  return function(...args) {
    if (!warned) {
      console.warn(`DeprecationWarning: ${msg}`);
      warned = true;
    }
    return fn.apply(this, args);
  };
}

function promisify(original) {
  if (typeof original !== 'function') throw new TypeError('promisify: original must be a function');

  function fn(...args) {
    return new Promise((resolve, reject) => {
      original(...args, (err, ...results) => {
        if (err) return reject(err);
        if (results.length <= 1) resolve(results[0]);
        else resolve(results);
      });
    });
  }
  Object.defineProperty(fn, 'length', { value: original.length - 1 });
  return fn;
}

function callbackify(original) {
  if (typeof original !== 'function') throw new TypeError('callbackify: original must be a function');

  function callbackified(...args) {
    const cb = args.pop();
    if (typeof cb !== 'function') throw new TypeError('callbackify: last argument must be a function');
    original.apply(this, args).then(
      (result) => process.nextTick(() => cb(null, result)),
      (err) => process.nextTick(() => cb(err)),
    );
  }
  return callbackified;
}

function inherits(ctor, superCtor) {
  if (ctor === undefined || ctor === null) throw new TypeError('ctor must be a constructor');
  if (superCtor === undefined || superCtor === null) throw new TypeError('superCtor must be a constructor');
  Object.setPrototypeOf(ctor.prototype, superCtor.prototype);
  ctor.super_ = superCtor;
}

function debuglog(section) {
  return function(...args) {
    const msg = format(...args);
    console.error(`${section}: ${msg}`);
  };
}

function isBoolean(arg) { return typeof arg === 'boolean'; }
function isNull(arg) { return arg === null; }
function isUndefined(arg) { return arg === undefined; }
function isNumber(arg) { return typeof arg === 'number'; }
function isString(arg) { return typeof arg === 'string'; }
function isSymbol(arg) { return typeof arg === 'symbol'; }
function isFunction(arg) { return typeof arg === 'function'; }
function isObject(arg) { return arg !== null && typeof arg === 'object'; }
function isRegExp(arg) { return arg instanceof RegExp; }
function isDate(arg) { return arg instanceof Date; }
function isError(arg) { return arg instanceof Error; }
function isPrimitive(arg) { return arg === null || arg === undefined || typeof arg === 'boolean' || typeof arg === 'number' || typeof arg === 'string' || typeof arg === 'symbol'; }
function isArray(arg) { return Array.isArray(arg); }
function isBuffer(arg) { return Buffer.isBuffer(arg); }
function isDeepStrictEqual(a, b) { return JSON.stringify(a) === JSON.stringify(b); }

const types = {
  isBoolean, isNull, isUndefined, isNumber, isString, isSymbol, isFunction, isObject,
  isRegExp, isDate, isError, isPrimitive, isArray, isBuffer,
};

function getSystemErrorName(code) {
  const names = {
    EACCES: 'EACCES', EADDRINUSE: 'EADDRINUSE', EADDRNOTAVAIL: 'EADDRNOTAVAIL',
    EAFNOSUPPORT: 'EAFNOSUPPORT', EAGAIN: 'EAGAIN', EALREADY: 'EALREADY',
    EBADF: 'EBADF', EBUSY: 'EBUSY', ECONNABORTED: 'ECONNABORTED',
    ECONNREFUSED: 'ECONNREFUSED', ECONNRESET: 'ECONNRESET', EEXIST: 'EEXIST',
    EFAULT: 'EFAULT', EHOSTUNREACH: 'EHOSTUNREACH', EINTR: 'EINTR', EINVAL: 'EINVAL',
    EIO: 'EIO', EISCONN: 'EISCONN', EISDIR: 'EISDIR', ELOOP: 'ELOOP',
    EMFILE: 'EMFILE', ENAMETOOLONG: 'ENAMETOOLONG', ENETDOWN: 'ENETDOWN',
    ENETUNREACH: 'ENETUNREACH', ENFILE: 'ENFILE', ENOBUFS: 'ENOBUFS',
    ENODEV: 'ENODEV', ENOENT: 'ENOENT', ENOMEM: 'ENOMEM', ENOPROTOOPT: 'ENOPROTOOPT',
    ENOSPC: 'ENOSPC', ENOSYS: 'ENOSYS', ENOTCONN: 'ENOTCONN', ENOTDIR: 'ENOTDIR',
    ENOTEMPTY: 'ENOTEMPTY', ENOTSOCK: 'ENOTSOCK', ENOTSUP: 'ENOTSUP',
    EPERM: 'EPERM', EPIPE: 'EPIPE', EPROTONOSUPPORT: 'EPROTONOSUPPORT',
    EPROTOTYPE: 'EPROTOTYPE', ERANGE: 'ERANGE', EROFS: 'EROFS',
    ESHUTDOWN: 'ESHUTDOWN', ESPIPE: 'ESPIPE', ESRCH: 'ESRCH',
    ETIMEDOUT: 'ETIMEDOUT', ETXTBSY: 'ETXTBSY', EWOULDBLOCK: 'EWOULDBLOCK',
    EXDEV: 'EXDEV',
  };
  return names[code] || 'UNKNOWN';
}

function stripVTControlCharacters(str) {
  return str.replace(/\x1B\[[0-9;]*[a-zA-Z]/g, '');
}

function toUSVString(str) {
  return typeof str === 'string' ? str : String(str);
}

function getSystemErrorMap() { return new Map(); }

module.exports = { format, inspect, deprecate, promisify, callbackify, inherits, debuglog, types, getSystemErrorName, stripVTControlCharacters, toUSVString, getSystemErrorMap };