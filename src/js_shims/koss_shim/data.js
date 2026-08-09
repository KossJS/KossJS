// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:data — Koss 原生数据结构模块
// 字节操作、编码工具（纯 Uint8Array 实现）

function encode(text) {
  var str = String(text);
  var bytes = [];
  for (var i = 0; i < str.length; i++) {
    var cc = str.charCodeAt(i);
    if (cc < 0x80) {
      bytes.push(cc);
    } else if (cc < 0x800) {
      bytes.push(0xc0 | (cc >> 6));
      bytes.push(0x80 | (cc & 0x3f));
    } else if (cc < 0x10000) {
      bytes.push(0xe0 | (cc >> 12));
      bytes.push(0x80 | ((cc >> 6) & 0x3f));
      bytes.push(0x80 | (cc & 0x3f));
    } else {
      bytes.push(0xf0 | (cc >> 18));
      bytes.push(0x80 | ((cc >> 12) & 0x3f));
      bytes.push(0x80 | ((cc >> 6) & 0x3f));
      bytes.push(0x80 | (cc & 0x3f));
    }
  }
  return new Uint8Array(bytes);
}

function decode(bytes) {
  if (!(bytes instanceof Uint8Array)) return String(bytes);
  var chars = [];
  var i = 0;
  while (i < bytes.length) {
    var b = bytes[i++];
    if (b < 0x80) {
      chars.push(b);
    } else if (b < 0xe0) {
      chars.push(((b & 0x1f) << 6) | (bytes[i++] & 0x3f));
    } else if (b < 0xf0) {
      var b2 = bytes[i++] & 0x3f;
      var b3 = bytes[i++] & 0x3f;
      chars.push(((b & 0x0f) << 12) | (b2 << 6) | b3);
    } else {
      var b2_ = bytes[i++] & 0x3f;
      var b3_ = bytes[i++] & 0x3f;
      var b4 = bytes[i++] & 0x3f;
      chars.push(((b & 0x07) << 18) | (b2_ << 12) | (b3_ << 6) | b4);
    }
  }
  return String.fromCharCode.apply(null, chars);
}

function concat() {
  var buffers = [];
  for (var i = 0; i < arguments.length; i++) buffers.push(arguments[i]);
  var totalLength = buffers.reduce(function(acc, buf) {
    var b = buf instanceof Uint8Array ? buf : new Uint8Array(0);
    return acc + b.length;
  }, 0);
  var result = new Uint8Array(totalLength);
  var offset = 0;
  for (var j = 0; j < buffers.length; j++) {
    var b = buffers[j] instanceof Uint8Array ? buffers[j] : new Uint8Array(0);
    result.set(b, offset);
    offset += b.length;
  }
  return result;
}

function compare(a, b) {
  var bytesA = a instanceof Uint8Array ? a : new Uint8Array(0);
  var bytesB = b instanceof Uint8Array ? b : new Uint8Array(0);
  var minLen = Math.min(bytesA.length, bytesB.length);
  for (var i = 0; i < minLen; i++) {
    if (bytesA[i] < bytesB[i]) return -1;
    if (bytesA[i] > bytesB[i]) return 1;
  }
  if (bytesA.length < bytesB.length) return -1;
  if (bytesA.length > bytesB.length) return 1;
  return 0;
}

function isEqual(a, b) {
  var bytesA = a instanceof Uint8Array ? a : new Uint8Array(0);
  var bytesB = b instanceof Uint8Array ? b : new Uint8Array(0);
  if (bytesA.length !== bytesB.length) return false;
  for (var i = 0; i < bytesA.length; i++) {
    if (bytesA[i] !== bytesB[i]) return false;
  }
  return true;
}

function toHex(bytes) {
  var arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(0);
  var hex = '';
  for (var i = 0; i < arr.length; i++) {
    var h = arr[i].toString(16);
    if (h.length < 2) h = '0' + h;
    hex += h;
  }
  return hex;
}

function fromHex(hex) {
  var str = String(hex);
  var bytes = new Uint8Array(str.length / 2);
  for (var i = 0; i < str.length; i += 2) {
    bytes[i / 2] = parseInt(str.substring(i, i + 2), 16);
  }
  return bytes;
}

var _b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
var _b64lookup = {};
for (var i = 0; i < _b64chars.length; i++) _b64lookup[_b64chars[i]] = i;

function toBase64(bytes) {
  var arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(0);
  var result = '';
  for (var i = 0; i < arr.length; i += 3) {
    var a = arr[i];
    var b = i + 1 < arr.length ? arr[i + 1] : 0;
    var c = i + 2 < arr.length ? arr[i + 2] : 0;
    var triple = (a << 16) | (b << 8) | c;
    result += _b64chars[(triple >> 18) & 0x3F];
    result += _b64chars[(triple >> 12) & 0x3F];
    result += i + 1 < arr.length ? _b64chars[(triple >> 6) & 0x3F] : '=';
    result += i + 2 < arr.length ? _b64chars[triple & 0x3F] : '=';
  }
  return result;
}

function fromBase64(b64) {
  var str = String(b64).replace(/[^A-Za-z0-9+/=]/g, '');
  var bytes = new Uint8Array(Math.floor(str.length * 3 / 4));
  var idx = 0;
  for (var i = 0; i < str.length; i += 4) {
    var a = _b64lookup[str[i]] || 0;
    var b = _b64lookup[str[i + 1]] || 0;
    var c = _b64lookup[str[i + 2]] || 0;
    var d = _b64lookup[str[i + 3]] || 0;
    var triple = (a << 18) | (b << 12) | (c << 6) | d;
    bytes[idx++] = (triple >> 16) & 0xFF;
    if (str[i + 2] !== '=') bytes[idx++] = (triple >> 8) & 0xFF;
    if (str[i + 3] !== '=') bytes[idx++] = triple & 0xFF;
  }
  return bytes.slice(0, idx);
}

function slice(bytes, start, end) {
  if (!(bytes instanceof Uint8Array)) return new Uint8Array(0);
  var s = start || 0;
  var e = end !== undefined ? end : bytes.length;
  if (s < 0) s = Math.max(0, bytes.length + s);
  if (e < 0) e = Math.max(0, bytes.length + e);
  if (s >= bytes.length) return new Uint8Array(0);
  return bytes.subarray(s, e);
}

function indexOf(bytes, search, start) {
  if (!(bytes instanceof Uint8Array)) return -1;
  var searchBytes = search instanceof Uint8Array ? search : fromHex(String(search));
  var from = start || 0;
  for (var i = from; i <= bytes.length - searchBytes.length; i++) {
    var found = true;
    for (var j = 0; j < searchBytes.length; j++) {
      if (bytes[i + j] !== searchBytes[j]) { found = false; break; }
    }
    if (found) return i;
  }
  return -1;
}

function includes(bytes, search, start) {
  return indexOf(bytes, search, start) !== -1;
}

function repeat(bytes, count) {
  if (!(bytes instanceof Uint8Array) || count <= 0) return new Uint8Array(0);
  var result = new Uint8Array(bytes.length * count);
  for (var i = 0; i < count; i++) {
    result.set(bytes, i * bytes.length);
  }
  return result;
}

function fill(bytes, value, start, end) {
  if (!(bytes instanceof Uint8Array)) return bytes;
  var s = start || 0;
  var e = end !== undefined ? end : bytes.length;
  var v = typeof value === 'number' ? value : 0;
  for (var i = s; i < e; i++) bytes[i] = v & 0xff;
  return bytes;
}

function toUTF8(text) {
  return encode(String(text));
}

function fromUTF8(bytes) {
  return decode(bytes);
}

module.exports = {
  encode: encode, decode: decode, concat: concat,
  compare: compare, isEqual: isEqual,
  toHex: toHex, fromHex: fromHex,
  toBase64: toBase64, fromBase64: fromBase64,
  slice: slice, indexOf: indexOf, includes: includes,
  repeat: repeat, fill: fill,
  toUTF8: toUTF8, fromUTF8: fromUTF8,
};
