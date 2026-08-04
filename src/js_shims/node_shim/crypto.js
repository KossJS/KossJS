// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/crypto - Node.js crypto module (L3)
// Maps to koss:crypto standard library

var kossCrypto = require('koss:crypto');
var { Buffer } = require('koss:buffer');

function toBuffer(data) {
  if (Buffer.isBuffer(data)) return data;
  if (typeof data === 'string') return Buffer.from(data, 'utf8');
  if (data instanceof Uint8Array) return Buffer.from(data);
  if (ArrayBuffer.isView(data)) return Buffer.from(data.buffer, data.byteOffset, data.byteLength);
  if (data instanceof ArrayBuffer) return Buffer.from(data);
  if (Array.isArray(data)) return Buffer.from(new Uint8Array(data));
  return Buffer.from(String(data));
}

function randomBytes(size, callback) {
  var bytes = kossCrypto.randomBytes(size === undefined ? 32 : size);
  if (callback) { callback(null, bytes); return; }
  return bytes;
}

function randomUUID(options) {
  return kossCrypto.uuid();
}

function randomFillSync(buffer, offset, size) {
  var off = offset || 0;
  var len = size || buffer.length - off;
  var bytes = kossCrypto.randomBytes(len);
  for (var i = 0; i < len; i++) buffer[off + i] = bytes[i];
  return buffer;
}

function randomFill(buffer, offset, size, callback) {
  if (typeof offset === 'function') { callback = offset; offset = 0; size = buffer.length; }
  else if (typeof size === 'function') { callback = size; size = buffer.length - offset; }
  try { callback(null, randomFillSync(buffer, offset, size)); }
  catch (err) { callback(err); }
}

function createHash(algorithm) {
  var algo = algorithm.toLowerCase().replace('-', '');
  var supported = ['sha1', 'sha256', 'sha384', 'sha512', 'md5'];
  if (supported.indexOf(algo) === -1) throw new Error('Digest method not supported: ' + algorithm);

  var chunks = [];
  return {
    update: function(chunk, encoding) {
      if (typeof chunk === 'string') {
        chunks.push(Buffer.from(chunk, encoding || 'utf8'));
      } else {
        chunks.push(toBuffer(chunk));
      }
      return this;
    },
    digest: function(encoding) {
      var combined = Buffer.concat(chunks);
      var hashBytes = kossCrypto.hash(algo, combined);
      if (encoding === 'hex' || !encoding) {
        return kossCrypto.hashHex(algo, combined);
      }
      if (encoding === 'base64') return Buffer.from(hashBytes).toString('base64');
      if (encoding === 'latin1') return Buffer.from(hashBytes).toString('latin1');
      return Buffer.from(hashBytes);
    },
    copy: function() {
      var copy = createHash(algo);
      for (var i = 0; i < chunks.length; i++) copy.update(chunks[i]);
      return copy;
    },
  };
}

function createHmac(algorithm, key) {
  var algo = algorithm.toLowerCase().replace('-', '');
  var keyBuf = toBuffer(key);
  var chunks = [];
  return {
    update: function(chunk) {
      if (typeof chunk === 'string') {
        chunks.push(Buffer.from(chunk, 'utf8'));
      } else {
        chunks.push(toBuffer(chunk));
      }
      return this;
    },
    digest: function(encoding) {
      var msg = Buffer.concat(chunks);
      var macBytes = kossCrypto.hmac(algo, keyBuf, msg);
      if (encoding === 'hex' || !encoding) {
        return kossCrypto.hmacHex(algo, keyBuf, msg);
      }
      if (encoding === 'base64') return Buffer.from(macBytes).toString('base64');
      if (encoding === 'latin1') return Buffer.from(macBytes).toString('latin1');
      return Buffer.from(macBytes);
    },
  };
}

function pbkdf2(password, salt, iterations, keylen, digest, callback) {
  if (typeof digest === 'function') { callback = digest; digest = 'sha256'; }
  try {
    var keyBytes = kossCrypto.pbkdf2(toBuffer(password), toBuffer(salt), Number(iterations), Number(keylen));
    callback(null, Buffer.from(keyBytes));
  } catch (err) { callback(err); }
}

function pbkdf2Sync(password, salt, iterations, keylen, digest) {
  var keyBytes = kossCrypto.pbkdf2(toBuffer(password), toBuffer(salt), Number(iterations), Number(keylen));
  return Buffer.from(keyBytes);
}

function timingSafeEqual(a, b) {
  var bufA = toBuffer(a);
  var bufB = toBuffer(b);
  return kossCrypto.timingSafeEqual(bufA, bufB);
}

function getHashes() {
  return ['sha1', 'sha256', 'sha384', 'sha512', 'md5'];
}

function getCiphers() { return ['aes-256-gcm', 'aes-128-gcm']; }
function getCurves() { return ['ed25519']; }

function generateKeyPairSync(type, options) {
  if (type === 'ed25519') {
    var kp = kossCrypto.internalCrypto.ed25519KeyPair();
    return {
      publicKey: Buffer.from(kp.publicKey),
      privateKey: Buffer.from(kp.privateKey),
    };
  }
  throw new Error('Key pair type not supported: ' + type);
}

function sign(algorithm, data, key) {
  var algo = (algorithm || 'sha256').toLowerCase().replace('-', '');
  var msgBuf = toBuffer(data);
  if (algo === 'ed25519') {
    var sig = kossCrypto.sign(toBuffer(key), msgBuf);
    return Buffer.from(sig);
  }
  var macBytes = kossCrypto.hmac(algo, toBuffer(key), msgBuf);
  return Buffer.from(macBytes);
}

function verify(algorithm, data, key, signature) {
  var algo = (algorithm || 'sha256').toLowerCase().replace('-', '');
  var msgBuf = toBuffer(data);
  var sigBuf = toBuffer(signature);
  if (algo === 'ed25519') {
    return kossCrypto.verify(toBuffer(key), msgBuf, sigBuf);
  }
  var expected = Buffer.from(kossCrypto.hmac(algo, toBuffer(key), msgBuf));
  return expected.length === sigBuf.length && kossCrypto.timingSafeEqual(expected, sigBuf);
}

function createCipheriv(algorithm, key, iv) {
  var keyBuf = toBuffer(key);
  var ivBuf = iv ? toBuffer(iv) : new Uint8Array(0);
  var chunks = [];
  return {
    update: function(data, inputEncoding, outputEncoding) {
      chunks.push(toBuffer(data));
      return '';
    },
    final: function(outputEncoding) {
      var plaintext = Buffer.concat(chunks);
      var aad = new Uint8Array(0);
      var nonce = ivBuf.length >= 12 ? ivBuf.slice(0, 12) : kossCrypto.randomBytes(12);
      var ct = kossCrypto.encrypt(keyBuf, plaintext, { nonce: nonce, aad: aad });
      if (outputEncoding === 'hex') {
        return kossCrypto.hashHex('sha256', ct);
      }
      if (outputEncoding === 'base64') return Buffer.from(ct).toString('base64');
      return Buffer.from(ct);
    },
    getAuthTag: function() { return Buffer.alloc(16); },
  };
}

function createDecipheriv(algorithm, key, iv) {
  var keyBuf = toBuffer(key);
  var ivBuf = iv ? toBuffer(iv) : new Uint8Array(0);
  var chunks = [];
  var authTag = null;
  return {
    update: function(data, inputEncoding, outputEncoding) {
      chunks.push(toBuffer(data));
      return '';
    },
    setAuthTag: function(tag) { authTag = toBuffer(tag); return this; },
    final: function(outputEncoding) {
      var ciphertext = Buffer.concat(chunks);
      var aad = new Uint8Array(0);
      var nonce = ivBuf.length >= 12 ? ivBuf.slice(0, 12) : new Uint8Array(12);
      var pt = kossCrypto.decrypt(keyBuf, ciphertext, { nonce: nonce, aad: aad });
      if (outputEncoding === 'hex') {
        return kossCrypto.hashHex('sha256', pt);
      }
      if (outputEncoding === 'base64') return Buffer.from(pt).toString('base64');
      return Buffer.from(pt);
    },
  };
}

var webcrypto = globalThis.crypto;
var subtle = globalThis.crypto && globalThis.crypto.subtle;

module.exports = {
  randomBytes: randomBytes, randomUUID: randomUUID, randomFillSync: randomFillSync, randomFill: randomFill,
  createHash: createHash, createHmac: createHmac, pbkdf2: pbkdf2, pbkdf2Sync: pbkdf2Sync,
  timingSafeEqual: timingSafeEqual, getHashes: getHashes, getCiphers: getCiphers, getCurves: getCurves,
  generateKeyPairSync: generateKeyPairSync, sign: sign, verify: verify,
  createCipheriv: createCipheriv, createDecipheriv: createDecipheriv,
  webcrypto: webcrypto, subtle: subtle,
};
