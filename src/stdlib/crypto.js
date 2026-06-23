'use strict';

var Buffer = require('buffer').Buffer;

function randomBytes(size, callback) {
  if (typeof size !== 'number' || size < 0) throw new Error('size must be a number >= 0');
  if (size === 0) {
    var empty = Buffer.alloc(0);
    if (callback) { process.nextTick(function() { callback(null, empty); }); return; }
    return empty;
  }
  try {
    var jsonStr = __koss_random_bytes(size);
    var arr = JSON.parse(jsonStr);
    var buf = Buffer.from(arr);
    if (callback) { process.nextTick(function() { callback(null, buf); }); }
    return buf;
  } catch (e) {
    if (callback) { process.nextTick(function() { callback(e); }); }
    throw e;
  }
}

function randomBytesSync(size) {
  return randomBytes(size);
}

function createHash(algorithm) {
  return new Hash(algorithm);
}

function Hash(algorithm) {
  this._algorithm = algorithm.toLowerCase().replace('sha-', 'sha');
  this._parts = [];
}
Hash.prototype.update = function(data, encoding) {
  if (typeof data === 'string') {
    this._parts.push(data);
  } else if (Buffer.isBuffer(data)) {
    this._parts.push(data.toString(encoding || 'utf8'));
  }
  return this;
};
Hash.prototype.digest = function(encoding) {
  var fullData = this._parts.join('');
  var hex;
  try {
    hex = __koss_hash(this._algorithm, fullData);
  } catch (e) {
    hex = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';
  }
  if (encoding === 'hex' || !encoding) return hex;
  if (encoding === 'base64') return _hexToBase64(hex);
  return hex;
};
Hash.prototype.copy = function() {
  var h = new Hash(this._algorithm);
  h._parts = this._parts.slice();
  return h;
};

function randomUUID(options) {
  try {
    return __koss_random_uuid();
  } catch (e) {
    return '00000000-0000-4000-8000-000000000000';
  }
}

function randomFillSync(buffer, offset, size) {
  offset = offset || 0;
  size = size || buffer.length - offset;
  var jsonStr = __koss_random_bytes(size);
  var arr = JSON.parse(jsonStr);
  for (var i = 0; i < size; i++) buffer[offset + i] = arr[i];
  return buffer;
}

function randomFill(buffer, offset, size, callback) {
  if (typeof offset === 'function') { callback = offset; offset = 0; }
  if (typeof size === 'function') { callback = size; size = buffer.length - offset; }
  try {
    randomFillSync(buffer, offset, size);
    if (callback) process.nextTick(function() { callback(null, buffer); });
  } catch (e) {
    if (callback) process.nextTick(function() { callback(e); });
  }
}

function createHmac(algorithm, key) {
  return { update: function() { return this; }, digest: function() { return ''; } };
}

function _hexToBase64(hex) {
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var bytes = [];
  for (var i = 0; i < hex.length; i += 2) {
    bytes.push(parseInt(hex.substr(i, 2), 16));
  }
  var result = '';
  for (var j = 0; j < bytes.length; j += 3) {
    var b = ((bytes[j] || 0) << 16) | ((bytes[j + 1] || 0) << 8) | (bytes[j + 2] || 0);
    result += chars[(b >> 18) & 63] + chars[(b >> 12) & 63] + chars[(b >> 6) & 63] + chars[b & 63];
  }
  var pad = bytes.length % 3;
  if (pad === 1) result = result.slice(0, -2) + '==';
  else if (pad === 2) result = result.slice(0, -1) + '=';
  return result;
}

function timingSafeEqual(a, b) {
  if (!a || !b) return false;
  if (a.length !== b.length) return false;
  var result = 0;
  for (var i = 0; i < a.length; i++) result |= a[i] ^ (b[i] || 0);
  return result === 0;
}

module.exports = {
  randomBytes: randomBytes,
  randomFill: randomFill,
  randomFillSync: randomFillSync,
  randomUUID: randomUUID,
  createHash: createHash,
  createHmac: createHmac,
  timingSafeEqual: timingSafeEqual,
  Hash: Hash,
  getCiphers: function() { return []; },
  getHashes: function() { return ['sha1', 'sha256', 'md5']; },
};
