// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:internal/crypto - Internal cryptography layer (L2)
// Not directly accessible to user code. Used by L3 compatibility layers.

var __koss_hash = globalThis.__koss_hash;
var __koss_random_bytes = globalThis.__koss_random_bytes;
var __koss_random_uuid = globalThis.__koss_random_uuid;
var __koss_hmac = globalThis.__koss_hmac;
var __koss_pbkdf2 = globalThis.__koss_pbkdf2;
var __koss_hash_bytes = globalThis.__koss_hash_bytes;
var __koss_hmac_bytes = globalThis.__koss_hmac_bytes;
var __koss_pbkdf2_bytes = globalThis.__koss_pbkdf2_bytes;
var __koss_aes_gcm_encrypt = globalThis.__koss_aes_gcm_encrypt;
var __koss_aes_gcm_decrypt = globalThis.__koss_aes_gcm_decrypt;
var __koss_ed25519_keypair = globalThis.__koss_ed25519_keypair;
var __koss_ed25519_sign = globalThis.__koss_ed25519_sign;
var __koss_ed25519_verify = globalThis.__koss_ed25519_verify;
var __koss_timing_safe_equal = globalThis.__koss_timing_safe_equal;

function _toHex(data) {
  var hex = '';
  for (var i = 0; i < data.length; i++) {
    var h = data[i].toString(16);
    if (h.length < 2) h = '0' + h;
    hex += h;
  }
  return hex;
}

function _parseJsonBytes(result) {
  if (typeof result === 'string' && result.charAt(0) === '[') {
    try { return new Uint8Array(JSON.parse(result)); } catch (e) { return null; }
  }
  if (result instanceof Uint8Array) return result;
  if (Array.isArray(result)) return new Uint8Array(result);
  if (result && typeof result.length === 'number' && result.length > 0) {
    var len = result.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) bytes[i] = result[i];
    return bytes;
  }
  return null;
}

function hash(algorithm, data) {
  if (typeof __koss_hash === 'function') {
    var result = __koss_hash(String(algorithm), String(data));
    if (result && typeof result === 'string') return result;
    throw new Error('Hash failed: ' + algorithm);
  }
  throw new Error('Hash capability not available');
}

function hashBytes(algorithm, data) {
  if (typeof __koss_hash_bytes !== 'function') {
    throw new Error('hashBytes capability not available');
  }
  var input = _toBytesInput(data);
  var algoStr = String(algorithm);
  var inputStr = JSON.stringify(Array.from(input));
  var result = __koss_hash_bytes(algoStr, inputStr);
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('hashBytes failed: ' + algorithm);
  return bytes;
}

function randomBytes(size) {
  if (typeof __koss_random_bytes === 'function') {
    var result = __koss_random_bytes(size === undefined ? 32 : Number(size));
    if (result && typeof result === 'string') {
      try {
        var arr = JSON.parse(result);
        return Uint8Array.from(arr);
      } catch (e) {
        return result;
      }
    }
    throw new Error('randomBytes failed');
  }
  throw new Error('Random bytes capability not available');
}

function randomUUID() {
  if (typeof __koss_random_uuid === 'function') {
    return __koss_random_uuid();
  }
  throw new Error('Random UUID capability not available');
}

function hmac(algorithm, key, data) {
  if (typeof __koss_hmac === 'function') {
    var result = __koss_hmac(String(algorithm), String(key), String(data));
    if (result && typeof result === 'string') return result;
    throw new Error('HMAC failed: ' + algorithm);
  }
  throw new Error('HMAC capability not available');
}

function hmacBytes(algorithm, key, data) {
  if (typeof __koss_hmac_bytes !== 'function') {
    throw new Error('hmacBytes capability not available');
  }
  var keyBytes = _toBytesInput(key);
  var dataBytes = _toBytesInput(data);
  var algoStr = String(algorithm);
  var result = __koss_hmac_bytes(algoStr, JSON.stringify(Array.from(keyBytes)), JSON.stringify(Array.from(dataBytes)));
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('hmacBytes failed: ' + algorithm);
  return bytes;
}

function pbkdf2(password, salt, iterations, keyLen) {
  if (typeof __koss_pbkdf2 === 'function') {
    var result = __koss_pbkdf2(
      String(password),
      String(salt),
      Number(iterations) || 100000,
      Number(keyLen) || 32
    );
    if (result && typeof result === 'string') return result;
    throw new Error('PBKDF2 failed');
  }
  throw new Error('PBKDF2 capability not available');
}

function pbkdf2Bytes(password, salt, iterations, keyLen) {
  if (typeof __koss_pbkdf2_bytes !== 'function') {
    throw new Error('pbkdf2Bytes capability not available');
  }
  var pwBytes = _toBytesInput(password);
  var saltBytes = _toBytesInput(salt);
  var result = __koss_pbkdf2_bytes(
    JSON.stringify(Array.from(pwBytes)),
    JSON.stringify(Array.from(saltBytes)),
    Number(iterations) || 100000,
    Number(keyLen) || 32
  );
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('pbkdf2Bytes failed');
  return bytes;
}

function _toBytesInput(data) {
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

function aesGcmEncrypt(key, nonce, aad, plaintext) {
  if (typeof __koss_aes_gcm_encrypt !== 'function') {
    throw new Error('aesGcmEncrypt capability not available');
  }
  var keyBytes = _toBytesInput(key);
  var nonceBytes = _toBytesInput(nonce);
  var aadBytes = _toBytesInput(aad);
  var ptBytes = _toBytesInput(plaintext);
  var result = __koss_aes_gcm_encrypt(
    JSON.stringify(Array.from(keyBytes)),
    JSON.stringify(Array.from(nonceBytes)),
    JSON.stringify(Array.from(aadBytes)),
    JSON.stringify(Array.from(ptBytes))
  );
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('aesGcmEncrypt failed');
  return bytes;
}

function aesGcmDecrypt(key, nonce, aad, ciphertext) {
  if (typeof __koss_aes_gcm_decrypt !== 'function') {
    throw new Error('aesGcmDecrypt capability not available');
  }
  var keyBytes = _toBytesInput(key);
  var nonceBytes = _toBytesInput(nonce);
  var aadBytes = _toBytesInput(aad);
  var ctBytes = _toBytesInput(ciphertext);
  var result = __koss_aes_gcm_decrypt(
    JSON.stringify(Array.from(keyBytes)),
    JSON.stringify(Array.from(nonceBytes)),
    JSON.stringify(Array.from(aadBytes)),
    JSON.stringify(Array.from(ctBytes))
  );
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('aesGcmDecrypt failed');
  return bytes;
}

function ed25519KeyPair() {
  if (typeof __koss_ed25519_keypair !== 'function') {
    throw new Error('ed25519KeyPair capability not available');
  }
  var result = __koss_ed25519_keypair();
  if (result && typeof result === 'object') {
    var pubkey = _parseJsonBytes(result.publicKey);
    var privkey = _parseJsonBytes(result.privateKey);
    if (pubkey && privkey) return { publicKey: pubkey, privateKey: privkey };
  }
  throw new Error('ed25519KeyPair failed');
}

function ed25519Sign(privateKey, message) {
  if (typeof __koss_ed25519_sign !== 'function') {
    throw new Error('ed25519Sign capability not available');
  }
  var keyBytes = _toBytesInput(privateKey);
  var msgBytes = _toBytesInput(message);
  var result = __koss_ed25519_sign(
    JSON.stringify(Array.from(keyBytes)),
    JSON.stringify(Array.from(msgBytes))
  );
  var bytes = _parseJsonBytes(result);
  if (!bytes) throw new Error('ed25519Sign failed');
  return bytes;
}

function ed25519Verify(publicKey, message, signature) {
  if (typeof __koss_ed25519_verify !== 'function') {
    throw new Error('ed25519Verify capability not available');
  }
  var pkBytes = _toBytesInput(publicKey);
  var msgBytes = _toBytesInput(message);
  var sigBytes = _toBytesInput(signature);
  return __koss_ed25519_verify(
    JSON.stringify(Array.from(pkBytes)),
    JSON.stringify(Array.from(msgBytes)),
    JSON.stringify(Array.from(sigBytes))
  );
}

function timingSafeEqual(a, b) {
  if (typeof __koss_timing_safe_equal !== 'function') {
    var bufA = _toBytesInput(a);
    var bufB = _toBytesInput(b);
    if (bufA.length !== bufB.length) return false;
    var result = 0;
    for (var i = 0; i < bufA.length; i++) result |= bufA[i] ^ bufB[i];
    return result === 0;
  }
  var aBytes = _toBytesInput(a);
  var bBytes = _toBytesInput(b);
  return __koss_timing_safe_equal(
    JSON.stringify(Array.from(aBytes)),
    JSON.stringify(Array.from(bBytes))
  );
}

module.exports = {
  hash: hash,
  hashBytes: hashBytes,
  randomBytes: randomBytes,
  randomUUID: randomUUID,
  hmac: hmac,
  hmacBytes: hmacBytes,
  pbkdf2: pbkdf2,
  pbkdf2Bytes: pbkdf2Bytes,
  aesGcmEncrypt: aesGcmEncrypt,
  aesGcmDecrypt: aesGcmDecrypt,
  ed25519KeyPair: ed25519KeyPair,
  ed25519Sign: ed25519Sign,
  ed25519Verify: ed25519Verify,
  timingSafeEqual: timingSafeEqual,
};
