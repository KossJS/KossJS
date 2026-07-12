// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/string_decoder - Node.js string_decoder module (L3)

function createTextDecoderPolyfill() {
  if (typeof TextDecoder !== 'undefined') return TextDecoder;
  return function(encoding) {
    this.encoding = encoding || 'utf-8';
    this.decode = function(bytes) {
      if (!bytes || bytes.length === 0) return '';
      const chars = [];
      let i = 0;
      while (i < bytes.length) {
        const byte = bytes[i++];
        if (byte < 0x80) {
          chars.push(byte);
        } else if (byte < 0xe0) {
          const byte2 = bytes[i++] & 0x3f;
          chars.push(((byte & 0x1f) << 6) | byte2);
        } else if (byte < 0xf0) {
          const byte2 = bytes[i++] & 0x3f;
          const byte3 = bytes[i++] & 0x3f;
          chars.push(((byte & 0x0f) << 12) | (byte2 << 6) | byte3);
        }
      }
      return String.fromCharCode(...chars);
    };
  };
}

const _TextDecoder = createTextDecoderPolyfill();

class StringDecoder {
  constructor(encoding = 'utf8') {
    this.encoding = encoding.toLowerCase();
    this._decoder = new _TextDecoder(this.encoding === 'utf8' || this.encoding === 'utf-8' ? 'utf-8' : this.encoding);
  }

  write(buffer) {
    if (typeof buffer === 'string') return buffer;
    const uint8 = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    return this._decoder.decode(uint8, { stream: true });
  }

  end(buffer) {
    if (buffer) return this.write(buffer);
    return '';
  }

  text(buffer, offset, length) {
    if (offset || length) {
      const uint8 = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
      const slice = uint8.slice(offset || 0, length !== undefined ? offset + length : undefined);
      return this._decoder.decode(slice);
    }
    return this._decoder.decode(buffer);
  }

  fillLast(buffer) {
    return this.end(buffer);
  }
}

module.exports = { StringDecoder };
