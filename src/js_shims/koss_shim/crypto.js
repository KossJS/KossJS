// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:crypto — Koss 原生加密与安全模块
// 哈希、HMAC、随机数、签名、对称加密，全部一步完成

var internalCrypto = require('koss:internal/crypto');

function _toBytes(data) {
  if (data instanceof Uint8Array) return data;
  if (Array.isArray(data)) return new Uint8Array(data);
  if (typeof data === 'string') {
    var bytes = new Uint8Array(data.length);
    for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
    return bytes;
  }
  return new Uint8Array(0);
}

function _toHex(bytes) {
  var hex = '';
  for (var i = 0; i < bytes.length; i++) {
    var h = bytes[i].toString(16);
    if (h.length < 2) h = '0' + h;
    hex += h;
  }
  return hex;
}

function _concatChunks(chunks) {
  var totalLen = 0;
  for (var i = 0; i < chunks.length; i++) totalLen += chunks[i].length;
  var result = new Uint8Array(totalLen);
  var offset = 0;
  for (var j = 0; j < chunks.length; j++) {
    result.set(chunks[j], offset);
    offset += chunks[j].length;
  }
  return result;
}

function hash(algorithm, data) {
  return internalCrypto.hashBytes(String(algorithm), _toBytes(data));
}

function hashHex(algorithm, data) {
  return _toHex(hash(algorithm, data));
}

function hmac(algorithm, key, data) {
  return internalCrypto.hmacBytes(String(algorithm), _toBytes(key), _toBytes(data));
}

function hmacHex(algorithm, key, data) {
  return _toHex(hmac(algorithm, key, data));
}

function randomBytes(n) {
  return internalCrypto.randomBytes(Number(n) || 32);
}

function uuid() {
  return internalCrypto.randomUUID();
}

function pbkdf2(password, salt, iterations, keylen) {
  return internalCrypto.pbkdf2Bytes(
    _toBytes(password),
    _toBytes(salt),
    Number(iterations) || 100000,
    Number(keylen) || 32
  );
}

function sign(privateKey, data) {
  return internalCrypto.ed25519Sign(_toBytes(privateKey), _toBytes(data));
}

function verify(publicKey, data, signature) {
  return internalCrypto.ed25519Verify(_toBytes(publicKey), _toBytes(data), _toBytes(signature));
}

function encrypt(key, data, options) {
  var opts = options || {};
  var nonce = opts.nonce || randomBytes(12);
  var aad = opts.aad || new Uint8Array(0);
  var ciphertext = internalCrypto.aesGcmEncrypt(_toBytes(key), nonce, aad, _toBytes(data));
  return { ciphertext: ciphertext, nonce: nonce };
}

function decrypt(key, ciphertext, options) {
  var opts = options || {};
  var nonce = opts.nonce || new Uint8Array(0);
  var aad = opts.aad || new Uint8Array(0);
  var ctBytes = _toBytes(ciphertext);
  return internalCrypto.aesGcmDecrypt(_toBytes(key), nonce, aad, ctBytes);
}

function createCipher(algorithm, key) {
  var keyBytes = _toBytes(key);
  var chunks = [];
  return {
    update: function(data) {
      chunks.push(_toBytes(data));
      return '';
    },
    final: function() {
      var plaintext = _concatChunks(chunks);
      var nonce = randomBytes(12);
      var ciphertext = internalCrypto.aesGcmEncrypt(keyBytes, nonce, new Uint8Array(0), plaintext);
      var result = new Uint8Array(nonce.length + ciphertext.length);
      result.set(nonce, 0);
      result.set(ciphertext, nonce.length);
      return result;
    },
    getAuthTag: function() { return new Uint8Array(16); },
  };
}

function createDecipher(algorithm, key) {
  var keyBytes = _toBytes(key);
  var chunks = [];
  return {
    update: function(data) {
      chunks.push(_toBytes(data));
      return '';
    },
    setAuthTag: function(tag) { return this; },
    final: function() {
      var combined = _concatChunks(chunks);
      var nonce = combined.slice(0, 12);
      var ciphertext = combined.slice(12);
      return internalCrypto.aesGcmDecrypt(keyBytes, nonce, new Uint8Array(0), ciphertext);
    },
  };
}

function timingSafeEqual(a, b) {
  return internalCrypto.timingSafeEqual(_toBytes(a), _toBytes(b));
}

var algorithms = ['sha1', 'sha256', 'sha384', 'sha512', 'md5'];

module.exports = {
  hash: hash, hashHex: hashHex,
  hmac: hmac, hmacHex: hmacHex,
  randomBytes: randomBytes, uuid: uuid, pbkdf2: pbkdf2,
  sign: sign, verify: verify,
  encrypt: encrypt, decrypt: decrypt,
  createCipher: createCipher, createDecipher: createDecipher,
  timingSafeEqual: timingSafeEqual, algorithms: algorithms,
};
