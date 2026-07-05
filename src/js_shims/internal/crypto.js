// koss:internal/crypto - Internal cryptography layer (L2)
// Not directly accessible to user code. Used by L3 compatibility layers.

var __koss_hash = globalThis.__koss_hash;
var __koss_random_bytes = globalThis.__koss_random_bytes;
var __koss_random_uuid = globalThis.__koss_random_uuid;
var __koss_hmac = globalThis.__koss_hmac;
var __koss_pbkdf2 = globalThis.__koss_pbkdf2;

function hash(algorithm, data) {
  if (typeof __koss_hash === 'function') {
    var result = __koss_hash(String(algorithm), String(data));
    if (result && typeof result === 'string') {
      return result;
    }
    throw new Error('Hash failed: ' + algorithm);
  }
  throw new Error('Hash capability not available');
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
    if (result && typeof result === 'string') {
      return result;
    }
    throw new Error('HMAC failed: ' + algorithm);
  }
  return hash(algorithm, key + ':' + data);
}

function pbkdf2(password, salt, iterations, keyLen) {
  if (typeof __koss_pbkdf2 === 'function') {
    var result = __koss_pbkdf2(
      String(password),
      String(salt),
      Number(iterations) || 100000,
      Number(keyLen) || 32
    );
    if (result && typeof result === 'string') {
      return result;
    }
    throw new Error('PBKDF2 failed');
  }
  throw new Error('PBKDF2 capability not available');
}

module.exports = {
  hash: hash,
  randomBytes: randomBytes,
  randomUUID: randomUUID,
  hmac: hmac,
  pbkdf2: pbkdf2,
};
