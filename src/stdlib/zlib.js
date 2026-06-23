'use strict';

var Buffer = require('buffer').Buffer;

function _toBytes(data) {
  if (Buffer.isBuffer(data)) {
    return '[' + Array.prototype.join.call(data, ',') + ']';
  }
  if (typeof data === 'string') {
    var b = Buffer.from(data);
    return '[' + Array.prototype.join.call(b, ',') + ']';
  }
  if (Array.isArray(data)) {
    return JSON.stringify(data);
  }
  throw new Error('Expected Buffer, string, or Array');
}

function _fromBytes(jsonStr) {
  var arr = JSON.parse(jsonStr);
  return Buffer.from(arr);
}

function _runSync(fn, data, callback) {
  try {
    if (callback) {
      var result = fn(data);
      process.nextTick(function() { callback(null, result); });
    }
    return fn(data);
  } catch (e) {
    if (callback) process.nextTick(function() { callback(e); });
    throw e;
  }
}

function gzip(buffer, callback) {
  return _runSync(function(buf) {
    var input = _toBytes(buf);
    return _fromBytes(__koss_gzip(input));
  }, buffer, callback);
}

function gunzip(buffer, callback) {
  return _runSync(function(buf) {
    var input = _toBytes(buf);
    return _fromBytes(__koss_gunzip(input));
  }, buffer, callback);
}

function deflate(buffer, callback) {
  return _runSync(function(buf) {
    var input = _toBytes(buf);
    return _fromBytes(__koss_deflate(input));
  }, buffer, callback);
}

function inflate(buffer, callback) {
  return _runSync(function(buf) {
    var input = _toBytes(buf);
    return _fromBytes(__koss_inflate(input));
  }, buffer, callback);
}

function gzipSync(buffer) { return gzip(buffer); }
function gunzipSync(buffer) { return gunzip(buffer); }
function deflateSync(buffer) { return deflate(buffer); }
function inflateSync(buffer) { return inflate(buffer); }

module.exports = {
  gzip: gzip,
  gzipSync: gzipSync,
  gunzip: gunzip,
  gunzipSync: gunzipSync,
  deflate: deflate,
  deflateSync: deflateSync,
  inflate: inflate,
  inflateSync: inflateSync,
  constants: {
    Z_OK: 0, Z_STREAM_END: 1, Z_NEED_DICT: 2, Z_ERRNO: -1,
    Z_STREAM_ERROR: -2, Z_DATA_ERROR: -3, Z_MEM_ERROR: -4,
    Z_BUF_ERROR: -5, Z_VERSION_ERROR: -6,
    Z_NO_FLUSH: 0, Z_PARTIAL_FLUSH: 1, Z_SYNC_FLUSH: 2,
    Z_FULL_FLUSH: 3, Z_FINISH: 4, Z_BLOCK: 5,
    Z_NO_COMPRESSION: 0, Z_BEST_SPEED: 1, Z_BEST_COMPRESSION: 9,
    Z_DEFAULT_COMPRESSION: -1, Z_FILTERED: 1, Z_HUFFMAN_ONLY: 2,
    Z_RLE: 3, Z_FIXED: 4, Z_DEFAULT_STRATEGY: 0,
    ZLIB_VERNUM: 0x1280,
    DEFLATE: 1, INFLATE: 2, GZIP: 3, GUNZIP: 4, DEFLATERAW: 5,
    INFLATERAW: 6, UNZIP: 7, BROTLI_DECODE: 8, BROTLI_ENCODE: 9,
    Z_MIN_CHUNK: 64, Z_MAX_CHUNK: Infinity, Z_DEFAULT_CHUNK: 16384,
    Z_MIN_MEMLEVEL: 1, Z_MAX_MEMLEVEL: 9, Z_DEFAULT_MEMLEVEL: 8,
    Z_MIN_LEVEL: -1, Z_MAX_LEVEL: 9, Z_DEFAULT_LEVEL: -1,
    Z_MIN_WINDOWBITS: 8, Z_MAX_WINDOWBITS: 15, Z_DEFAULT_WINDOWBITS: 15,
  },
};
