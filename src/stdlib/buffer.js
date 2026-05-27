'use strict';

const kMaxLength = 2147483647;
const kStringMaxLength = 536870888;

class FastBuffer extends Uint8Array {
  constructor(buffer, byteOffset, length) {
    if (typeof buffer === 'number') {
      super(buffer);
    } else if (buffer instanceof ArrayBuffer) {
      super(buffer, byteOffset, length);
    } else if (ArrayBuffer.isView(buffer)) {
      super(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    } else {
      super(buffer);
    }
  }
}

function createUnsafeBuffer(size) {
  return new FastBuffer(size);
}

function addBufferPrototypeMethods(proto) {
  proto.copy = function(target, targetStart, sourceStart, sourceEnd) {
    if (sourceStart === undefined) sourceStart = 0;
    if (sourceEnd === undefined) sourceEnd = this.length;
    if (targetStart === undefined) targetStart = 0;
    
    const sourceLen = sourceEnd - sourceStart;
    const targetLen = target.length - targetStart;
    const len = Math.min(sourceLen, targetLen);
    
    for (let i = 0; i < len; i++) {
      target[targetStart + i] = this[sourceStart + i];
    }
    return len;
  };

  proto.fill = function(value, offset, end, encoding) {
    if (offset === undefined) offset = 0;
    if (end === undefined) end = this.length;
    if (typeof value === 'string') {
      value = value.charCodeAt(0);
    }
    for (let i = offset; i < end; i++) {
      this[i] = value;
    }
    return this;
  };

  proto.includes = function(val, byteOffset, encoding) {
    return this.indexOf(val, byteOffset, encoding) !== -1;
  };

  proto.indexOf = function(val, byteOffset, encoding) {
    if (typeof val === 'number') {
      for (let i = byteOffset || 0; i < this.length; i++) {
        if (this[i] === val) return i;
      }
      return -1;
    }
    if (typeof val === 'string') {
      const str = this.toString('utf8', 0);
      return str.indexOf(val, byteOffset);
    }
    return -1;
  };

  proto.lastIndexOf = function(val, byteOffset, encoding) {
    if (typeof val === 'number') {
      for (let i = (byteOffset || this.length) - 1; i >= 0; i--) {
        if (this[i] === val) return i;
      }
      return -1;
    }
    return -1;
  };

  proto.slice = function(start, end) {
    return new FastBuffer(this.buffer, this.byteOffset + start, end - start);
  };

  proto.subarray = function(start, end) {
    return new FastBuffer(this.buffer, this.byteOffset + start, (end || this.length) - start);
  };

  Object.defineProperty(proto, 'toString', {
    writable: true, configurable: true,
    value: function(encoding, start, end) {
    if (encoding === undefined || encoding === 'utf8' || encoding === 'utf-8') {
      let str = '';
      const startIdx = start || 0;
      const endIdx = end || this.length;
      for (let i = startIdx; i < endIdx; i++) {
        str += String.fromCharCode(this[i]);
      }
      return decodeURIComponent(encodeURIComponent(str));
    }
    if (encoding === 'hex') {
      const startIdx = start || 0;
      const endIdx = end || this.length;
      let hex = '';
      for (let i = startIdx; i < endIdx; i++) {
        hex += (this[i] < 16 ? '0' : '') + this[i].toString(16);
      }
      return hex;
    }
    if (encoding === 'base64') {
      const startIdx = start || 0;
      const endIdx = end || this.length;
      const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
      let output = '';
      for (let i = startIdx; i < endIdx; i += 3) {
        const a = this[i];
        const b = i + 1 < endIdx ? this[i + 1] : 0;
        const c = i + 2 < endIdx ? this[i + 2] : 0;
        const triplet = (a << 16) | (b << 8) | c;
        output += chars[(triplet >> 18) & 0x3F];
        output += chars[(triplet >> 12) & 0x3F];
        output += i + 1 < endIdx ? chars[(triplet >> 6) & 0x3F] : '=';
        output += i + 2 < endIdx ? chars[triplet & 0x3F] : '=';
      }
      return output;
    }
    if (encoding === 'latin1' || encoding === 'binary') {
      const startIdx = start || 0;
      const endIdx = end || this.length;
      let str = '';
      for (let i = startIdx; i < endIdx; i++) {
        str += String.fromCharCode(this[i]);
      }
      return str;
    }
    return '';
  }
  });

  proto.equals = function(otherBuffer) {
    if (!ArrayBuffer.isView(otherBuffer)) return false;
    if (this.length !== otherBuffer.length) return false;
    for (let i = 0; i < this.length; i++) {
      if (this[i] !== otherBuffer[i]) return false;
    }
    return true;
  };

  proto.compare = function(target, targetStart, targetEnd, sourceStart, sourceEnd) {
    if (targetStart === undefined) targetStart = 0;
    if (targetEnd === undefined) targetEnd = target.length;
    if (sourceStart === undefined) sourceStart = 0;
    if (sourceEnd === undefined) sourceEnd = this.length;

    const minLen = Math.min(targetEnd - targetStart, sourceEnd - sourceStart);
    for (let i = 0; i < minLen; i++) {
      const cmp = this[sourceStart + i] - target[targetStart + i];
      if (cmp !== 0) return cmp;
    }
    return (targetEnd - targetStart) - (sourceEnd - sourceStart);
  };

  proto.swap16 = function() {
    for (let i = 0; i < this.length; i += 2) {
      const temp = this[i];
      this[i] = this[i + 1];
      this[i + 1] = temp;
    }
    return this;
  };

  proto.swap32 = function() {
    for (let i = 0; i < this.length; i += 4) {
      const temp = this[i];
      this[i] = this[i + 3];
      this[i + 3] = temp;
      const temp2 = this[i + 1];
      this[i + 1] = this[i + 2];
      this[i + 2] = temp2;
    }
    return this;
  };

  proto.swap64 = function() {
    for (let i = 0; i < this.length; i += 8) {
      const temp = this[i];
      this[i] = this[i + 7];
      this[i + 7] = temp;
      const temp2 = this[i + 1];
      this[i + 1] = this[i + 6];
      this[i + 6] = temp2;
      const temp3 = this[i + 2];
      this[i + 2] = this[i + 5];
      this[i + 5] = temp3;
      const temp4 = this[i + 3];
      this[i + 3] = this[i + 4];
      this[i + 4] = temp4;
    }
    return this;
  };

  proto.write = function(string, offset, length, encoding) {
    if (offset === undefined) offset = 0;
    if (encoding === undefined) encoding = 'utf8';
    
    const str = String(string);
    let written = 0;
    
    for (let i = 0; i < str.length && offset + i < this.length; i++) {
      const code = str.charCodeAt(i);
      if (code > 127 && encoding === 'utf8') {
        this[offset + i] = code & 0xFF;
        written++;
      } else {
        this[offset + i] = code;
        written++;
      }
    }
    return written;
  };

  proto.toJSON = function() {
    const data = [];
    for (let i = 0; i < this.length; i++) {
      data.push(this[i]);
    }
    return { type: 'Buffer', data };
  };

  proto.writeUInt8 = function(value, offset) {
    this[offset || 0] = value & 0xFF;
    return offset + 1;
  };

  proto.writeUInt16LE = function(value, offset) {
    const o = offset || 0;
    this[o] = value & 0xFF;
    this[o + 1] = (value >> 8) & 0xFF;
    return o + 2;
  };

  proto.writeUInt16BE = function(value, offset) {
    const o = offset || 0;
    this[o] = (value >> 8) & 0xFF;
    this[o + 1] = value & 0xFF;
    return o + 2;
  };

  proto.writeUInt32LE = function(value, offset) {
    const o = offset || 0;
    this[o] = value & 0xFF;
    this[o + 1] = (value >> 8) & 0xFF;
    this[o + 2] = (value >> 16) & 0xFF;
    this[o + 3] = (value >> 24) & 0xFF;
    return o + 4;
  };

  proto.writeUInt32BE = function(value, offset) {
    const o = offset || 0;
    this[o] = (value >> 24) & 0xFF;
    this[o + 1] = (value >> 16) & 0xFF;
    this[o + 2] = (value >> 8) & 0xFF;
    this[o + 3] = value & 0xFF;
    return o + 4;
  };

  proto.writeInt8 = function(value, offset) {
    this[offset || 0] = value & 0xFF;
    return offset + 1;
  };

  proto.readUInt8 = function(offset) {
    return this[offset || 0];
  };

  proto.readUInt16LE = function(offset) {
    const o = offset || 0;
    return this[o] | (this[o + 1] << 8);
  };

  proto.readUInt16BE = function(offset) {
    const o = offset || 0;
    return (this[o] << 8) | this[o + 1];
  };

  proto.readUInt32LE = function(offset) {
    const o = offset || 0;
    return this[o] + (this[o + 1] << 8) + (this[o + 2] << 16) + (this[o + 3] << 24);
  };

  proto.readUInt32BE = function(offset) {
    const o = offset || 0;
    return (this[o] << 24) + (this[o + 1] << 16) + (this[o + 2] << 8) + this[o + 3];
  };

  proto.readInt8 = function(offset) {
    const val = this[offset || 0];
    return val > 127 ? val - 256 : val;
  };

  proto.readInt16LE = function(offset) {
    const val = this.readUInt16LE(offset);
    return val > 32767 ? val - 65536 : val;
  };

  proto.readInt16BE = function(offset) {
    const val = this.readUInt16BE(offset);
    return val > 32767 ? val - 65536 : val;
  };

  proto.readInt32LE = function(offset) {
    const val = this.readUInt32LE(offset);
    return val > 2147483647 ? val - 4294967296 : val;
  };

  proto.readInt32BE = function(offset) {
    const val = this.readUInt32BE(offset);
    return val > 2147483647 ? val - 4294967296 : val;
  };
}

function markAsUntransferable(buffer) {}

addBufferPrototypeMethods(FastBuffer.prototype);
addBufferPrototypeMethods(Buffer.prototype);

function Buffer(arg, encodingOrOffset, length) {
  if (typeof arg === 'number') {
    return Buffer.alloc(arg);
  }
  return Buffer.from(arg, encodingOrOffset, length);
}

Object.defineProperty(Buffer, Symbol('species'), {
  __proto__: null,
  enumerable: false,
  configurable: true,
  get() { return FastBuffer; },
});

Buffer.from = function(value, encodingOrOffset, length) {
  if (typeof value === 'string') {
    const arr = new FastBuffer(value.length);
    for (let i = 0; i < value.length; i++) {
      arr[i] = value.charCodeAt(i);
    }
    return arr;
  }
  
  if (Array.isArray(value)) {
    const arr = new FastBuffer(value.length);
    for (let i = 0; i < value.length; i++) {
      arr[i] = value[i];
    }
    return arr;
  }
  
  if (value instanceof ArrayBuffer) {
    return new FastBuffer(value, encodingOrOffset, length);
  }
  
  if (ArrayBuffer.isView(value)) {
    const arr = new FastBuffer(value.byteLength);
    arr.set(value);
    return arr;
  }
  
  return new FastBuffer(0);
};

Buffer.alloc = function(size, fill, encoding) {
  const buf = new FastBuffer(size);
  if (fill !== undefined && fill !== 0) {
    if (typeof fill === 'string') {
      fill = fill.charCodeAt(0);
    }
    buf.fill(fill);
  }
  return buf;
};

Buffer.allocUnsafe = function(size) {
  return new FastBuffer(size);
};

Buffer.allocUnsafeSlow = function(size) {
  return new FastBuffer(size);
};

Buffer.isBuffer = function(b) {
  return b instanceof FastBuffer;
};

Buffer.compare = function(buf1, buf2) {
  if (!(ArrayBuffer.isView(buf1)) || !(ArrayBuffer.isView(buf2))) {
    throw new TypeError('Argument must be a buffer');
  }
  if (buf1 === buf2) return 0;
  const len = Math.min(buf1.length, buf2.length);
  for (let i = 0; i < len; i++) {
    if (buf1[i] !== buf2[i]) {
      return buf1[i] < buf2[i] ? -1 : 1;
    }
  }
  return buf1.length - buf2.length;
};

Buffer.isEncoding = function(encoding) {
  const validEncodings = ['utf8', 'utf-16le', 'ucs2', 'ascii', 'latin1', 'base64', 'hex'];
  return typeof encoding === 'string' && validEncodings.includes(encoding.toLowerCase());
};

Buffer.concat = function(list, length) {
  if (!Array.isArray(list)) {
    throw new TypeError('list argument must be an Array');
  }
  
  if (list.length === 0) {
    return new FastBuffer(0);
  }
  
  if (length === undefined) {
    length = 0;
    for (let i = 0; i < list.length; i++) {
      if (ArrayBuffer.isView(list[i])) {
        length += list[i].length;
      }
    }
  }
  
  const buf = new FastBuffer(length);
  let offset = 0;
  for (let i = 0; i < list.length && offset < length; i++) {
    const item = list[i];
    if (ArrayBuffer.isView(item)) {
      const len = Math.min(item.length, length - offset);
      for (let j = 0; j < len; j++) {
        buf[offset++] = item[j];
      }
    }
  }
  return buf;
};

Buffer.byteLength = function(string, encoding) {
  if (typeof string !== 'string') {
    if (ArrayBuffer.isView(string)) {
      return string.byteLength;
    }
    if (string instanceof ArrayBuffer) {
      return string.byteLength;
    }
    throw new TypeError('Argument must be a string or Buffer');
  }
  if (encoding === 'utf8' || encoding === 'utf-8' || encoding === undefined) {
    // UTF-8 byte length: count multi-byte characters
    let len = 0;
    for (let i = 0; i < string.length; i++) {
      const code = string.charCodeAt(i);
      if (code < 0x80) len += 1;
      else if (code < 0x800) len += 2;
      else if (code < 0x10000) len += 3;
      else len += 4;
    }
    return len;
  }
  if (encoding === 'hex') return string.length / 2;
  if (encoding === 'base64') return Math.ceil(string.length * 3 / 4);
  if (encoding === 'latin1' || encoding === 'binary') return string.length;
  if (encoding === 'utf16le' || encoding === 'ucs2') return string.length * 2;
  return string.length;
};

Buffer.poolSize = 8 * 1024;

const constants = Object.defineProperties({}, {
  MAX_LENGTH: {
    __proto__: null,
    value: kMaxLength,
    writable: false,
    enumerable: true,
  },
  MAX_STRING_LENGTH: {
    __proto__: null,
    value: kStringMaxLength,
    writable: false,
    enumerable: true,
  },
});

FastBuffer.prototype.constructor = Buffer;
Buffer.prototype = FastBuffer.prototype;

function isUtf8(buf) {
  for (let i = 0; i < buf.length; i++) {
    if (buf[i] > 127) return false;
  }
  return true;
}

function isAscii(buf) {
  for (let i = 0; i < buf.length; i++) {
    if (buf[i] > 127) return false;
  }
  return true;
}

function btoa(input) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
  let output = '';
  for (let i = 0; i < input.length; i += 3) {
    const a = input.charCodeAt(i);
    const b = i + 1 < input.length ? input.charCodeAt(i + 1) : 0;
    const c = i + 2 < input.length ? input.charCodeAt(i + 2) : 0;
    const triplet = (a << 16) | (b << 8) | c;
    output += chars[(triplet >> 18) & 0x3F];
    output += chars[(triplet >> 12) & 0x3F];
    output += i + 1 < input.length ? chars[(triplet >> 6) & 0x3F] : '=';
    output += i + 2 < input.length ? chars[triplet & 0x3F] : '=';
  }
  return output;
}

function atob(input) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
  let output = '';
  for (let i = 0; i < input.length; i += 4) {
    const a = chars.indexOf(input[i]);
    const b = i + 1 < input.length ? chars.indexOf(input[i + 1]) : 0;
    const c = i + 2 < input.length ? chars.indexOf(input[i + 2]) : 0;
    const d = i + 3 < input.length ? chars.indexOf(input[i + 3]) : 0;
    const triplet = (a << 18) | (b << 12) | (c << 6) | d;
    output += String.fromCharCode((triplet >> 16) & 0xFF);
    if (input[i + 2] !== '=') {
      output += String.fromCharCode((triplet >> 8) & 0xFF);
    }
    if (input[i + 3] !== '=') {
      output += String.fromCharCode(triplet & 0xFF);
    }
  }
  return output;
}

module.exports = {
  Buffer,
  transcode: undefined,
  isUtf8,
  isAscii,
  kMaxLength,
  kStringMaxLength,
  btoa,
  atob,
};

Object.defineProperties(module.exports, {
  constants: {
    __proto__: null,
    configurable: false,
    enumerable: true,
    value: constants,
  },
});