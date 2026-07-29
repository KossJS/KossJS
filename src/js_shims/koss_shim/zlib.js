// Copyright (C) 2026 TT23XR Studio
//
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:zlib — Koss 标准库压缩模块
// gzip, gunzip, inflate, deflate, brotli 压缩/解压缩
// 纯 JS 实现，兼容 Node.js zlib API

var ioModule;
try { ioModule = require('koss:io'); } catch (e) { ioModule = null; }

var EventEmitter;
try { EventEmitter = require('koss:events').EventEmitter; } catch (e) {
  EventEmitter = function() { this._events = {}; };
}

var Buffer;
try { Buffer = require('koss:buffer').Buffer; } catch (e) {
  Buffer = globalThis.Buffer;
}

// ═══════════════════════════════════════════
// 内部工具
// ═══════════════════════════════════════════

function _toBytes(data) {
  if (data instanceof Uint8Array) return data;
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  if (Array.isArray(data)) return new Uint8Array(data);
  if (typeof data === 'string') {
    var bytes = new Uint8Array(data.length);
    for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
    return bytes;
  }
  if (data && data._isBuffer && data._data) return data._data;
  if (data && typeof data.buffer === 'object') {
    return new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength || data.length);
  }
  return new Uint8Array(0);
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

// ═══════════════════════════════════════════
// CRC32
// ═══════════════════════════════════════════

var _crc32Table = (function() {
  var table = new Uint32Array(256);
  for (var n = 0; n < 256; n++) {
    var c = n;
    for (var k = 0; k < 8; k++) {
      if (c & 1) {
        c = 0xEDB88320 ^ (c >>> 1);
      } else {
        c = c >>> 1;
      }
    }
    table[n] = c;
  }
  return table;
})();

function crc32(buf) {
  var bytes = _toBytes(buf);
  var crc = 0xFFFFFFFF;
  for (var i = 0; i < bytes.length; i++) {
    crc = _crc32Table[(crc ^ bytes[i]) & 0xFF] ^ (crc >>> 8);
  }
  return (crc ^ 0xFFFFFFFF) >>> 0;
}

// ═══════════════════════════════════════════
// 位操作
// ═══════════════════════════════════════════

function _createBitReader(data) {
  var bytes = _toBytes(data);
  return {
    _data: bytes,
    _pos: 0,
    _bitBuf: 0,
    _bitCount: 0,
    readBits: function(n) {
      var result = 0;
      for (var i = 0; i < n; i++) {
        if (this._bitCount === 0) {
          if (this._pos >= this._data.length) return result;
          this._bitBuf = this._data[this._pos++];
          this._bitCount = 8;
        }
        result |= (this._bitBuf & 1) << i;
        this._bitBuf >>>= 1;
        this._bitCount--;
      }
      return result;
    },
    readBytes: function(n) {
      var result = new Uint8Array(n);
      for (var i = 0; i < n; i++) {
        result[i] = this.readBits(8);
      }
      return result;
    },
    readBytesAlign: function() {
      if (this._bitCount > 0) {
        this._bitCount = 0;
        this._bitBuf = 0;
      }
    },
    available: function() {
      return (this._data.length - this._pos) * 8 + this._bitCount;
    },
    bytesRead: function() {
      return this._pos - (this._bitCount > 0 ? 1 : 0);
    }
  };
}

function _createBitWriter() {
  return {
    _chunks: [],
    _currentByte: 0,
    _bitCount: 0,
    writeBits: function(value, n) {
      for (var i = 0; i < n; i++) {
        this._currentByte |= ((value >>> i) & 1) << this._bitCount;
        this._bitCount++;
        if (this._bitCount === 8) {
          this._chunks.push(new Uint8Array([this._currentByte]));
          this._currentByte = 0;
          this._bitCount = 0;
        }
      }
    },
    flush: function() {
      if (this._bitCount > 0) {
        this._chunks.push(new Uint8Array([this._currentByte]));
        this._currentByte = 0;
        this._bitCount = 0;
      }
    },
    toBytes: function() {
      this.flush();
      return _concatChunks(this._chunks);
    }
  };
}

// ═══════════════════════════════════════════
// Huffman 编解码
// ═══════════════════════════════════════════

function _buildHuffmanCodes(lengths) {
  var maxLen = 0;
  var blCount = new Uint32Array(16);
  for (var i = 0; i < lengths.length; i++) {
    if (lengths[i] > maxLen) maxLen = lengths[i];
    if (lengths[i] > 0) blCount[lengths[i]]++;
  }
  var nextCode = new Uint32Array(16);
  var code = 0;
  for (var bits = 1; bits <= maxLen; bits++) {
    code = (code + blCount[bits - 1]) << 1;
    nextCode[bits] = code;
  }
  var codes = new Uint32Array(lengths.length);
  for (var j = 0; j < lengths.length; j++) {
    if (lengths[j] > 0) {
      codes[j] = nextCode[lengths[j]]++;
    }
  }
  return { codes: codes, maxLen: maxLen };
}

function _createHuffmanDecoder(lengths) {
  var info = _buildHuffmanCodes(lengths);
  var maxLen = info.maxLen;
  var lookup = [];
  for (var bits = 1; bits <= Math.min(maxLen, 9); bits++) {
    var size = 1 << bits;
    var table = new Int32Array(size);
    table.fill(-1);
    for (var sym = 0; sym < lengths.length; sym++) {
      if (lengths[sym] === bits) {
        var revCode = _reverseBits(info.codes[sym], bits);
        table[revCode] = sym;
      }
    }
    lookup.push(table);
  }
  var bigTable = null;
  if (maxLen > 9) {
    var bigSize = 1 << 9;
    bigTable = new Array(bigSize);
    for (var i = 0; i < bigSize; i++) bigTable[i] = null;
    for (var sym2 = 0; sym2 < lengths.length; sym2++) {
      if (lengths[sym2] > 0 && lengths[sym2] <= 9) {
        var rev = _reverseBits(info.codes[sym2], lengths[sym2]);
        bigTable[rev] = { symbol: sym2, bits: lengths[sym2] };
      }
    }
  }
  return {
    read: function(reader) {
      if (maxLen <= 9) {
        var peeked = reader.readBits(9);
        for (var b = 1; b <= maxLen; b++) {
          var idx = peeked & ((1 << b) - 1);
          if (lookup[b - 1][idx] >= 0) {
            return lookup[b - 1][idx];
          }
        }
      } else {
        var peeked2 = reader.readBits(9);
        var entry = bigTable[peeked2];
        if (entry) {
          if (entry.bits < 9) reader._bitCount += 9 - entry.bits;
          else {
            for (var extra = 9; extra < maxLen; extra++) {
              var bit = reader.readBits(1);
              var testCode = (entry.symbol << 1) | bit;
              for (var s = 0; s < lengths.length; s++) {
                if (lengths[s] === extra + 1 && _reverseBits(info.codes[s], extra + 1) === testCode) {
                  return s;
                }
              }
            }
          }
          return entry.symbol;
        }
        var code = peeked2;
        for (var bLen = 10; bLen <= maxLen; bLen++) {
          code = (code << 1) | reader.readBits(1);
          for (var s2 = 0; s2 < lengths.length; s2++) {
            if (lengths[s2] === bLen) {
              var rev = _reverseBits(info.codes[s2], bLen);
              if (rev === code) return s2;
            }
          }
        }
      }
      return -1;
    }
  };
}

function _reverseBits(val, bits) {
  var result = 0;
  for (var i = 0; i < bits; i++) {
    result = (result << 1) | (val & 1);
    val >>>= 1;
  }
  return result;
}

// ═══════════════════════════════════════════
// Deflate 长度/距离表
// ═══════════════════════════════════════════

var _deflateLengthBase = [
  3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258
];
var _deflateLengthExtra = [
  0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0
];
var _deflateDistBase = [
  1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577
];
var _deflateDistExtra = [
  0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13
];

var _deflateLitLenLengths = [
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,8,8,8,8,8,8,8,8
];

var _deflateDistLengths = [
  5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,
  5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5
];

function _getDeflateLenCode(len) {
  for (var i = 0; i < _deflateLengthBase.length; i++) {
    if (_deflateLengthBase[i] === len) return 257 + i;
  }
  return 258;
}

function _getDeflateLenExtra(len) {
  for (var i = 0; i < _deflateLengthBase.length; i++) {
    if (_deflateLengthBase[i] === len) return _deflateLengthExtra[i];
  }
  return 0;
}

function _getDeflateDistCode(dist) {
  for (var i = 0; i < _deflateDistBase.length; i++) {
    if (_deflateDistBase[i] === dist) return i;
  }
  return 29;
}

function _getDeflateDistExtra(dist) {
  for (var i = 0; i < _deflateDistBase.length; i++) {
    if (_deflateDistBase[i] === dist) return _deflateDistExtra[i];
  }
  return 0;
}

// ═══════════════════════════════════════════
// DEFLATE 解压缩
// ═══════════════════════════════════════════

function _inflateBlock(reader, output) {
  var bfinal = reader.readBits(1);
  var btype = reader.readBits(2);

  if (btype === 0) {
    reader.readBytesAlign();
    var len = reader.readBits(16) | (reader.readBits(16) << 16);
    var nlen = reader.readBits(16) | (reader.readBits(16) << 16);
    var stored = reader.readBytes(len);
    for (var i = 0; i < stored.length; i++) output.push(stored[i]);
    return bfinal;
  }

  if (btype === 1 || btype === 2) {
    var fixedLitLens, fixedDistLens;
    if (btype === 1) {
      fixedLitLens = new Uint8Array(288);
      fixedDistLens = new Uint8Array(32);
      for (var i = 0; i < 144; i++) fixedLitLens[i] = 8;
      for (var i2 = 144; i2 < 256; i2++) fixedLitLens[i2] = 9;
      for (var i3 = 256; i3 < 280; i3++) fixedLitLens[i3] = 7;
      for (var i4 = 280; i4 < 288; i4++) fixedLitLens[i4] = 8;
      for (var i5 = 0; i5 < 32; i5++) fixedDistLens[i5] = 5;
    } else {
      var hlit = reader.readBits(5) + 257;
      var hdist = reader.readBits(5) + 1;
      var hclen = reader.readBits(4) + 4;

      var clenOrder = [16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];
      var clenLens = new Uint8Array(19);
      for (var c = 0; c < hclen; c++) {
        clenLens[clenOrder[c]] = reader.readBits(3);
      }

      var clenDecoder = _createHuffmanDecoder(clenLens);
      var totalLens = hlit + hdist;
      var lensArr = [];
      while (lensArr.length < totalLens) {
        var sym = clenDecoder.read(reader);
        if (sym < 16) {
          lensArr.push(sym);
        } else if (sym === 16) {
          var rep = reader.readBits(2) + 3;
          var last = lensArr[lensArr.length - 1];
          for (var r = 0; r < rep; r++) lensArr.push(last);
        } else if (sym === 17) {
          var rep2 = reader.readBits(3) + 3;
          for (var r2 = 0; r2 < rep2; r2++) lensArr.push(0);
        } else if (sym === 18) {
          var rep3 = reader.readBits(7) + 11;
          for (var r3 = 0; r3 < rep3; r3++) lensArr.push(0);
        }
      }

      fixedLitLens = new Uint8Array(hlit);
      fixedDistLens = new Uint8Array(hdist);
      for (var li = 0; li < hlit; li++) fixedLitLens[li] = lensArr[li];
      for (var di = 0; di < hdist; di++) fixedDistLens[di] = lensArr[hlit + di];
    }

    var litDecoder = _createHuffmanDecoder(fixedLitLens);
    var distDecoder = _createHuffmanDecoder(fixedDistLens);

    while (true) {
      var sym2 = litDecoder.read(reader);
      if (sym2 < 0) break;
      if (sym2 < 256) {
        output.push(sym2);
      } else if (sym2 === 256) {
        break;
      } else {
        var lenIdx = sym2 - 257;
        var length = _deflateLengthBase[lenIdx];
        var lenExtra = _deflateLengthExtra[lenIdx];
        if (lenExtra > 0) length += reader.readBits(lenExtra);

        var distSym = distDecoder.read(reader);
        var dist = _deflateDistBase[distSym];
        var distExtra = _deflateDistExtra[distSym];
        if (distExtra > 0) dist += reader.readBits(distExtra);

        for (var d = 0; d < length; d++) {
          output.push(output[output.length - dist]);
        }
      }
    }
  }
  return bfinal;
}

function _inflateRaw(input) {
  var reader = _createBitReader(input);
  var output = [];
  var bfinal = 0;
  while (!bfinal) {
    bfinal = _inflateBlock(reader, output);
  }
  return new Uint8Array(output);
}

// ═══════════════════════════════════════════
// DEFLATE 压缩
// ═══════════════════════════════════════════

function _findMatch(data, pos, windowSize) {
  var bestLen = 0;
  var bestDist = 0;
  var maxLen = Math.min(258, data.length - pos);
  var start = Math.max(0, pos - windowSize);
  for (var i = start; i < pos; i++) {
    var len = 0;
    while (len < maxLen && data[i + len] === data[pos + len]) {
      len++;
    }
    if (len > bestLen) {
      bestLen = len;
      bestDist = pos - i;
      if (bestLen >= 258) break;
    }
  }
  if (bestLen >= 3) {
    return { len: bestLen, dist: bestDist };
  }
  return null;
}

function _deflateCompress(input, level) {
  var data = _toBytes(input);
  var writer = _createBitWriter();
  var windowSize = 32768;

  writer.writeBits(1, 1);

  var pos = 0;
  while (pos < data.length) {
    var match = _findMatch(data, pos, windowSize);

    if (match) {
      var lenCode = _getDeflateLenCode(match.len);
      var lenExtra = _getDeflateLenExtra(match.len);
      var lenIdx = lenCode - 257;
      writer.writeBits(_deflateLitLenLengths[lenCode], 7);
      writer.writeBits(lenIdx, 7);
      if (lenExtra > 0) writer.writeBits(match.len - _deflateLengthBase[lenIdx], lenExtra);

      var distCode = _getDeflateDistCode(match.dist);
      var distExtra = _getDeflateDistExtra(match.dist);
      writer.writeBits(_deflateDistLengths[distCode], 5);
      writer.writeBits(distCode, 5);
      if (distExtra > 0) writer.writeBits(match.dist - _deflateDistBase[distCode], distExtra);

      pos += match.len;
    } else {
      var litLenCode = data[pos];
      writer.writeBits(_deflateLitLenLengths[litLenCode], 7);
      writer.writeBits(litLenCode, litLenCode < 144 ? 7 : 8);
      pos++;
    }
  }

  writer.writeBits(_deflateLitLenLengths[256], 7);
  writer.writeBits(256, 7);
  writer.flush();

  return writer.toBytes();
}

// ═══════════════════════════════════════════
// gzip 格式处理
// ═══════════════════════════════════════════

var GZIP_MAGIC1 = 0x1f;
var GZIP_MAGIC2 = 0x8b;
var GZIP_DEFLATE = 8;
var GZIP_FHCRC = 0x02;
var GZIP_FEXTRA = 0x04;
var GZIP_FNAME = 0x08;
var GZIP_FCOMMENT = 0x10;

function _gzipHeader(opts) {
  var header = new Uint8Array(10);
  header[0] = GZIP_MAGIC1;
  header[1] = GZIP_MAGIC2;
  header[2] = GZIP_DEFLATE;
  header[3] = 0;

  var now = Math.floor(Date.now() / 1000);
  header[4] = now & 0xff;
  header[5] = (now >>> 8) & 0xff;
  header[6] = (now >>> 16) & 0xff;
  header[7] = (now >>> 24) & 0xff;
  header[8] = 0;
  header[9] = 0xff;
  return header;
}

function _gzipTrailer(crc, size) {
  var trailer = new Uint8Array(8);
  trailer[0] = crc & 0xff;
  trailer[1] = (crc >>> 8) & 0xff;
  trailer[2] = (crc >>> 16) & 0xff;
  trailer[3] = (crc >>> 24) & 0xff;
  trailer[4] = size & 0xff;
  trailer[5] = (size >>> 8) & 0xff;
  trailer[6] = (size >>> 16) & 0xff;
  trailer[7] = (size >>> 24) & 0xff;
  return trailer;
}

function _gunzipInternal(input) {
  var bytes = _toBytes(input);
  if (bytes.length < 10) throw new Error('Invalid gzip data');
  if (bytes[0] !== GZIP_MAGIC1 || bytes[1] !== GZIP_MAGIC2) {
    throw new Error('Not a gzip file');
  }
  var flags = bytes[3];
  var pos = 10;

  if (flags & GZIP_FEXTRA) {
    var xlen = bytes[pos] | (bytes[pos + 1] << 8);
    pos += 2 + xlen;
  }
  if (flags & GZIP_FNAME) {
    while (pos < bytes.length && bytes[pos] !== 0) pos++;
    pos++;
  }
  if (flags & GZIP_FCOMMENT) {
    while (pos < bytes.length && bytes[pos] !== 0) pos++;
    pos++;
  }
  if (flags & GZIP_FHCRC) {
    pos += 2;
  }

  var compressedData = bytes.subarray(pos, bytes.length - 8);
  var trailerStart = bytes.length - 8;
  var expectedCrc = bytes[trailerStart] | (bytes[trailerStart + 1] << 8) |
                    (bytes[trailerStart + 2] << 16) | (bytes[trailerStart + 3] << 24);
  var expectedSize = bytes[trailerStart + 4] | (bytes[trailerStart + 5] << 8) |
                     (bytes[trailerStart + 6] << 16) | (bytes[trailerStart + 7] << 24);

  var decompressed = _inflateRaw(compressedData);

  if (expectedSize > 0 && (decompressed.length >>> 0) !== (expectedSize >>> 0)) {
    throw new Error('gzip size mismatch');
  }
  var actualCrc = crc32(decompressed);
  if (expectedCrc !== actualCrc) {
    throw new Error('gzip CRC mismatch');
  }

  return decompressed;
}

function _gzipInternal(input, opts) {
  var data = _toBytes(input);
  var header = _gzipHeader(opts);
  var compressed = _deflateCompress(data, (opts && opts.level) || 6);
  var computedCrc = crc32(data);
  var size = data.length;
  var trailer = _gzipTrailer(computedCrc, size);

  var result = new Uint8Array(header.length + compressed.length + trailer.length);
  result.set(header, 0);
  result.set(compressed, header.length);
  result.set(trailer, header.length + compressed.length);
  return result;
}

// ═══════════════════════════════════════════
// deflate/inflate 格式（raw deflate without header）
// ═══════════════════════════════════════════

function _deflateInternal(input, opts) {
  var data = _toBytes(input);
  var level = (opts && opts.level !== undefined) ? opts.level : 6;
  if (level <= 0) {
    var writer = _createBitWriter();
    writer.writeBits(1, 1);
    writer.writeBits(0, 2);
    var pos = 0;
    while (pos < data.length) {
      var chunk = Math.min(data.length - pos, 65535);
      var left = data.length - pos - chunk;
      var bfinal = left === 0 ? 1 : 0;
      writer.writeBits(bfinal, 1);
      writer.writeBits(0, 2);
      writer.writeBits(chunk, 16);
      writer.writeBits((~chunk) & 0xffff, 16);
      writer.flush();
      var blockBytes = writer.toBytes();
      writer = _createBitWriter();
      writer.writeBits(bfinal, 1);
      writer.writeBits(0, 2);
      writer.writeBits(chunk, 16);
      writer.writeBits((~chunk) & 0xffff, 16);
      for (var i = 0; i < chunk; i++) {
        writer.writeBits(data[pos + i], 8);
      }
      pos += chunk;
    }
    writer.flush();
    return writer.toBytes();
  }
  return _deflateCompress(data, level);
}

function _inflateInternal(input, opts) {
  return _inflateRaw(input);
}

// ═══════════════════════════════════════════
// Brotli 兼容接口
// ═══════════════════════════════════════════

var _brotliAvailable = false;
try {
  if (globalThis.BrotliEncoder || globalThis.BrotliDecoder) {
    _brotliAvailable = true;
  }
} catch (e) {}

function _brotliCompressInternal(input, opts) {
  if (_brotliAvailable && globalThis.BrotliEncoder) {
    var encoder = new globalThis.BrotliEncoder({ quality: (opts && opts.quality) || 11 });
    var data = _toBytes(input);
    var result = encoder.compress(data);
    encoder.close();
    return result;
  }
  throw new Error('brotli compression not available in this environment');
}

function _brotliDecompressInternal(input, opts) {
  if (_brotliAvailable && globalThis.BrotliDecoder) {
    var decoder = new globalThis.BrotliDecoder();
    var data = _toBytes(input);
    var result = decoder.decompress(data);
    decoder.close();
    return result;
  }
  throw new Error('brotli decompression not available in this environment');
}

// ═══════════════════════════════════════════
// 流式转换器基类
// ═══════════════════════════════════════════

function _createTransform(fn) {
  var stream;
  try {
    stream = require('koss:internal/stream');
  } catch (e) {
    stream = null;
  }

  var instance;
  if (stream && stream.Transform) {
    instance = new stream.Transform();
    instance._transform = function(chunk, encoding, callback) {
      try {
        var result = fn(chunk, this._opts);
        if (result) this.push(result);
        callback();
      } catch (e) {
        callback(e);
      }
    };
  } else {
    var chunks = [];
    var finished = false;
    instance = {
      _opts: {},
      _chunks: chunks,
      _finished: finished,
      write: function(chunk) {
        chunks.push(_toBytes(chunk));
        return true;
      },
      end: function(chunk) {
        if (chunk) chunks.push(_toBytes(chunk));
        finished = true;
        try {
          var combined = _concatChunks(chunks);
          var result = fn(combined, instance._opts);
          if (instance._flushCallback) instance._flushCallback(result);
        } catch (e) {
          if (instance._errorCallback) instance._errorCallback(e);
        }
      },
      on: function(event, handler) {
        if (event === 'finish') instance._flushCallback = handler;
        if (event === 'error') instance._errorCallback = handler;
        return instance;
      },
      once: function(event, handler) { return instance.on(event, handler); },
      emit: function() { return instance; },
      pipe: function(dest) {
        instance._flushCallback = function(data) { dest.write(data); };
        return dest;
      },
    };
  }
  instance._opts = {};
  return instance;
}

// ═══════════════════════════════════════════
// 同步 API
// ═══════════════════════════════════════════

function gzipSync(input, opts) {
  return _gzipInternal(input, opts);
}

function gunzipSync(input, opts) {
  return _gunzipInternal(input, opts);
}

function deflateSync(input, opts) {
  return _deflateInternal(input, opts);
}

function inflateSync(input, opts) {
  return _inflateInternal(input, opts);
}

function brotliCompressSync(input, opts) {
  return _brotliCompressInternal(input, opts);
}

function brotliDecompressSync(input, opts) {
  return _brotliDecompressInternal(input, opts);
}

// ═══════════════════════════════════════════
// 异步 API（回调）
// ═══════════════════════════════════════════

function gzip(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = gzipSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function gunzip(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = gunzipSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function deflate(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = deflateSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function inflate(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = inflateSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function brotliCompress(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = brotliCompressSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function brotliDecompress(input, opts, callback) {
  if (typeof opts === 'function') { callback = opts; opts = {}; }
  try {
    var result = brotliDecompressSync(input, opts);
    if (callback) callback(null, result);
    return result;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

// ═══════════════════════════════════════════
// 流式 API
// ═══════════════════════════════════════════

function createGzip(opts) {
  var s = _createTransform(function(chunk) { return gzipSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

function createGunzip(opts) {
  var s = _createTransform(function(chunk) { return gunzipSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

function createDeflate(opts) {
  var s = _createTransform(function(chunk) { return deflateSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

function createInflate(opts) {
  var s = _createTransform(function(chunk) { return inflateSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

function createBrotliCompress(opts) {
  var s = _createTransform(function(chunk) { return brotliCompressSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

function createBrotliDecompress(opts) {
  var s = _createTransform(function(chunk) { return brotliDecompressSync(chunk, opts); });
  s._opts = opts || {};
  return s;
}

// ═══════════════════════════════════════════
// 常量
// ═══════════════════════════════════════════

var constants = {
  Z_NO_FLUSH: 0,
  Z_PARTIAL_FLUSH: 1,
  Z_SYNC_FLUSH: 2,
  Z_FULL_FLUSH: 3,
  Z_FINISH: 4,
  Z_OK: 0,
  Z_STREAM_END: 1,
  Z_NEED_DICT: 2,
  Z_ERRNO: -1,
  Z_STREAM_ERROR: -2,
  Z_DATA_ERROR: -3,
  Z_MEM_ERROR: -4,
  Z_BUF_ERROR: -5,
  Z_NO_COMPRESSION: 0,
  Z_BEST_SPEED: 1,
  Z_BEST_COMPRESSION: 9,
  Z_DEFAULT_COMPRESSION: -1,
  Z_FILTERED: 1,
  Z_HUFFMAN_ONLY: 2,
  Z_RLE: 3,
  Z_FIXED: 4,
  Z_DEFAULT_STRATEGY: 0,
  DEFLATE: 1,
  INFLATE: 2,
  GZIP: 31,
  GUNZIP: 47,
};

// ═══════════════════════════════════════════
// 模块导出
// ═══════════════════════════════════════════

module.exports = {
  gzip: gzip,
  gunzip: gunzip,
  deflate: deflate,
  inflate: inflate,
  brotliCompress: brotliCompress,
  brotliDecompress: brotliDecompress,

  gzipSync: gzipSync,
  gunzipSync: gunzipSync,
  deflateSync: deflateSync,
  inflateSync: inflateSync,
  brotliCompressSync: brotliCompressSync,
  brotliDecompressSync: brotliDecompressSync,

  createGzip: createGzip,
  createGunzip: createGunzip,
  createDeflate: createDeflate,
  createInflate: createInflate,
  createBrotliCompress: createBrotliCompress,
  createBrotliDecompress: createBrotliDecompress,

  constants: constants,

  crc32: crc32,
};
