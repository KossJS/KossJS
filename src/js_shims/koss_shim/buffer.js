// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:buffer — Koss 标准库 Buffer 模块
// Node.js Buffer 兼容实现，纯 JS，无外部依赖

var _b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
var _b64lookup = new Uint8Array(256);
for (var i = 0; i < _b64chars.length; i++) _b64lookup[_b64chars.charCodeAt(i)] = i;

var _hexchars = '0123456789abcdef';
var _hexlookup = new Uint8Array(256);
for (var i = 0; i < 256; i++) _hexlookup[i] = _hexchars.charCodeAt(i & 0x0f);

var kMaxLength = 0x7fffffff;

var TEXT_DECODER_INSTANCES = {};

var constants = {
  MAX_LENGTH: kMaxLength,
  MAX_STRING_LENGTH: kMaxLength >>> 1,
  UTF8: 0,
  UTF16LE: 1,
};

// ─── Blob ───
function Blob(parts, options) {
  var opts = options || {};
  this._type = opts.type || '';
  this._parts = [];
  if (parts) {
    for (var i = 0; i < parts.length; i++) {
      var p = parts[i];
      if (p instanceof Uint8Array) {
        this._parts.push(p);
      } else if (typeof p === 'string') {
        this._parts.push(_utf8Encode(p));
      } else if (p && p.buffer) {
        var view = new Uint8Array(p.buffer, p.byteOffset, p.byteLength);
        this._parts.push(new Uint8Array(view));
      }
    }
  }
  this.size = 0;
  for (var j = 0; j < this._parts.length; j++) this.size += this._parts[j].length;
}

Blob.prototype.arrayBuffer = function() {
  var total = this.size;
  var result = new Uint8Array(total);
  var offset = 0;
  for (var i = 0; i < this._parts.length; i++) {
    result.set(this._parts[i], offset);
    offset += this._parts[i].length;
  }
  return result.buffer;
};

Blob.prototype.slice = function(start, end, type) {
  var s = start || 0;
  var e = end !== undefined ? end : this.size;
  if (s < 0) s = Math.max(0, this.size + s);
  if (e < 0) e = Math.max(0, this.size + e);
  var len = e - s;
  var parts = [];
  var offset = 0;
  for (var i = 0; i < this._parts.length; i++) {
    var p = this._parts[i];
    if (s < offset + p.length && e > offset) {
      var ps = Math.max(0, s - offset);
      var pe = Math.min(p.length, e - offset);
      parts.push(p.subarray(ps, pe));
    }
    offset += p.length;
  }
  return new Blob(parts, { type: type || this._type });
};

Blob.prototype.text = function() {
  var buf = new NodeBuffer(this.size);
  var offset = 0;
  for (var i = 0; i < this._parts.length; i++) {
    buf.set(this._parts[i], offset);
    offset += this._parts[i].length;
  }
  return buf.toString('utf8');
};

Blob.prototype.stream = function() {
  throw new Error('Blob.stream is not supported in this environment');
};

Object.defineProperty(Blob.prototype, 'type', {
  get: function() { return this._type; }
});

// ─── NodeBuffer ───
function NodeBuffer(value, encodingOrOffset, length) {
  if (typeof value === 'number') {
    this._data = new Uint8Array(value);
    this._length = value;
  } else if (typeof value === 'string') {
    var enc = encodingOrOffset || 'utf8';
    var bytes = _encodeString(value, enc);
    this._data = bytes;
    this._length = bytes.length;
  } else if (value instanceof Uint8Array || value instanceof ArrayBuffer || value instanceof NodeBuffer) {
    var src;
    if (value instanceof NodeBuffer) {
      src = value._data;
    } else if (value instanceof ArrayBuffer) {
      src = new Uint8Array(value);
    } else {
      src = value;
    }
    var offset = encodingOrOffset || 0;
    var len = length !== undefined ? length : src.length - offset;
    this._data = new Uint8Array(len);
    this._data.set(src.subarray(offset, offset + len));
    this._length = len;
  } else if (value && value._isBuffer) {
    this._data = new Uint8Array(value.length);
    this._data.set(value._data);
    this._length = value.length;
  } else {
    this._data = new Uint8Array(0);
    this._length = 0;
  }
}

NodeBuffer.prototype._isBuffer = true;

Object.defineProperty(NodeBuffer.prototype, 'length', {
  get: function() { return this._length; }
});

Object.defineProperty(NodeBuffer.prototype, 'byteLength', {
  get: function() { return this._length; }
});

Object.defineProperty(NodeBuffer.prototype, 'byteOffset', {
  get: function() { return 0; }
});

Object.defineProperty(NodeBuffer.prototype, 'buffer', {
  get: function() { return this._data.buffer; }
});

// ─── Static Methods ───
NodeBuffer.isBuffer = function(obj) {
  return !!(obj && obj._isBuffer);
};

NodeBuffer.isEncoding = function(encoding) {
  return _normalizeEncoding(encoding) !== null;
};

NodeBuffer.byteLength = function(str, encoding) {
  if (typeof str === 'number') return str;
  if (str instanceof NodeBuffer) return str._length;
  if (str instanceof Uint8Array) return str.length;
  var enc = encoding || 'utf8';
  if (enc === 'base64') {
    var s = String(str).replace(/[^A-Za-z0-9+/=]/g, '');
    return Math.floor(s.length * 3 / 4);
  }
  if (enc === 'hex') {
    return Math.floor(String(str).length / 2);
  }
  return _utf8Encode(String(str)).length;
};

NodeBuffer.concat = function(list, totalLength) {
  if (!Array.isArray(list) || list.length === 0) return new NodeBuffer(0);
  if (totalLength === undefined) {
    totalLength = 0;
    for (var i = 0; i < list.length; i++) {
      var item = list[i];
      if (item instanceof NodeBuffer) totalLength += item._length;
      else if (item instanceof Uint8Array) totalLength += item.length;
      else totalLength += String(item).length;
    }
  }
  var result = new NodeBuffer(totalLength);
  var offset = 0;
  for (var j = 0; j < list.length; j++) {
    var item2 = list[j];
    var bytes;
    if (item2 instanceof NodeBuffer) {
      bytes = item2._data;
    } else if (item2 instanceof Uint8Array) {
      bytes = item2;
    } else if (typeof item2 === 'string') {
      bytes = _utf8Encode(item2);
    } else {
      bytes = new Uint8Array(0);
    }
    result._data.set(bytes.subarray(0, Math.min(bytes.length, totalLength - offset)), offset);
    offset += bytes.length;
    if (offset >= totalLength) break;
  }
  result._length = offset;
  return result;
};

NodeBuffer.from = function(value, encodingOrOffset, length) {
  if (typeof value === 'string') {
    return new NodeBuffer(value, encodingOrOffset, length);
  }
  if (value instanceof NodeBuffer) {
    return new NodeBuffer(value, encodingOrOffset, length);
  }
  if (value instanceof Uint8Array || value instanceof ArrayBuffer) {
    return new NodeBuffer(value, encodingOrOffset, length);
  }
  if (value && typeof value === 'object' && value[Symbol && Symbol.toPrimitive]) {
    return new NodeBuffer(String(value), encodingOrOffset, length);
  }
  return new NodeBuffer(value, encodingOrOffset, length);
};

NodeBuffer.alloc = function(size, fill, encoding) {
  var buf = new NodeBuffer(size);
  if (fill !== undefined && fill !== null) {
    if (typeof fill === 'string') {
      var fillBytes = _encodeString(fill, encoding || 'utf8');
      for (var i = 0; i < size; i++) {
        buf._data[i] = fillBytes[i % fillBytes.length];
      }
    } else if (typeof fill === 'number') {
      buf._data.fill(fill & 0xff);
    } else if (fill instanceof Uint8Array) {
      for (var i = 0; i < size; i++) {
        buf._data[i] = fill[i % fill.length];
      }
    }
  } else {
    buf._data.fill(0);
  }
  return buf;
};

NodeBuffer.allocUnsafe = function(size) {
  var buf = new NodeBuffer(size);
  return buf;
};

NodeBuffer.allocUnsafeSlow = function(size) {
  return new NodeBuffer(size);
};

NodeBuffer.compare = function(a, b) {
  var bytesA = a instanceof NodeBuffer ? a._data : (a instanceof Uint8Array ? a : new Uint8Array(0));
  var bytesB = b instanceof NodeBuffer ? b._data : (b instanceof Uint8Array ? b : new Uint8Array(0));
  var minLen = Math.min(bytesA.length, bytesB.length);
  for (var i = 0; i < minLen; i++) {
    if (bytesA[i] < bytesB[i]) return -1;
    if (bytesA[i] > bytesB[i]) return 1;
  }
  if (bytesA.length < bytesB.length) return -1;
  if (bytesA.length > bytesB.length) return 1;
  return 0;
};

NodeBuffer.prototype.compare = function(other, start, end, thisStart, thisEnd) {
  var otherBuf = other instanceof NodeBuffer ? other._data :
                 (other instanceof Uint8Array ? other : new Uint8Array(0));
  var oStart = start || 0;
  var oEnd = end !== undefined ? end : otherBuf.length;
  var tStart = thisStart || 0;
  var tEnd = thisEnd !== undefined ? tEnd : this._length;
  var oLen = oEnd - oStart;
  var tLen = tEnd - tStart;
  var minLen = Math.min(oLen, tLen);
  for (var i = 0; i < minLen; i++) {
    if (this._data[tStart + i] < otherBuf[oStart + i]) return -1;
    if (this._data[tStart + i] > otherBuf[oStart + i]) return 1;
  }
  if (tLen < oLen) return -1;
  if (tLen > oLen) return 1;
  return 0;
};

// ─── Instance Methods ───
NodeBuffer.prototype.toString = function(encoding, start, end) {
  var enc = encoding || 'utf8';
  var s = start || 0;
  var e = end !== undefined ? end : this._length;
  if (s < 0) s = Math.max(0, this._length + s);
  if (e < 0) e = Math.max(0, this._length + e);
  if (s >= this._length) return '';
  var slice = this._data.subarray(s, e);

  if (enc === 'utf8' || enc === 'utf-8') {
    return _utf8Decode(slice);
  }
  if (enc === 'ascii' || enc === 'binary') {
    var chars = [];
    for (var i = 0; i < slice.length; i++) chars.push(String.fromCharCode(slice[i]));
    return chars.join('');
  }
  if (enc === 'base64') return _toBase64(slice);
  if (enc === 'base64url') return _toBase64(slice).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  if (enc === 'hex') return _toHex(slice);
  if (enc === 'latin1' || enc === 'binary') {
    var chars2 = [];
    for (var i2 = 0; i2 < slice.length; i2++) chars2.push(String.fromCharCode(slice[i2]));
    return chars2.join('');
  }
  if (enc === 'utf16le' || enc === 'utf-16le') {
    var chars3 = [];
    for (var i3 = 0; i3 < slice.length - 1; i3 += 2) {
      chars3.push(String.fromCharCode(slice[i3] | (slice[i3 + 1] << 8)));
    }
    return chars3.join('');
  }
  return _utf8Decode(slice);
};

NodeBuffer.prototype.toJSON = function() {
  var data = [];
  for (var i = 0; i < this._length; i++) data.push(this._data[i]);
  return { type: 'Buffer', data: data };
};

NodeBuffer.prototype.slice = function(start, end) {
  var s = start || 0;
  var e = end !== undefined ? end : this._length;
  if (s < 0) s = Math.max(0, this._length + s);
  if (e < 0) e = Math.max(0, this._length + e);
  if (s >= this._length) return new NodeBuffer(0);
  var len = Math.max(0, e - s);
  var buf = new NodeBuffer(len);
  buf._data.set(this._data.subarray(s, s + len));
  return buf;
};

NodeBuffer.prototype.subarray = NodeBuffer.prototype.slice;

NodeBuffer.prototype.copy = function(target, targetStart, sourceStart, sourceEnd) {
  var targetBuf = target instanceof NodeBuffer ? target :
                  (target instanceof Uint8Array ? new NodeBuffer(target) : target);
  var tStart = targetStart || 0;
  var sStart = sourceStart || 0;
  var sEnd = sourceEnd !== undefined ? sourceEnd : this._length;
  if (sStart < 0) sStart = Math.max(0, this._length + sStart);
  if (sEnd < 0) sEnd = Math.max(0, this._length + sEnd);
  if (tStart < 0) tStart = Math.max(0, targetBuf._length + tStart);
  var len = Math.min(sEnd - sStart, targetBuf._length - tStart);
  if (len <= 0) return 0;
  targetBuf._data.set(this._data.subarray(sStart, sStart + len), tStart);
  return len;
};

NodeBuffer.prototype.fill = function(value, start, end, encoding) {
  var s = start || 0;
  var e = end !== undefined ? end : this._length;
  if (s < 0) s = Math.max(0, this._length + s);
  if (e < 0) e = Math.max(0, this._length + e);
  if (s >= this._length) return this;

  if (typeof value === 'string') {
    var fillBytes = _encodeString(value, encoding || 'utf8');
    for (var i = s; i < e; i++) {
      this._data[i] = fillBytes[(i - s) % fillBytes.length];
    }
  } else if (typeof value === 'number') {
    for (var i2 = s; i2 < e; i2++) {
      this._data[i2] = value & 0xff;
    }
  } else if (value instanceof Uint8Array || value instanceof NodeBuffer) {
    var src = value instanceof NodeBuffer ? value._data : value;
    for (var i3 = s; i3 < e; i3++) {
      this._data[i3] = src[(i3 - s) % src.length];
    }
  }
  return this;
};

NodeBuffer.prototype.write = function(string, offset, length, encoding) {
  var str = String(string);
  var enc = encoding || 'utf8';
  var off = offset || 0;
  var len = length !== undefined ? length : this._length - off;
  if (off < 0) off = 0;
  if (off >= this._length) return 0;
  var bytes;
  if (enc === 'ascii' || enc === 'binary' || enc === 'latin1') {
    bytes = new Uint8Array(str.length);
    for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0x7f;
  } else {
    bytes = _encodeString(str, enc);
  }
  var written = Math.min(bytes.length, len);
  this._data.set(bytes.subarray(0, written), off);
  return written;
};

NodeBuffer.prototype.readUInt8 = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off >= this._length)) throw new RangeError('Index out of range');
  return this._data[off];
};

NodeBuffer.prototype.readUInt16LE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  return this._data[off] | (this._data[off + 1] << 8);
};

NodeBuffer.prototype.readUInt16BE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  return (this._data[off] << 8) | this._data[off + 1];
};

NodeBuffer.prototype.readUInt32LE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  return (this._data[off] | (this._data[off + 1] << 8) | (this._data[off + 2] << 16) | (this._data[off + 3] << 24)) >>> 0;
};

NodeBuffer.prototype.readUInt32BE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  return ((this._data[off] << 24) | (this._data[off + 1] << 16) | (this._data[off + 2] << 8) | this._data[off + 3]) >>> 0;
};

NodeBuffer.prototype.readInt8 = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off >= this._length)) throw new RangeError('Index out of range');
  var val = this._data[off];
  return (val & 0x80) ? (0x100 - val) * -1 : val;
};

NodeBuffer.prototype.readInt16LE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  var val = this._data[off] | (this._data[off + 1] << 8);
  return (val & 0x8000) ? (0x10000 - val) * -1 : val;
};

NodeBuffer.prototype.readInt16BE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  var val = (this._data[off] << 8) | this._data[off + 1];
  return (val & 0x8000) ? (0x10000 - val) * -1 : val;
};

NodeBuffer.prototype.readInt32LE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  return this._data[off] | (this._data[off + 1] << 8) | (this._data[off + 2] << 16) | (this._data[off + 3] << 24);
};

NodeBuffer.prototype.readInt32BE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  return (this._data[off] << 24) | (this._data[off + 1] << 16) | (this._data[off + 2] << 8) | this._data[off + 3];
};

NodeBuffer.prototype.writeUInt8 = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = value & 0xff;
  return off + 1;
};

NodeBuffer.prototype.writeUInt16LE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = value & 0xff;
  this._data[off + 1] = (value >>> 8) & 0xff;
  return off + 2;
};

NodeBuffer.prototype.writeUInt16BE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = (value >>> 8) & 0xff;
  this._data[off + 1] = value & 0xff;
  return off + 2;
};

NodeBuffer.prototype.writeUInt32LE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = value & 0xff;
  this._data[off + 1] = (value >>> 8) & 0xff;
  this._data[off + 2] = (value >>> 16) & 0xff;
  this._data[off + 3] = (value >>> 24) & 0xff;
  return off + 4;
};

NodeBuffer.prototype.writeUInt32BE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = (value >>> 24) & 0xff;
  this._data[off + 1] = (value >>> 16) & 0xff;
  this._data[off + 2] = (value >>> 8) & 0xff;
  this._data[off + 3] = value & 0xff;
  return off + 4;
};

NodeBuffer.prototype.writeInt8 = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off >= this._length)) throw new RangeError('Index out of range');
  this._data[off] = value < 0 ? (256 + value) & 0xff : value & 0xff;
  return off + 1;
};

NodeBuffer.prototype.writeInt16LE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  var v = value < 0 ? (65536 + value) & 0xffff : value & 0xffff;
  this._data[off] = v & 0xff;
  this._data[off + 1] = (v >>> 8) & 0xff;
  return off + 2;
};

NodeBuffer.prototype.writeInt16BE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 1 >= this._length)) throw new RangeError('Index out of range');
  var v = value < 0 ? (65536 + value) & 0xffff : value & 0xffff;
  this._data[off] = (v >>> 8) & 0xff;
  this._data[off + 1] = v & 0xff;
  return off + 2;
};

NodeBuffer.prototype.writeInt32LE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var v = value < 0 ? (4294967296 + value) & 0xffffffff : value >>> 0;
  this._data[off] = v & 0xff;
  this._data[off + 1] = (v >>> 8) & 0xff;
  this._data[off + 2] = (v >>> 16) & 0xff;
  this._data[off + 3] = (v >>> 24) & 0xff;
  return off + 4;
};

NodeBuffer.prototype.writeInt32BE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var v = value < 0 ? (4294967296 + value) & 0xffffffff : value >>> 0;
  this._data[off] = (v >>> 24) & 0xff;
  this._data[off + 1] = (v >>> 16) & 0xff;
  this._data[off + 2] = (v >>> 8) & 0xff;
  this._data[off + 3] = v & 0xff;
  return off + 4;
};

NodeBuffer.prototype.writeFloatLE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  view.setFloat32(off, value, true);
  return off + 4;
};

NodeBuffer.prototype.writeFloatBE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  view.setFloat32(off, value, false);
  return off + 4;
};

NodeBuffer.prototype.writeDoubleLE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 7 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  view.setFloat64(off, value, true);
  return off + 8;
};

NodeBuffer.prototype.writeDoubleBE = function(value, offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 7 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  view.setFloat64(off, value, false);
  return off + 8;
};

NodeBuffer.prototype.readFloatLE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  return view.getFloat32(off, true);
};

NodeBuffer.prototype.readFloatBE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 3 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  return view.getFloat32(off, false);
};

NodeBuffer.prototype.readDoubleLE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 7 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  return view.getFloat64(off, true);
};

NodeBuffer.prototype.readDoubleBE = function(offset, noAssert) {
  var off = offset || 0;
  if (!noAssert && (off < 0 || off + 7 >= this._length)) throw new RangeError('Index out of range');
  var view = new DataView(this._data.buffer, this._data.byteOffset, this._data.byteLength);
  return view.getFloat64(off, false);
};

NodeBuffer.prototype.indexOf = function(value, byteOffset, encoding) {
  var off = byteOffset || 0;
  if (off < 0) off = Math.max(0, this._length + off);
  if (typeof value === 'string') {
    var enc = encoding || 'utf8';
    var searchBytes = _encodeString(value, enc);
    return _indexOfBytes(this._data, searchBytes, off);
  }
  if (value instanceof NodeBuffer) return _indexOfBytes(this._data, value._data, off);
  if (value instanceof Uint8Array) return _indexOfBytes(this._data, value, off);
  if (typeof value === 'number') return _indexOfByte(this._data, value, off);
  return -1;
};

NodeBuffer.prototype.lastIndexOf = function(value, byteOffset, encoding) {
  var off = byteOffset !== undefined ? byteOffset : this._length - 1;
  if (off < 0) off = Math.max(0, this._length + off);
  if (off >= this._length) off = this._length - 1;
  if (typeof value === 'string') {
    var enc = encoding || 'utf8';
    var searchBytes = _encodeString(value, enc);
    return _lastIndexOfBytes(this._data, searchBytes, off);
  }
  if (value instanceof NodeBuffer) return _lastIndexOfBytes(this._data, value._data, off);
  if (value instanceof Uint8Array) return _lastIndexOfBytes(this._data, value, off);
  if (typeof value === 'number') return _lastIndexOfByte(this._data, value, off);
  return -1;
};

NodeBuffer.prototype.includes = function(value, byteOffset, encoding) {
  return this.indexOf(value, byteOffset, encoding) !== -1;
};

NodeBuffer.prototype.equals = function(other) {
  if (!other) return false;
  var otherData = other instanceof NodeBuffer ? other._data :
                  (other instanceof Uint8Array ? other : null);
  if (!otherData) return false;
  if (this._length !== otherData.length) return false;
  for (var i = 0; i < this._length; i++) {
    if (this._data[i] !== otherData[i]) return false;
  }
  return true;
};

NodeBuffer.prototype.set = function(array, offset) {
  var off = offset || 0;
  var src = array instanceof NodeBuffer ? array._data :
            (array instanceof Uint8Array ? array : new Uint8Array(0));
  var len = Math.min(src.length, this._length - off);
  this._data.set(src.subarray(0, len), off);
};

NodeBuffer.prototype.entries = function() {
  var arr = [];
  for (var i = 0; i < this._length; i++) arr.push([i, this._data[i]]);
  return arr[Symbol.iterator]();
};

NodeBuffer.prototype.keys = function() {
  var arr = [];
  for (var i = 0; i < this._length; i++) arr.push(i);
  return arr[Symbol.iterator]();
};

NodeBuffer.prototype.values = function() {
  var arr = [];
  for (var i = 0; i < this._length; i++) arr.push(this._data[i]);
  return arr[Symbol.iterator]();
};

NodeBuffer.prototype.copyWithin = function(target, start, end) {
  var t = target;
  var s = start || 0;
  var e = end !== undefined ? end : this._length;
  if (t < 0) t = Math.max(0, this._length + t);
  if (s < 0) s = Math.max(0, this._length + s);
  if (e < 0) e = Math.max(0, this._length + e);
  var len = e - s;
  var temp = new Uint8Array(len);
  temp.set(this._data.subarray(s, s + len));
  this._data.set(temp.subarray(0, Math.min(len, this._length - t)), t);
  return this;
};

NodeBuffer.prototype.reverse = function() {
  var len = this._length;
  for (var i = 0; i < Math.floor(len / 2); i++) {
    var tmp = this._data[i];
    this._data[i] = this._data[len - 1 - i];
    this._data[len - 1 - i] = tmp;
  }
  return this;
};

NodeBuffer.prototype.swap16 = function() {
  for (var i = 0; i < this._length - 1; i += 2) {
    var tmp = this._data[i];
    this._data[i] = this._data[i + 1];
    this._data[i + 1] = tmp;
  }
  return this;
};

NodeBuffer.prototype.swap32 = function() {
  for (var i = 0; i < this._length - 3; i += 4) {
    var tmp = this._data[i];
    this._data[i] = this._data[i + 3];
    this._data[i + 3] = tmp;
    tmp = this._data[i + 1];
    this._data[i + 1] = this._data[i + 2];
    this._data[i + 2] = tmp;
  }
  return this;
};

NodeBuffer.prototype.swap64 = function() {
  for (var i = 0; i < this._length - 7; i += 8) {
    var tmp = this._data[i];
    this._data[i] = this._data[i + 7];
    this._data[i + 7] = tmp;
    tmp = this._data[i + 1];
    this._data[i + 1] = this._data[i + 6];
    this._data[i + 6] = tmp;
    tmp = this._data[i + 2];
    this._data[i + 2] = this._data[i + 5];
    this._data[i + 5] = tmp;
    tmp = this._data[i + 3];
    this._data[i + 3] = this._data[i + 4];
    this._data[i + 4] = tmp;
  }
  return this;
};

NodeBuffer.prototype.toISOString = function() {
  var hex = '';
  for (var i = 0; i < this._length; i++) {
    var h = this._data[i].toString(16);
    if (h.length < 2) h = '0' + h;
    hex += h;
  }
  return hex;
};

NodeBuffer.prototype.inspect = function() {
  return '<NodeBuffer ' + this.length + ' bytes>';
};

NodeBuffer.prototype[Symbol.iterator] = function() {
  var index = 0;
  var self = this;
  return {
    next: function() {
      if (index < self._length) {
        return { value: self._data[index++], done: false };
      }
      return { done: true };
    }
  };
};

NodeBuffer.prototype.toLocaleString = NodeBuffer.prototype.toString;

NodeBuffer.poolSize = 8192;

// ─── Encoding Helpers ───
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

function _encodeString(str, encoding) {
  var enc = _normalizeEncoding(encoding) || 'utf8';
  if (enc === 'utf8') return _utf8Encode(str);
  if (enc === 'ascii') {
    var bytes = new Uint8Array(str.length);
    for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0x7f;
    return bytes;
  }
  if (enc === 'latin1' || enc === 'binary') {
    var bytes2 = new Uint8Array(str.length);
    for (var i2 = 0; i2 < str.length; i2++) bytes2[i2] = str.charCodeAt(i2) & 0xff;
    return bytes2;
  }
  if (enc === 'base64') return _fromBase64(str);
  if (enc === 'base64url') return _fromBase64(str.replace(/-/g, '+').replace(/_/g, '/'));
  if (enc === 'hex') return _fromHex(str);
  if (enc === 'utf16le') return _utf16leEncode(str);
  return _utf8Encode(str);
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

function _utf16leEncode(str) {
  var bytes = new Uint8Array(str.length * 2);
  for (var i = 0; i < str.length; i++) {
    var cc = str.charCodeAt(i);
    bytes[i * 2] = cc & 0xff;
    bytes[i * 2 + 1] = (cc >> 8) & 0xff;
  }
  return bytes;
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

function _fromHex(hex) {
  var str = String(hex).replace(/[^0-9a-fA-F]/g, '');
  var bytes = new Uint8Array(Math.floor(str.length / 2));
  for (var i = 0; i < str.length; i += 2) {
    bytes[i / 2] = parseInt(str.substring(i, i + 2), 16);
  }
  return bytes;
}

function _toBase64(bytes) {
  var arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(0);
  var result = '';
  for (var i = 0; i < arr.length; i += 3) {
    var a = arr[i];
    var b = i + 1 < arr.length ? arr[i + 1] : 0;
    var c = i + 2 < arr.length ? arr[i + 2] : 0;
    var triple = (a << 16) | (b << 8) | c;
    result += _b64chars[(triple >> 18) & 0x3F];
    result += _b64chars[(triple >> 12) & 0x3F];
    result += i + 1 < arr.length ? _b64chars[(triple >> 6) & 0x3F] : '=';
    result += i + 2 < arr.length ? _b64chars[triple & 0x3F] : '=';
  }
  return result;
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

function _indexOfBytes(haystack, needle, offset) {
  if (needle.length === 0) return offset;
  for (var i = offset; i <= haystack.length - needle.length; i++) {
    var found = true;
    for (var j = 0; j < needle.length; j++) {
      if (haystack[i + j] !== needle[j]) { found = false; break; }
    }
    if (found) return i;
  }
  return -1;
}

function _indexOfByte(haystack, byte, offset) {
  for (var i = offset; i < haystack.length; i++) {
    if (haystack[i] === byte) return i;
  }
  return -1;
}

function _lastIndexOfBytes(haystack, needle, offset) {
  if (needle.length === 0) return offset;
  var maxStart = Math.min(offset, haystack.length - needle.length);
  for (var i = maxStart; i >= 0; i--) {
    var found = true;
    for (var j = 0; j < needle.length; j++) {
      if (haystack[i + j] !== needle[j]) { found = false; break; }
    }
    if (found) return i;
  }
  return -1;
}

function _lastIndexOfByte(haystack, byte, offset) {
  for (var i = Math.min(offset, haystack.length - 1); i >= 0; i--) {
    if (haystack[i] === byte) return i;
  }
  return -1;
}

// ─── TextEncoder / TextDecoder ───
function TextEncoderShim() {
  this.encoding = 'utf-8';
}

TextEncoderShim.prototype.encode = function(input) {
  return _utf8Encode(String(input || ''));
};

TextEncoderShim.prototype.encodeInto = function(src, dest) {
  var srcBytes = _utf8Encode(String(src || ''));
  var written = Math.min(srcBytes.length, dest.length);
  dest.set(srcBytes.subarray(0, written));
  return { read: _utf8Decode(srcBytes.subarray(0, written)).length, written: written };
};

function TextDecoderShim(label, options) {
  var opts = options || {};
  var enc = _normalizeEncoding(label) || 'utf8';
  if (enc === 'utf16le') enc = 'utf16le';
  this.encoding = enc;
  this.fatal = opts.fatal || false;
  this.ignoreBOM = opts.ignoreBOM || false;
}

TextDecoderShim.prototype.decode = function(input) {
  var bytes;
  if (input instanceof NodeBuffer) {
    bytes = input._data;
  } else if (input instanceof Uint8Array) {
    bytes = input;
  } else if (input && input.buffer) {
    bytes = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  } else {
    bytes = new Uint8Array(0);
  }

  var enc = this.encoding;
  if (enc === 'utf8' || enc === 'utf-8') {
    var start = 0;
    if (!this.ignoreBOM && bytes.length >= 3 &&
        bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
      start = 3;
    }
    return _utf8Decode(bytes.subarray(start));
  }
  if (enc === 'ascii' || enc === 'latin1' || enc === 'binary') {
    var chars = [];
    for (var i = 0; i < bytes.length; i++) chars.push(String.fromCharCode(bytes[i]));
    return chars.join('');
  }
  if (enc === 'utf16le') {
    var chars2 = [];
    for (var i2 = 0; i2 < bytes.length - 1; i2 += 2) {
      chars2.push(String.fromCharCode(bytes[i2] | (bytes[i2 + 1] << 8)));
    }
    return chars2.join('');
  }
  if (enc === 'hex') return _toHex(bytes);
  if (enc === 'base64') return _toBase64(bytes);
  return _utf8Decode(bytes);
};

// ─── atob / btoa ───
function _atob(str) {
  var bytes = _fromBase64(String(str));
  var chars = [];
  for (var i = 0; i < bytes.length; i++) chars.push(String.fromCharCode(bytes[i]));
  return chars.join('');
}

function _btoa(str) {
  var bytes = [];
  for (var i = 0; i < str.length; i++) bytes.push(str.charCodeAt(i) & 0xff);
  return _toBase64(new Uint8Array(bytes));
}

// ─── Module Exports ───
module.exports = {
  Buffer: NodeBuffer,
  Blob: Blob,
  TextEncoder: TextEncoderShim,
  TextDecoder: TextDecoderShim,
  atob: _atob,
  btoa: _btoa,
  constants: constants,
  INSPECT_MAX_BYTES: 50,
  kMaxLength: kMaxLength,
};
