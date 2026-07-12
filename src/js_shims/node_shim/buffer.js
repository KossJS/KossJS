// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/buffer - Node.js buffer module (L3)

const __NativeBuffer = globalThis.Buffer;

function __KossTE(enc) {
    this.encode = function(str) {
        const bytes = [];
        for (let i = 0; i < str.length; i++) {
            const cc = str.charCodeAt(i);
            if (cc < 0x80) { bytes.push(cc); }
            else if (cc < 0x800) { bytes.push(0xc0 | (cc >> 6)); bytes.push(0x80 | (cc & 0x3f)); }
            else if (cc < 0x10000) { bytes.push(0xe0 | (cc >> 12)); bytes.push(0x80 | ((cc >> 6) & 0x3f)); bytes.push(0x80 | (cc & 0x3f)); }
        }
        return new Uint8Array(bytes);
    };
}

function __KossTD() {
    this.decode = function(bytes) {
        if (!bytes || bytes.length === 0) return '';
        const chars = [];
        let i = 0;
        while (i < bytes.length) {
            const b = bytes[i++];
            if (b < 0x80) { chars.push(b); }
            else if (b < 0xe0) { const b2 = bytes[i++] & 0x3f; chars.push(((b & 0x1f) << 6) | b2); }
            else if (b < 0xf0) { const b2 = bytes[i++] & 0x3f; const b3 = bytes[i++] & 0x3f; chars.push(((b & 0x0f) << 12) | (b2 << 6) | b3); }
        }
        return String.fromCharCode(...chars);
    };
}

if (typeof globalThis.TextEncoder === 'undefined') globalThis.TextEncoder = __KossTE;
if (typeof globalThis.TextDecoder === 'undefined') globalThis.TextDecoder = __KossTD;

const _b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
const _b64lookup = new Uint8Array(256);
for (let i = 0; i < 64; i++) _b64lookup[_b64chars.charCodeAt(i)] = i;

function _base64Decode(str) {
    const clean = str.replace(/[^A-Za-z0-9+/]/g, '');
    const len = clean.length;
    const outLen = (len * 3) >> 2;
    const result = new Uint8Array(outLen);
    let j = 0;
    for (let i = 0; i < len; i += 4) {
        const a = _b64lookup[clean.charCodeAt(i)] || 0;
        const b = _b64lookup[clean.charCodeAt(i + 1)] || 0;
        const c = _b64lookup[clean.charCodeAt(i + 2)] || 0;
        const d = _b64lookup[clean.charCodeAt(i + 3)] || 0;
        result[j++] = (a << 2) | (b >> 4);
        if (i + 2 < len) result[j++] = ((b & 15) << 4) | (c >> 2);
        if (i + 3 < len) result[j++] = ((c & 3) << 6) | d;
    }
    return result.slice(0, j);
}

function _base64Encode(bytes) {
    let result = '';
    for (let i = 0; i < bytes.length; i += 3) {
        const a = bytes[i];
        const b = bytes[i + 1] || 0;
        const c = bytes[i + 2] || 0;
        result += _b64chars[a >> 2];
        result += _b64chars[((a & 3) << 4) | (b >> 4)];
        result += (i + 1 < bytes.length) ? _b64chars[((b & 15) << 2) | (c >> 6)] : '=';
        result += (i + 2 < bytes.length) ? _b64chars[c & 63] : '=';
    }
    return result;
}

function _hexDecode(str) {
    const result = new Uint8Array(str.length / 2);
    for (let i = 0; i < str.length; i += 2) {
        result[i / 2] = parseInt(str.substr(i, 2), 16);
    }
    return result;
}

function _hexEncode(bytes) {
    let result = '';
    for (let i = 0; i < bytes.length; i++) {
        result += bytes[i].toString(16).padStart(2, '0');
    }
    return result;
}

function NodeBuffer(arg, encodingOrOffset, length) {
    if (!(this instanceof NodeBuffer)) {
        if (typeof arg === 'string') return NodeBuffer.from(arg, encodingOrOffset);
        if (typeof arg === 'number') return NodeBuffer.alloc(arg);
        if (Array.isArray(arg)) return NodeBuffer.from(arg);
        return NodeBuffer.from(arg);
    }
    this._data = new Uint8Array(arg || 0);
    _bindIndexProps(this, this._data);
}

NodeBuffer.from = function(value, encoding) {
    if (typeof value === 'string') {
        var buf = new NodeBuffer(0);
        if (encoding === 'base64') {
            buf._data = _base64Decode(value);
        } else if (encoding === 'hex') {
            buf._data = _hexDecode(value);
        } else {
            var enc = new globalThis.TextEncoder();
            buf._data = enc.encode(value);
        }
        _bindIndexProps(buf, buf._data);
        return buf;
    }
    if (ArrayBuffer.isView(value)) {
        var buf2 = new NodeBuffer(0);
        buf2._data = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
        _bindIndexProps(buf2, buf2._data);
        return buf2;
    }
    if (value instanceof ArrayBuffer) {
        var buf3 = new NodeBuffer(0);
        buf3._data = new Uint8Array(value);
        _bindIndexProps(buf3, buf3._data);
        return buf3;
    }
    if (Array.isArray(value)) {
        var buf4 = new NodeBuffer(0);
        buf4._data = new Uint8Array(value);
        _bindIndexProps(buf4, buf4._data);
        return buf4;
    }
    return new NodeBuffer(0);
};

NodeBuffer.alloc = function(size, fill) {
    var buf = new NodeBuffer(0);
    buf._data = new Uint8Array(size);
    if (fill !== undefined) buf._data.fill(typeof fill === 'string' ? fill.charCodeAt(0) : fill);
    _bindIndexProps(buf, buf._data);
    return buf;
};

NodeBuffer.allocUnsafe = function(size) {
    return new NodeBuffer(size);
};

NodeBuffer.byteLength = function(string, encoding) {
    if (encoding === 'base64') {
        var clean = string.replace(/[^A-Za-z0-9+/]/g, '');
        return Math.floor(clean.length * 3 / 4);
    }
    if (encoding === 'hex') return Math.floor(string.length / 2);
    if (encoding === 'utf8' || encoding === 'utf-8') {
        return new globalThis.TextEncoder().encode(string).length;
    }
    return string.length;
};

NodeBuffer.concat = function(list) {
    var total = list.reduce(function(sum, b) { return sum + b._data.length; }, 0);
    var data = new Uint8Array(total);
    var offset = 0;
    for (var i = 0; i < list.length; i++) {
        data.set(list[i]._data, offset);
        offset += list[i]._data.length;
    }
    var buf = new NodeBuffer(0);
    buf._data = data;
    _bindIndexProps(buf, data);
    return buf;
};

NodeBuffer.compare = function(a, b) {
    for (var i = 0; i < Math.min(a._data.length, b._data.length); i++) {
        if (a._data[i] !== b._data[i]) return a._data[i] - b._data[i];
    }
    return a._data.length - b._data.length;
};

NodeBuffer.isBuffer = function(obj) {
    return obj instanceof NodeBuffer || (obj && obj._data instanceof Uint8Array);
};

NodeBuffer.isEncoding = function(encoding) {
    return ['utf8', 'utf-8', 'utf16le', 'latin1', 'ascii', 'base64', 'hex'].includes(String(encoding).toLowerCase());
};

function _bindIndexProps(buf, data) {
    for (var i = 0; i < data.length; i++) {
        (function(idx) {
            Object.defineProperty(buf, String(idx), {
                get: function() { return data[idx]; },
                set: function(v) { data[idx] = typeof v === 'number' ? v & 0xff : 0; },
                enumerable: true,
                configurable: true,
            });
        })(i);
    }
}

Object.defineProperty(NodeBuffer.prototype, 'toString', {
    value: function(encoding, start, end) {
        var slice = this._data.subarray(start || 0, end || this._data.length);
        if (encoding === 'base64') return _base64Encode(slice);
        if (encoding === 'hex') return _hexEncode(slice);
        return new globalThis.TextDecoder().decode(slice);
    },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'slice', {
    value: function(start, end) {
        var buf = new NodeBuffer(0);
        buf._data = this._data.subarray(start, end);
        _bindIndexProps(buf, buf._data);
        return buf;
    },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'copy', {
    value: function(target, targetStart, sourceStart, sourceEnd) {
        target._data.set(this._data.subarray(sourceStart || 0, sourceEnd || this._data.length), targetStart || 0);
    },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'fill', {
    value: function(value, offset, end) {
        this._data.fill(typeof value === 'string' ? value.charCodeAt(0) : value, offset || 0, end || this._data.length);
        return this;
    },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'write', {
    value: function(string, offset) {
        var bytes = NodeBuffer.from(string)._data;
        this._data.set(bytes, offset || 0);
        return bytes.length;
    },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'readUInt8', {
    value: function(offset) { return this._data[offset]; },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'writeUInt8', {
    value: function(value, offset) { this._data[offset] = value; return offset + 1; },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'toJSON', {
    value: function() { return { type: 'Buffer', data: Array.from(this._data) }; },
    writable: true,
    configurable: true,
});

Object.defineProperty(NodeBuffer.prototype, 'length', {
    get: function() { return this._data.length; },
    enumerable: true
});

const Blob = globalThis.Blob;

function atob(data) { return NodeBuffer.from(data, 'base64').toString(); }
function btoa(data) { return NodeBuffer.from(data).toString('base64'); }

const constants = { MAX_LENGTH: 4294967296, MAX_STRING_LENGTH: 1073741824 };
const kMaxLength = constants.MAX_LENGTH;
const kStringMaxLength = constants.MAX_STRING_LENGTH;

module.exports = {
    Buffer: NodeBuffer,
    Blob,
    TextEncoder: globalThis.TextEncoder,
    TextDecoder: globalThis.TextDecoder,
    atob,
    btoa,
    constants,
    kMaxLength,
    kStringMaxLength,
};
