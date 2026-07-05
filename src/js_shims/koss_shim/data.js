// koss:data — Koss 原生数据结构模块
// 字节操作、编码工具（纯 Uint8Array 实现）

function encode(text) {
  var str = String(text);
  var bytes = new Uint8Array(str.length);
  for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0xff;
  return bytes;
}

function decode(bytes) {
  if (bytes instanceof Uint8Array) {
    var chars = [];
    for (var i = 0; i < bytes.length; i++) chars.push(String.fromCharCode(bytes[i]));
    return chars.join('');
  }
  return String(bytes);
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

module.exports = {
  encode: encode, decode: decode, concat: concat,
  compare: compare, isEqual: isEqual,
  toHex: toHex, fromHex: fromHex,
  toBase64: toBase64, fromBase64: fromBase64,
};
