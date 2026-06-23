'use strict';

var binding;
try {
  binding = internalBinding('util');
} catch (_) {
  binding = {};
}

var getSystemErrorName;
if (binding && typeof binding.getSystemErrorName === 'function') {
  getSystemErrorName = binding.getSystemErrorName;
} else {
  getSystemErrorName = function(errno) { return 'Unknown error (' + errno + ')'; };
}

var getSystemErrorMap;
if (binding && typeof binding.getSystemErrorMap === 'function') {
  getSystemErrorMap = binding.getSystemErrorMap;
} else {
  getSystemErrorMap = function() { return {}; };
}

function _formatValue(v) {
  if (typeof v === 'string') return v;
  return inspect(v);
}

function format(f) {
  if (typeof f !== 'string') {
    var arr = [];
    for (var i = 0; i < arguments.length; i++) arr.push(_formatValue(arguments[i]));
    return arr.join(' ');
  }
  var args = arguments;
  var index = 1;
  var str = String(f).replace(/%[sdifoO%]/g, function(match) {
    if (match === '%%') return '%';
    if (index >= args.length) return match;
    switch (match) {
      case '%s': return String(args[index++]);
      case '%d':
      case '%i': return parseInt(args[index++], 10);
      case '%f': return parseFloat(args[index++]);
      case '%o':
      case '%O': return _formatValue(args[index++]);
      default: return match;
    }
  });
  for (; index < args.length; index++) str += ' ' + _formatValue(args[index]);
  return str;
}

function formatWithOptions(options, f) {
  if (typeof f !== 'string') {
    var arr = [];
    for (var i = 1; i < arguments.length; i++) arr.push(inspect(arguments[i]));
    return arr.join(' ');
  }
  var args = arguments;
  var index = 2;
  var str = String(f).replace(/%[sdifoO%]/g, function(match) {
    if (match === '%%') return '%';
    if (index >= args.length) return match;
    switch (match) {
      case '%s': return String(args[index++]);
      case '%d':
      case '%i': return parseInt(args[index++], 10);
      case '%f': return parseFloat(args[index++]);
      case '%o':
      case '%O': return inspect(args[index++], options);
      default: return match;
    }
  });
  for (; index < args.length; index++) str += ' ' + inspect(args[index]);
  return str;
}

function inspect(obj, opts) {
  var depth;
  var maxArrayLength;
  var showHidden;

  if (typeof opts === 'object' && opts !== null) {
    depth = opts.depth;
    maxArrayLength = opts.maxArrayLength;
    showHidden = opts.showHidden;
  }

  if (depth === undefined || depth === null) depth = 2;
  if (maxArrayLength === undefined || maxArrayLength === null) maxArrayLength = 100;

  return _inspect(obj, depth, showHidden, maxArrayLength, 0);
}

function _inspect(obj, depth, showHidden, maxArrayLength, level) {
  if (obj === null) return 'null';
  if (obj === undefined) return 'undefined';
  if (typeof obj === 'boolean') return obj.toString();
  if (typeof obj === 'number') return obj.toString();
  if (typeof obj === 'string') return "'" + obj.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/\n/g, '\\n').replace(/\r/g, '\\r').replace(/\t/g, '\\t') + "'";
  if (typeof obj === 'function') return '[Function: ' + (obj.name || '(anonymous)') + ']';
  if (typeof obj === 'symbol') return obj.toString();

  if (level >= depth) {
    if (typeof obj === 'object') return '[Object]';
    return String(obj);
  }

  if (Array.isArray(obj)) {
    var len = Math.min(obj.length, maxArrayLength);
    var items = [];
    for (var i = 0; i < len; i++) {
      items.push(_inspect(obj[i], depth, showHidden, maxArrayLength, level + 1));
    }
    if (obj.length > maxArrayLength) items.push('... ' + (obj.length - maxArrayLength) + ' more items');
    return '[' + items.join(', ') + ']';
  }

  if (obj instanceof Date) return obj.toISOString();
  if (obj instanceof RegExp) return obj.toString();
  if (obj instanceof Map) {
    var mapItems = [];
    var mapEntries = obj.entries();
    var mapEntry = mapEntries.next();
    while (!mapEntry.done) {
      mapItems.push(_inspect(mapEntry.value[0], depth, showHidden, maxArrayLength, level + 1) + ' => ' + _inspect(mapEntry.value[1], depth, showHidden, maxArrayLength, level + 1));
      mapEntry = mapEntries.next();
    }
    return 'Map(' + mapItems.length + ') {' + mapItems.join(', ') + '}';
  }
  if (obj instanceof Set) {
    var setItems = [];
    var setValues = obj.values();
    var setValue = setValues.next();
    while (!setValue.done) {
      setItems.push(_inspect(setValue.value, depth, showHidden, maxArrayLength, level + 1));
      setValue = setValues.next();
    }
    return 'Set(' + setItems.length + ') {' + setItems.join(', ') + '}';
  }

  var keys = Object.keys(obj);
  if (showHidden) {
    var allKeys = Object.getOwnPropertyNames(obj);
    for (var k = 0; k < allKeys.length; k++) {
      if (keys.indexOf(allKeys[k]) === -1) keys.push(allKeys[k]);
    }
  }
  var props = [];
  for (var j = 0; j < keys.length; j++) {
    var key = keys[j];
    var val = obj[key];
    props.push(key + ': ' + _inspect(val, depth, showHidden, maxArrayLength, level + 1));
  }
  var constructorName = obj.constructor && obj.constructor.name ? obj.constructor.name : 'Object';
  return constructorName + ' {' + props.join(', ') + '}';
}

function deprecate(fn, msg, code) {
  if (typeof fn !== 'function') throw new TypeError('fn must be a function');
  var warned = false;
  var deprecated = function() {
    if (!warned) {
      warned = true;
      var warning = 'Deprecation' + (code ? ' [' + code + ']' : '') + ': ' + msg;
      if (typeof process !== 'undefined' && typeof process.emitWarning === 'function') {
        process.emitWarning(warning, 'DeprecationWarning');
      } else {
        console.error(warning);
      }
    }
    return fn.apply(this, arguments);
  };
  return deprecated;
}

function promisify(original) {
  if (typeof original !== 'function') throw new TypeError('fn must be a function');
  var fn = function() {
    var self = this;
    var args = Array.prototype.slice.call(arguments);
    return new Promise(function(resolve, reject) {
      args.push(function(err) {
        if (err) return reject(err);
        var results = Array.prototype.slice.call(arguments, 1);
        resolve(results.length <= 1 ? results[0] : results);
      });
      original.apply(self, args);
    });
  };
  Object.defineProperty(fn, 'name', { value: original.name + ' promisified' });
  return fn;
}

function callbackify(original) {
  if (typeof original !== 'function') throw new TypeError('fn must be a function');
  var fn = function() {
    var args = Array.prototype.slice.call(arguments);
    var cb = args.pop();
    if (typeof cb !== 'function') throw new TypeError('last argument must be a function');
    var self = this;
    original.apply(self, args).then(function(val) {
      process.nextTick(cb, null, val);
    }, function(err) {
      process.nextTick(cb, err);
    });
  };
  return fn;
}

function inherits(ctor, superCtor) {
  if (ctor === undefined || ctor === null) throw new TypeError('ctor must be a function');
  if (superCtor === undefined || superCtor === null) throw new TypeError('superCtor must be a function');
  ctor.super_ = superCtor;
  ctor.prototype = Object.create(superCtor.prototype, {
    constructor: {
      value: ctor,
      writable: true,
      configurable: true,
    },
  });
}

function debuglog(section, cb) {
  var fn = function debug() {
    if (!debuglog.enabled) return;
    var msg = format.apply(null, arguments);
    process.stderr.write(section + ': ' + msg + '\n');
  };
  fn.enabled = false;
  if (typeof process !== 'undefined' && process.env && process.env.NODE_DEBUG) {
    var sections = process.env.NODE_DEBUG.split(',');
    for (var i = 0; i < sections.length; i++) {
      if (sections[i].trim().toLowerCase() === section.toLowerCase()) {
        fn.enabled = true;
        break;
      }
    }
  }
  if (typeof cb === 'function') cb(fn);
  return fn;
}

function isArray(value) { return Array.isArray(value); }
function isBoolean(value) { return typeof value === 'boolean'; }
function isNull(value) { return value === null; }
function isNullOrUndefined(value) { return value === null || value === undefined; }
function isNumber(value) { return typeof value === 'number'; }
function isString(value) { return typeof value === 'string'; }
function isSymbol(value) { return typeof value === 'symbol'; }
function isUndefined(value) { return value === undefined; }
function isRegExp(value) { return value instanceof RegExp; }
function isDate(value) { return value instanceof Date; }
function isError(value) { return value instanceof Error; }
function isFunction(value) { return typeof value === 'function'; }
function isPrimitive(value) {
  return value === null || value === undefined || typeof value === 'boolean' || typeof value === 'number' || typeof value === 'string' || typeof value === 'symbol' || typeof value === 'bigint';
}
function isObject(value) { return value !== null && typeof value === 'object'; }
function isBuffer(value) { return value instanceof Uint8Array && value.constructor && value.constructor.name === 'Buffer'; }
function isDeepStrictEqual(a, b) { return a === b; }
function isPlainObject(value) {
  if (value === null || typeof value !== 'object') return false;
  var proto = Object.getPrototypeOf(value);
  return proto === null || proto === Object.prototype;
}

function getSystemErrorName(errno) {
  return getSystemErrorName(errno);
}

function getSystemErrorMap() {
  return getSystemErrorMap();
}

function stripVTControlCharacters(str) {
  return str.replace(/\x1b\[[0-9;]*m/g, '');
}

var types = {
  isArray: isArray,
  isBoolean: isBoolean,
  isNull: isNull,
  isNullOrUndefined: isNullOrUndefined,
  isNumber: isNumber,
  isString: isString,
  isSymbol: isSymbol,
  isUndefined: isUndefined,
  isRegExp: isRegExp,
  isDate: isDate,
  isError: isError,
  isFunction: isFunction,
  isPrimitive: isPrimitive,
  isObject: isObject,
  isBuffer: isBuffer,
  isDeepStrictEqual: isDeepStrictEqual,
  isPlainObject: isPlainObject,
};

var TextEncoder = globalThis.TextEncoder;
var TextDecoder = globalThis.TextDecoder;

module.exports = {
  format: format,
  formatWithOptions: formatWithOptions,
  inspect: inspect,
  deprecate: deprecate,
  promisify: promisify,
  callbackify: callbackify,
  inherits: inherits,
  debuglog: debuglog,
  getSystemErrorName: getSystemErrorName,
  getSystemErrorMap: getSystemErrorMap,
  stripVTControlCharacters: stripVTControlCharacters,
  types: types,
  TextEncoder: TextEncoder,
  TextDecoder: TextDecoder,
};
