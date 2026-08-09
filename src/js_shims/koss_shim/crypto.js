// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

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
  if (data && typeof data.length === 'number' && data.length > 0) {
    var len = data.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) bytes[i] = data[i];
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

function _deriveKey32(keyStr) {
  var keyBytes = _toBytes(keyStr);
  if (keyBytes.length >= 32) return keyBytes.slice(0, 32);
  var derived = internalCrypto.hashBytes('sha256', keyBytes);
  return derived.slice(0, 32);
}

function _deriveAESKey(keyStr) {
  var keyBytes = _toBytes(keyStr);
  if (keyBytes.length === 32) return keyBytes;
  if (keyBytes.length > 32) return keyBytes.slice(0, 32);
  var derived = internalCrypto.hashBytes('sha256', keyBytes);
  if (!(derived instanceof Uint8Array)) derived = new Uint8Array(derived);
  return derived.slice(0, 32);
}

function hash(algorithm, data) {
  return _toHex(internalCrypto.hashBytes(String(algorithm), _toBytes(data)));
}

function hashHex(algorithm, data) {
  return hash(algorithm, data);
}

function hmac(algorithm, key, data) {
  return _toHex(internalCrypto.hmacBytes(String(algorithm), _toBytes(key), _toBytes(data)));
}

function hmacHex(algorithm, key, data) {
  return hmac(algorithm, key, data);
}

function randomBytes(n) {
  return internalCrypto.randomBytes(n === undefined ? 32 : Number(n));
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

function ed25519KeyPair() {
  var kp = internalCrypto.ed25519KeyPair();
  return {
    publicKey: kp.publicKey,
    privateKey: kp.privateKey,
  };
}

function hashBytes(algorithm, data) {
  return internalCrypto.hashBytes(String(algorithm), _toBytes(data));
}

function hmacBytes(algorithm, key, data) {
  return internalCrypto.hmacBytes(String(algorithm), _toBytes(key), _toBytes(data));
}

function encrypt(key, data, options) {
  var opts = options || {};
  var aad = opts.aad || new Uint8Array(0);
  var nonce = opts.nonce || randomBytes(12);
  var ciphertext = internalCrypto.aesGcmEncrypt(_toBytes(key), nonce, aad, _toBytes(data));
  if (opts.nonce) return ciphertext;
  var result = new Uint8Array(nonce.length + ciphertext.length);
  result.set(nonce, 0);
  result.set(ciphertext, nonce.length);
  return result;
}

function decrypt(key, ciphertext, options) {
  var opts = options || {};
  var aad = opts.aad || new Uint8Array(0);
  var ctBytes = _toBytes(ciphertext);
  var nonce;
  var body;
  if (opts.nonce) {
    nonce = _toBytes(opts.nonce);
    body = ctBytes;
  } else {
    nonce = ctBytes.slice(0, 12);
    body = ctBytes.slice(12);
  }
  return internalCrypto.aesGcmDecrypt(_toBytes(key), nonce, aad, body);
}

function _parseCipherAlgorithm(algorithm) {
  var a = String(algorithm).toLowerCase().replace('_', '-');
  if (a === 'aes-128-gcm') return { keyLen: 16 };
  if (a === 'aes-192-gcm') return { keyLen: 24 };
  if (a === 'aes-256-gcm') return { keyLen: 32 };
  return null;
}

function createCipher(algorithm, key) {
  var spec = _parseCipherAlgorithm(algorithm);
  if (!spec) throw new Error('Unsupported cipher: ' + algorithm);
  var keyBytes = _toBytes(key);
  if (keyBytes.length !== spec.keyLen) {
    throw new Error('Invalid key length ' + keyBytes.length + ' for ' + algorithm);
  }
  var nonce = randomBytes(12);
  var chunks = [];
  var aad = new Uint8Array(0);
  return {
    update: function(data) {
      chunks.push(_toBytes(data));
      return '';
    },
    setAAD: function(buf) { aad = _toBytes(buf); return this; },
    final: function() {
      var plaintext = _concatChunks(chunks);
      var ciphertext = internalCrypto.aesGcmEncrypt(keyBytes, nonce, aad, plaintext);
      var result = new Uint8Array(nonce.length + ciphertext.length);
      result.set(nonce, 0);
      result.set(ciphertext, nonce.length);
      return result;
    },
    getAuthTag: function() {
      var all = _concatChunks(chunks);
      var ciphertext = internalCrypto.aesGcmEncrypt(keyBytes, nonce, aad, all);
      return ciphertext.slice(ciphertext.length - 16);
    },
  };
}

function createDecipher(algorithm, key) {
  var spec = _parseCipherAlgorithm(algorithm);
  if (!spec) throw new Error('Unsupported cipher: ' + algorithm);
  var keyBytes = _toBytes(key);
  if (keyBytes.length !== spec.keyLen) {
    throw new Error('Invalid key length ' + keyBytes.length + ' for ' + algorithm);
  }
  var chunks = [];
  var aad = new Uint8Array(0);
  var authTag = null;
  return {
    update: function(data) {
      chunks.push(_toBytes(data));
      return '';
    },
    setAAD: function(buf) { aad = _toBytes(buf); return this; },
    setAuthTag: function(tag) { authTag = _toBytes(tag); return this; },
    final: function() {
      var combined = _concatChunks(chunks);
      if (combined.length < 12) throw new Error('Ciphertext too short');
      var nonce = combined.slice(0, 12);
      var body = combined.slice(12);
      return internalCrypto.aesGcmDecrypt(keyBytes, nonce, aad, body);
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
  hashBytes: hashBytes, hmacBytes: hmacBytes,
  randomBytes: randomBytes, uuid: uuid, pbkdf2: pbkdf2,
  sign: sign, verify: verify,
  ed25519KeyPair: ed25519KeyPair,
  encrypt: encrypt, decrypt: decrypt,
  createCipher: createCipher, createDecipher: createDecipher,
  timingSafeEqual: timingSafeEqual, algorithms: algorithms,
};
