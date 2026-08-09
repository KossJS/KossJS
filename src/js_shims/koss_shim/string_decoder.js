// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:string_decoder — Koss 标准库字符串解码器模块
// Node.js StringDecoder 兼容实现，纯 JS，无外部依赖

// ─── Encoding Helpers ───

var _b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
var _b64lookup = new Uint8Array(256);
var i;
for (i = 0; i < _b64chars.length; i++) _b64lookup[_b64chars.charCodeAt(i)] = i;

function _normalizeEncoding(enc) {
  if (!enc) return 'utf8';
  var e = String(enc).toLowerCase().replace(/[-_\s]/g, '');
  if (e === 'utf8' || e === 'utf-8') return 'utf8';
  if (e === 'ascii' || e === 'usascii' || e === 'us-ascii') return 'ascii';
  if (e === 'binary' || e === 'latin1' || e === 'iso88591' || e === 'iso-8859-1') return 'latin1';
  if (e === 'base64') return 'base64';
  if (e === 'base64url') return 'base64url';
  if (e === 'hex') return 'hex';
  if (e === 'utf16le' || e === 'utf-16le' || e === 'ucs2' || e === 'ucs-2') return 'utf16le';
  return null;
}

function _utf8Encode(str) {
  var bytes = [];
  for (var i = 0; i < str.length; i++) {
    var cc = str.charCodeAt(i);
    if (cc < 0x80) {
      bytes.push(cc);
    } else if (cc < 0x800) {
      bytes.push(0xc0 | (cc >> 6));
      bytes.push(0x80 | (cc & 0x3f));
    } else if (cc >= 0xd800 && cc <= 0xdbff) {
      i++;
      var cc2 = str.charCodeAt(i) || 0;
      var codePoint = ((cc - 0xd800) << 10) + (cc2 - 0xdc00) + 0x10000;
      bytes.push(0xf0 | (codePoint >> 18));
      bytes.push(0x80 | ((codePoint >> 12) & 0x3f));
      bytes.push(0x80 | ((codePoint >> 6) & 0x3f));
      bytes.push(0x80 | (codePoint & 0x3f));
    } else {
      bytes.push(0xe0 | (cc >> 12));
      bytes.push(0x80 | ((cc >> 6) & 0x3f));
      bytes.push(0x80 | (cc & 0x3f));
    }
  }
  return new Uint8Array(bytes);
}

function _utf8Decode(bytes) {
  var chars = [];
  var i = 0;
  while (i < bytes.length) {
    var b = bytes[i++];
    if (b < 0x80) {
      chars.push(b);
    } else if (b < 0xe0) {
      chars.push(((b & 0x1f) << 6) | (bytes[i++] & 0x3f));
    } else if (b < 0xf0) {
      var b2 = bytes[i++] & 0x3f;
      var b3 = bytes[i++] & 0x3f;
      chars.push(((b & 0x0f) << 12) | (b2 << 6) | b3);
    } else if (b < 0xf8) {
      var b2_ = bytes[i++] & 0x3f;
      var b3_ = bytes[i++] & 0x3f;
      var b4 = bytes[i++] & 0x3f;
      var codePoint = ((b & 0x07) << 18) | (b2_ << 12) | (b3_ << 6) | b4;
      if (codePoint > 0xffff) {
        codePoint -= 0x10000;
        chars.push(0xd800 + (codePoint >> 10));
        chars.push(0xdc00 + (codePoint & 0x3ff));
      } else {
        chars.push(codePoint);
      }
    }
  }
  return String.fromCharCode.apply(null, chars);
}

function _utf16leDecode(bytes) {
  var chars = [];
  for (var i = 0; i < bytes.length - 1; i += 2) {
    chars.push(String.fromCharCode(bytes[i] | (bytes[i + 1] << 8)));
  }
  return chars.join('');
}

function _latin1Decode(bytes) {
  var chars = [];
  for (var i = 0; i < bytes.length; i++) {
    chars.push(String.fromCharCode(bytes[i]));
  }
  return chars.join('');
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

function _toBase64(bytes) {
  var result = '';
  for (var i = 0; i < bytes.length; i += 3) {
    var a = bytes[i];
    var b = i + 1 < bytes.length ? bytes[i + 1] : 0;
    var c = i + 2 < bytes.length ? bytes[i + 2] : 0;
    var triple = (a << 16) | (b << 8) | c;
    result += _b64chars[(triple >> 18) & 0x3F];
    result += _b64chars[(triple >> 12) & 0x3F];
    result += i + 1 < bytes.length ? _b64chars[(triple >> 6) & 0x3F] : '=';
    result += i + 2 < bytes.length ? _b64chars[triple & 0x3F] : '=';
  }
  return result;
}

function _fromHex(hex) {
  var str = String(hex).replace(/[^0-9a-fA-F]/g, '');
  var bytes = new Uint8Array(Math.floor(str.length / 2));
  for (var i = 0; i < str.length; i += 2) {
    bytes[i / 2] = parseInt(str.substring(i, i + 2), 16);
  }
  return bytes;
}

function _fromBase64(b64) {
  var str = String(b64).replace(/[^A-Za-z0-9+/=]/g, '');
  var bytes = new Uint8Array(Math.floor(str.length * 3 / 4));
  var idx = 0;
  for (var i = 0; i < str.length; i += 4) {
    var a = _b64lookup[str.charCodeAt(i)] || 0;
    var b = _b64lookup[str.charCodeAt(i + 1)] || 0;
    var c = _b64lookup[str.charCodeAt(i + 2)] || 0;
    var d = _b64lookup[str.charCodeAt(i + 3)] || 0;
    var triple = (a << 18) | (b << 12) | (c << 6) | d;
    bytes[idx++] = (triple >> 16) & 0xFF;
    if (str[i + 2] !== '=') bytes[idx++] = (triple >> 8) & 0xFF;
    if (str[i + 3] !== '=') bytes[idx++] = triple & 0xFF;
  }
  return bytes.slice(0, idx);
}

function _fromBase64Url(b64url) {
  return _fromBase64(b64url.replace(/-/g, '+').replace(/_/g, '/'));
}

// ═══════════════════════════════════════════
// StringDecoder
// ═══════════════════════════════════════════

function StringDecoder(encoding) {
  this.encoding = _normalizeEncoding(encoding) || 'utf8';
  this._lastNeed = 0;
  this._lastTotal = 0;
  this._buffer = '';
}

StringDecoder.prototype.write = function(buffer) {
  if (buffer.length === 0) return '';

  var buf;
  if (typeof buffer === 'string') {
    buf = _utf8Encode(buffer);
  } else if (buffer instanceof Uint8Array) {
    buf = buffer;
  } else if (buffer && buffer.buffer) {
    buf = new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  } else {
    buf = new Uint8Array(0);
  }

  var enc = this.encoding;

  if (enc === 'utf8' || enc === 'utf-8') {
    return this._writeUtf8(buf);
  }
  if (enc === 'ascii' || enc === 'latin1' || enc === 'binary') {
    return this._writeAscii(buf);
  }
  if (enc === 'utf16le' || enc === 'utf-16le' || enc === 'ucs2' || enc === 'ucs-2') {
    return this._writeUtf16le(buf);
  }
  if (enc === 'base64') {
    return this._writeBase64(buf);
  }
  if (enc === 'base64url') {
    return this._writeBase64Url(buf);
  }
  if (enc === 'hex') {
    return this._writeHex(buf);
  }

  return this._writeUtf8(buf);
};

StringDecoder.prototype.end = function(buffer) {
  var result = buffer ? this.write(buffer) : '';
  if (this._bufferBytes && this._bufferBytes.length > 0) {
    if (this.encoding === 'utf8' || this.encoding === 'utf-8') {
      result += _utf8Decode(this._bufferBytes);
    } else if (this.encoding === 'utf16le' || this.encoding === 'utf-16le' || this.encoding === 'ucs2' || this.encoding === 'ucs-2') {
      result += _utf16leDecode(this._bufferBytes);
    } else {
      result += _latin1Decode(this._bufferBytes);
    }
  }
  this._buffer = '';
  this._bufferBytes = null;
  this._lastNeed = 0;
  this._lastTotal = 0;
  return result;
};

StringDecoder.prototype.text = function(buffer, offset, length) {
  if (offset || length) {
    var uint8 = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    var slice = uint8.slice(offset || 0, length !== undefined ? offset + length : undefined);
    return this._decodeSingle(slice);
  }
  return this.write(buffer);
};

StringDecoder.prototype.fillLast = function(buffer) {
  return this.end(buffer);
};

// ─── Encoding-specific writers ───

StringDecoder.prototype._writeUtf8 = function(buf) {
  var result = '';

  if (this._lastNeed > 0 && this._bufferBytes) {
    var buffered = this._bufferBytes.length;
    // 从新 buf 取需要的字节补齐到完整序列
    var take = Math.min(this._lastNeed, buf.length);
    var partial = new Uint8Array(buffered + take);
    partial.set(this._bufferBytes, 0);
    partial.set(buf.subarray(0, take), buffered);
    this._lastNeed -= take;
    if (this._lastNeed > 0) {
      // 仍不完整，继续保存
      var newBuf = new Uint8Array(buffered + take);
      newBuf.set(this._bufferBytes, 0);
      newBuf.set(buf.subarray(0, take), buffered);
      this._bufferBytes = newBuf;
    } else {
      result = _utf8Decode(partial);
      this._bufferBytes = null;
    }
    buf = buf.subarray(take);
  }

  if (buf.length > 0) {
    // 处理结尾不完整的 UTF-8 序列：从末尾向前找到第一个非续字节。
    // 只有"起始字节后缺少续字节"的情况需要缓冲。
    var split = buf.length;
    var i = buf.length - 1;
    // 跳过末尾的续字节
    while (i >= 0 && (buf[i] & 0xc0) === 0x80) i--;
    if (i >= 0) {
      var b0 = buf[i];
      var need;
      if (b0 >= 0xf0) need = 4;
      else if (b0 >= 0xe0) need = 3;
      else if (b0 >= 0xc0) need = 2;
      else need = 1;
      // 从 i 到末尾的字节数
      var consumed = buf.length - i;
      if (consumed < need) {
        // 起始字节已出现但续字节不足 → 整个序列缓冲
        split = i;
      }
    }
    var complete = buf.subarray(0, split);
    if (complete.length > 0) {
      result += _utf8Decode(complete);
    }
    var remaining = buf.subarray(split);
    if (remaining.length > 0) {
      // 从第一个起始字节算起的完整序列所需字节数
      var first = remaining[0];
      var totalNeed;
      if (first >= 0xf0) totalNeed = 4;
      else if (first >= 0xe0) totalNeed = 3;
      else if (first >= 0xc0) totalNeed = 2;
      else totalNeed = 1;
      this._lastNeed = totalNeed - remaining.length;
      this._bufferBytes = new Uint8Array(remaining);
    } else {
      this._bufferBytes = null;
    }
  }

  return result;
};

StringDecoder.prototype._writeAscii = function(buf) {
  var result = '';
  for (var i = 0; i < buf.length; i++) {
    result += String.fromCharCode(buf[i] & 0x7f);
  }
  return result;
};

StringDecoder.prototype._writeUtf16le = function(buf) {
  var total = this._lastTotal + buf.length;
  var result = '';

  if (this._lastNeed > 0) {
    var needed = this._lastNeed;
    if (needed > buf.length) needed = buf.length;
    var partial = new Uint8Array(this._lastTotal + needed);
    partial.set(this._bufferBytes, 0);
    partial.set(buf.subarray(0, needed), this._lastTotal);
    result = _utf16leDecode(partial);
    this._lastNeed -= needed;
    this._lastTotal = this._lastNeed;
    if (this._lastNeed > 0) {
      this._bufferBytes = new Uint8Array(result.length * 2);
      for (var j = 0; j < result.length; j++) {
        var cc = result.charCodeAt(j);
        this._bufferBytes[j * 2] = cc & 0xff;
        this._bufferBytes[j * 2 + 1] = (cc >> 8) & 0xff;
      }
    } else {
      this._bufferBytes = null;
    }
    buf = buf.subarray(needed);
  }

  var remaining = buf.length % 2;
  var decodeLen = buf.length - remaining;
  if (decodeLen > 0) {
    result += _utf16leDecode(buf.subarray(0, decodeLen));
  }

  if (remaining > 0) {
    this._lastNeed = 1;
    this._lastTotal = 1;
    this._bufferBytes = buf.subarray(decodeLen, decodeLen + 1);
  } else if (this._lastNeed > 0 && remaining === 0) {
    this._lastNeed = 0;
    this._lastTotal = 0;
    this._bufferBytes = null;
  }

  return result;
};

StringDecoder.prototype._writeBase64 = function(buf) {
  var result = this._buffer + _toBase64(buf);
  var chunkSize = Math.floor(result.length / 4) * 4;
  if (chunkSize > 0) {
    var decoded = _fromBase64(result.substring(0, chunkSize));
    this._buffer = result.substring(chunkSize);
    return _latin1Decode(decoded);
  }
  this._buffer = result;
  return '';
};

StringDecoder.prototype._writeBase64Url = function(buf) {
  var result = this._buffer + _toBase64(buf).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  var chunkSize = Math.floor(result.length / 4) * 4;
  if (chunkSize > 0) {
    var b64 = result.substring(0, chunkSize).replace(/-/g, '+').replace(/_/g, '/');
    var pad = b64.length % 4;
    if (pad === 2) b64 += '==';
    else if (pad === 3) b64 += '=';
    var decoded = _fromBase64(b64);
    this._buffer = result.substring(chunkSize);
    return _latin1Decode(decoded);
  }
  this._buffer = result;
  return '';
};

StringDecoder.prototype._writeHex = function(buf) {
  var result = this._buffer + _toHex(buf);
  var chunkSize = Math.floor(result.length / 2) * 2;
  if (chunkSize > 0) {
    var decoded = _fromHex(result.substring(0, chunkSize));
    this._buffer = result.substring(chunkSize);
    return _latin1Decode(decoded);
  }
  this._buffer = result;
  return '';
};

StringDecoder.prototype._decodeSingle = function(buf) {
  var enc = this.encoding;
  if (enc === 'utf8' || enc === 'utf-8') return _utf8Decode(buf);
  if (enc === 'ascii' || enc === 'latin1' || enc === 'binary') return _latin1Decode(buf);
  if (enc === 'utf16le' || enc === 'utf-16le' || enc === 'ucs2' || enc === 'ucs-2') return _utf16leDecode(buf);
  if (enc === 'base64') return _latin1Decode(_fromBase64(_toBase64(buf)));
  if (enc === 'base64url') return _latin1Decode(_fromBase64Url(_toBase64(buf).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')));
  if (enc === 'hex') return _latin1Decode(_fromHex(_toHex(buf)));
  return _utf8Decode(buf);
};

module.exports = { StringDecoder: StringDecoder };
