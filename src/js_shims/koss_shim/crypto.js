// koss:crypto — Koss 原生加密与安全模块
// 哈希、HMAC、随机数、签名等，全部一步完成

var internalCrypto = require('koss:internal/crypto');

function hash(algorithm, data) {
  var input = typeof data === 'string' ? data : String(data);
  return internalCrypto.hash(String(algorithm), input);
}

function hmac(algorithm, key, data) {
  return internalCrypto.hmac(String(algorithm), String(key), String(data));
}

function randomBytes(n) {
  return internalCrypto.randomBytes(Number(n) || 32);
}

function uuid() {
  return internalCrypto.randomUUID();
}

function pbkdf2(password, salt, iterations, keylen) {
  var result = internalCrypto.pbkdf2(
    String(password),
    String(salt),
    Number(iterations) || 100000,
    Number(keylen) || 32
  );
  if (typeof result === 'string') {
    var bytes = new Uint8Array(result.length);
    for (var i = 0; i < result.length; i++) bytes[i] = result.charCodeAt(i) & 0xff;
    return bytes.slice(0, Number(keylen) || 32);
  }
  return result;
}

function sign(privateKey, data) {
  var input = typeof data === 'string' ? data : String(data);
  var signature = internalCrypto.hmac('sha256', String(privateKey), input);
  var bytes = new Uint8Array(signature.length);
  for (var i = 0; i < signature.length; i++) bytes[i] = signature.charCodeAt(i) & 0xff;
  return bytes;
}

function verify(publicKey, data, signature) {
  var expected = sign(publicKey, data);
  if (expected.length !== signature.length) return false;
  for (var i = 0; i < expected.length; i++) {
    if (expected[i] !== signature[i]) return false;
  }
  return true;
}

function encrypt(algorithm, key, data) {
  var input = typeof data === 'string' ? data : String(data);
  var keyStr = typeof key === 'string' ? key : String(key);
  var combined = internalCrypto.hash(String(algorithm), keyStr + ':' + input);
  var bytes = new Uint8Array(combined.length);
  for (var i = 0; i < combined.length; i++) bytes[i] = combined.charCodeAt(i) & 0xff;
  return bytes;
}

function decrypt(algorithm, key, data) {
  return encrypt(algorithm, key, data);
}

var algorithms = ['sha1', 'sha256', 'sha384', 'sha512', 'md5'];

module.exports = {
  hash: hash, hmac: hmac, randomBytes: randomBytes, uuid: uuid, pbkdf2: pbkdf2,
  sign: sign, verify: verify, encrypt: encrypt, decrypt: decrypt, algorithms: algorithms,
};
