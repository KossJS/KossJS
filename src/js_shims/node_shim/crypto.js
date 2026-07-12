// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/crypto - Node.js crypto module (L3)

const internalCrypto = require('koss:internal/crypto');
const { Buffer } = require('koss:node/buffer');

function toBuffer(data) {
  if (Buffer.isBuffer(data)) return data;
  if (typeof data === 'string') return Buffer.from(data, 'utf8');
  if (ArrayBuffer.isView(data)) return Buffer.from(data.buffer, data.byteOffset, data.byteLength);
  if (data instanceof ArrayBuffer) return Buffer.from(data);
  if (Array.isArray(data)) return Buffer.from(new Uint8Array(data));
  return Buffer.from(String(data));
}

function randomBytes(size, callback) {
  const bytes = internalCrypto.randomBytes(size === undefined ? 32 : size);
  if (callback) { callback(null, bytes); return; }
  return bytes;
}

function randomUUID(options) {
  return internalCrypto.randomUUID();
}

function randomFillSync(buffer, offset, size) {
  const off = offset || 0;
  const len = size || buffer.length - off;
  const bytes = internalCrypto.randomBytes(len);
  for (let i = 0; i < len; i++) buffer[off + i] = bytes[i];
  return buffer;
}

function randomFill(buffer, offset, size, callback) {
  if (typeof offset === 'function') { callback = offset; offset = 0; size = buffer.length; }
  else if (typeof size === 'function') { callback = size; size = buffer.length - offset; }
  try { callback(null, randomFillSync(buffer, offset, size)); }
  catch (err) { callback(err); }
}

function createHash(algorithm) {
  const algo = algorithm.toLowerCase().replace('-', '');
  const supported = algo === 'sha256' || algo === 'sha1' || algo === 'md5';
  if (!supported) throw new Error(`Digest method not supported: ${algorithm}`);

  let data = '';
  return {
    update(chunk, encoding) {
      data += typeof chunk === 'string' ? chunk : toBuffer(chunk).toString();
      return this;
    },
    digest(encoding) {
      const hex = internalCrypto.hash(algo, data);
      if (encoding === 'hex' || !encoding) return hex;
      if (encoding === 'base64') return Buffer.from(hex, 'hex').toString('base64');
      return Buffer.from(hex, 'hex');
    },
    copy() { return Object.create(this); },
  };
}

function createHmac(algorithm, key) {
  const algo = algorithm.toLowerCase();
  const keyStr = toBuffer(key).toString();
  let data = '';
  return {
    update(chunk) { data += typeof chunk === 'string' ? chunk : toBuffer(chunk).toString(); return this; },
    digest(encoding) {
      if (typeof internalCrypto.hmac === 'function') {
        const hex = internalCrypto.hmac(algo, keyStr, data);
        if (encoding === 'hex' || !encoding) return hex;
        if (encoding === 'base64') return Buffer.from(hex, 'hex').toString('base64');
        return Buffer.from(hex, 'hex');
      }
      const hash = createHash(algo).update(keyStr + ':' + data).digest('hex');
      return encoding === 'hex' || !encoding ? hash : Buffer.from(hash, 'hex').toString(encoding);
    },
  };
}

function pbkdf2(password, salt, iterations, keylen, digest, callback) {
  if (typeof digest === 'function') { callback = digest; digest = 'sha256'; }
  try {
    const hex = internalCrypto.pbkdf2(String(password), String(salt), Number(iterations), Number(keylen));
    callback(null, Buffer.from(hex, 'hex'));
  } catch (err) { callback(err); }
}

function pbkdf2Sync(password, salt, iterations, keylen, digest) {
  const hex = internalCrypto.pbkdf2(String(password), String(salt), Number(iterations), Number(keylen));
  return Buffer.from(hex, 'hex');
}

function timingSafeEqual(a, b) {
  const bufA = toBuffer(a);
  const bufB = toBuffer(b);
  if (bufA._data.length !== bufB._data.length) return false;
  let result = 0;
  for (let i = 0; i < bufA._data.length; i++) result |= bufA._data[i] ^ bufB._data[i];
  return result === 0;
}

function getHashes() {
  return ['sha1', 'sha256', 'md5'];
}

function getCiphers() { return []; }
function getCurves() { return []; }

const webcrypto = globalThis.crypto;
const subtle = globalThis.crypto?.subtle;

module.exports = { randomBytes, randomUUID, randomFillSync, randomFill, createHash, createHmac, pbkdf2, pbkdf2Sync, timingSafeEqual, getHashes, getCiphers, getCurves, webcrypto, subtle, randomFill: { bind: randomFill }, randomFillSync: { bind: randomFillSync } };