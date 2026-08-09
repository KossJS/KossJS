// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

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
  var len = size !== undefined && size !== null ? size : buffer.length - off;
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
      var bytes = kossCrypto.hashBytes(algo, combined);
      if (encoding === 'hex' || !encoding) return Buffer.from(bytes).toString('hex');
      if (encoding === 'base64') return Buffer.from(bytes).toString('base64');
      if (encoding === 'latin1') return Buffer.from(bytes).toString('latin1');
      return Buffer.from(bytes);
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
      var bytes = kossCrypto.hmacBytes(algo, keyBuf, msg);
      if (encoding === 'hex' || !encoding) return Buffer.from(bytes).toString('hex');
      if (encoding === 'base64') return Buffer.from(bytes).toString('base64');
      if (encoding === 'latin1') return Buffer.from(bytes).toString('latin1');
      return Buffer.from(bytes);
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

function getCiphers() { return ['aes-128-gcm', 'aes-192-gcm', 'aes-256-gcm']; }
function getCurves() { return ['ed25519']; }

function generateKeyPairSync(type, options) {
  if (type === 'ed25519') {
    var kp = kossCrypto.ed25519KeyPair();
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
  throw new Error('Signing with algorithm "' + algorithm + '" is not supported. Only ed25519 is available.');
}

function verify(algorithm, data, key, signature) {
  var algo = (algorithm || 'sha256').toLowerCase().replace('-', '');
  var msgBuf = toBuffer(data);
  var sigBuf = toBuffer(signature);
  if (algo === 'ed25519') {
    return kossCrypto.verify(toBuffer(key), msgBuf, sigBuf);
  }
  throw new Error('Verifying with algorithm "' + algorithm + '" is not supported. Only ed25519 is available.');
}

function _parseCipherAlgorithm(algorithm) {
  var a = String(algorithm).toLowerCase().replace('_', '-');
  if (a === 'aes-128-gcm') return { keyLen: 16 };
  if (a === 'aes-192-gcm') return { keyLen: 24 };
  if (a === 'aes-256-gcm') return { keyLen: 32 };
  return null;
}

function _encodeOutput(bytes, encoding) {
  if (encoding === 'hex') return Buffer.from(bytes).toString('hex');
  if (encoding === 'base64') return Buffer.from(bytes).toString('base64');
  if (encoding === 'latin1') return Buffer.from(bytes).toString('latin1');
  return Buffer.from(bytes);
}

function createCipheriv(algorithm, key, iv) {
  var spec = _parseCipherAlgorithm(algorithm);
  if (!spec) throw new Error('Unsupported cipher: ' + algorithm);
  var keyBuf = toBuffer(key);
  if (keyBuf.length !== spec.keyLen) {
    throw new Error('Invalid key length ' + keyBuf.length + ' for ' + algorithm);
  }
  var ivBuf = iv ? toBuffer(iv) : new Uint8Array(0);
  if (ivBuf.length !== 12) {
    throw new Error('Invalid IV length ' + ivBuf.length + ' for AES-GCM (expected 12 bytes)');
  }
  var chunks = [];
  var aad = new Uint8Array(0);
  var cipherState = null;
  return {
    update: function(data, inputEncoding, outputEncoding) {
      chunks.push(toBuffer(data));
      return '';
    },
    setAAD: function(buf) { aad = toBuffer(buf); return this; },
    final: function(outputEncoding) {
      if (cipherState) return '';
      var plaintext = Buffer.concat(chunks);
      var ct = kossCrypto.encrypt(keyBuf, plaintext, { nonce: ivBuf, aad: aad });
      if (ct.length < 16) throw new Error('AES-GCM encryption produced invalid output');
      var body = ct.subarray(0, ct.length - 16);
      cipherState = { authTag: Buffer.from(ct.subarray(ct.length - 16)) };
      return _encodeOutput(body, outputEncoding);
    },
    getAuthTag: function() {
      if (!cipherState) throw new Error('Cannot get auth tag before final()');
      return cipherState.authTag;
    },
  };
}

function createDecipheriv(algorithm, key, iv) {
  var spec = _parseCipherAlgorithm(algorithm);
  if (!spec) throw new Error('Unsupported cipher: ' + algorithm);
  var keyBuf = toBuffer(key);
  if (keyBuf.length !== spec.keyLen) {
    throw new Error('Invalid key length ' + keyBuf.length + ' for ' + algorithm);
  }
  var ivBuf = iv ? toBuffer(iv) : new Uint8Array(0);
  if (ivBuf.length !== 12) {
    throw new Error('Invalid IV length ' + ivBuf.length + ' for AES-GCM (expected 12 bytes)');
  }
  var chunks = [];
  var aad = new Uint8Array(0);
  var authTag = null;
  var decrypted = false;
  return {
    update: function(data, inputEncoding, outputEncoding) {
      chunks.push(toBuffer(data));
      return '';
    },
    setAAD: function(buf) { aad = toBuffer(buf); return this; },
    setAuthTag: function(tag) { authTag = toBuffer(tag); return this; },
    final: function(outputEncoding) {
      if (decrypted) return '';
      if (!authTag) throw new Error('Cannot decrypt without auth tag (call setAuthTag first)');
      var ciphertext = Buffer.concat(chunks);
      var combined = new Uint8Array(ciphertext.length + authTag.length);
      combined.set(ciphertext, 0);
      combined.set(authTag, ciphertext.length);
      var pt = kossCrypto.decrypt(keyBuf, combined, { nonce: ivBuf, aad: aad });
      decrypted = true;
      return _encodeOutput(pt, outputEncoding);
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
