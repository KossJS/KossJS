// koss:node/zlib - Node.js zlib module (L3)

const { __koss_gzip, __koss_gunzip, __koss_deflate, __koss_inflate } = globalThis;
const { Buffer } = require('koss:node/buffer');

function toBytes(data) {
  if (Buffer.isBuffer(data)) return Array.from(data._data);
  if (typeof data === 'string') return Array.from(Buffer.from(data)._data);
  if (data instanceof Uint8Array) return Array.from(data);
  return Array.from(Buffer.from(String(data))._data);
}

function fromBytes(data) {
  if (typeof data === 'string') {
    try {
      var arr = JSON.parse(data);
      var buf = Buffer.alloc(arr.length);
      for (var i = 0; i < arr.length; i++) buf._data[i] = arr[i];
      return buf;
    }
    catch { return Buffer.from(data, 'utf8'); }
  }
  if (data instanceof Uint8Array) {
    var buf2 = Buffer.alloc(data.length);
    buf2._data.set(data);
    return buf2;
  }
  return data;
}

function compress(data, fn) {
  if (typeof fn !== 'function') throw new Error('Compression not available');
  const input = JSON.stringify(toBytes(data));
  const result = fn(input);
  return fromBytes(result);
}

function gzipSync(data, options) { return compress(data, __koss_gzip); }
function gunzipSync(data, options) { return compress(data, __koss_gunzip); }
function deflateSync(data, options) { return compress(data, __koss_deflate); }
function inflateSync(data, options) { return compress(data, __koss_inflate); }

function gzip(data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { var result = gzipSync(data, options); if (callback) process.nextTick(() => callback(null, result)); }
  catch (err) { if (callback) process.nextTick(() => callback(err)); }
}

function gunzip(data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { var result = gunzipSync(data, options); if (callback) process.nextTick(() => callback(null, result)); }
  catch (err) { if (callback) process.nextTick(() => callback(err)); }
}

function deflate(data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { var result = deflateSync(data, options); if (callback) process.nextTick(() => callback(null, result)); }
  catch (err) { if (callback) process.nextTick(() => callback(err)); }
}

function inflate(data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { var result = inflateSync(data, options); if (callback) process.nextTick(() => callback(null, result)); }
  catch (err) { if (callback) process.nextTick(() => callback(err)); }
}

// Promises
function gzipAsync(data, options) {
  return new Promise((resolve, reject) => gzip(data, options, (err, res) => err ? reject(err) : resolve(res)));
}
function gunzipAsync(data, options) {
  return new Promise((resolve, reject) => gunzip(data, options, (err, res) => err ? reject(err) : resolve(res)));
}
function deflateAsync(data, options) {
  return new Promise((resolve, reject) => deflate(data, options, (err, res) => err ? reject(err) : resolve(res)));
}
function inflateAsync(data, options) {
  return new Promise((resolve, reject) => inflate(data, options, (err, res) => err ? reject(err) : resolve(res)));
}

const constants = {
  Z_OK: 0, Z_STREAM_END: 1, Z_NEED_DICT: 2, Z_ERRNO: -1, Z_STREAM_ERROR: -2,
  Z_DATA_ERROR: -3, Z_MEM_ERROR: -4, Z_BUF_ERROR: -5, Z_VERSION_ERROR: -6,
  Z_NO_COMPRESSION: 0, Z_BEST_SPEED: 1, Z_BEST_COMPRESSION: 9, Z_DEFAULT_COMPRESSION: -1,
  Z_FILTERED: 1, Z_HUFFMAN_ONLY: 2, Z_RLE: 3, Z_FIXED: 4, Z_DEFAULT_STRATEGY: 0,
  Z_BINARY: 0, Z_TEXT: 1, Z_ASCII: 1, Z_UNKNOWN: 2,
  Z_DEFLATED: 8,
  Z_NULL: 0,
};

module.exports = { gzipSync, gunzipSync, deflateSync, inflateSync, gzip, gunzip, deflate, inflate, gzipAsync, gunzipAsync, deflateAsync, inflateAsync, constants };