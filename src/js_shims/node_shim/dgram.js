// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/dgram - Node.js dgram module (L3)
// 基于 __koss_udp_* Rust 原生函数实现真实 UDP 收发。

var events = require('koss:events');
var EventEmitter = events.EventEmitter;

var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;

function _toBytes(data) {
  if (Buffer && Buffer.isBuffer(data)) return data._data || data;
  if (data instanceof Uint8Array) return data;
  if (Array.isArray(data)) return new Uint8Array(data);
  if (typeof data === 'string') {
    var bytes = new Uint8Array(data.length);
    for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
    return bytes;
  }
  return new Uint8Array(0);
}

function _b64c(bytes) {
  var out = '';
  var i = 0;
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  for (; i + 2 < bytes.length; i += 3) {
    var n = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2];
    out += chars[(n >> 18) & 63] + chars[(n >> 12) & 63] + chars[(n >> 6) & 63] + chars[n & 63];
  }
  var rem = bytes.length - i;
  if (rem === 1) {
    var m = bytes[i] << 16;
    out += chars[(m >> 18) & 63] + chars[(m >> 12) & 63] + '==';
  } else if (rem === 2) {
    var k = (bytes[i] << 16) | (bytes[i + 1] << 8);
    out += chars[(k >> 18) & 63] + chars[(k >> 12) & 63] + chars[(k >> 6) & 63] + '=';
  }
  return out;
}

function _b64dec(s) {
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var lookup = {};
  for (var i = 0; i < chars.length; i++) lookup[chars.charCodeAt(i)] = i;
  var out = [];
  for (var j = 0; j < s.length; j += 4) {
    var a = lookup[s.charCodeAt(j)] || 0;
    var b = lookup[s.charCodeAt(j + 1)] || 0;
    var c = lookup[s.charCodeAt(j + 2)] || 0;
    var d = lookup[s.charCodeAt(j + 3)] || 0;
    var t = (a << 18) | (b << 12) | (c << 6) | d;
    out.push((t >> 16) & 0xFF);
    if (s[j + 2] !== '=') out.push((t >> 8) & 0xFF);
    if (s[j + 3] !== '=') out.push(t & 0xFF);
  }
  return new Uint8Array(out);
}

var nextTick = (typeof process !== 'undefined' && process.nextTick) ? process.nextTick : setTimeout;

class Socket extends EventEmitter {
  constructor(options) {
    super();
    this.type = options?.type || 'udp4';
    this.reuseAddr = options?.reuseAddr || false;
    this.ipv6Only = options?.ipv6Only || false;
    this._fd = null;
    this._bound = false;
    this._closed = false;
    this._polling = false;
    this._recvInterval = null;
  }

  _ensureFd() {
    if (this._fd !== null) return;
    if (typeof globalThis.__koss_udp_create !== 'function') {
      throw new Error('UDP capability not available (NET_UDP not granted)');
    }
    this._fd = globalThis.__koss_udp_create(this.type);
  }

  bind(port, address, callback) {
    if (typeof address === 'function') { callback = address; address = '0.0.0.0'; }
    if (typeof port === 'function') { callback = port; port = 0; }
    this._ensureFd();
    var addr = address || (this.type === 'udp6' ? '::' : '0.0.0.0');
    try {
      globalThis.__koss_udp_bind(this._fd, String(addr), Number(port) || 0);
    } catch (err) {
      if (callback) return nextTick(() => callback(err));
      throw err;
    }
    this._bound = true;
    this._startPolling();
    if (callback) nextTick(() => callback(null));
    this.emit('listening');
    return this;
  }

  _startPolling() {
    if (this._polling || this._closed) return;
    this._polling = true;
    var self = this;
    this._recvInterval = setInterval(function() {
      if (self._closed || typeof globalThis.__koss_udp_recv !== 'function') return;
      try {
        var result = globalThis.__koss_udp_recv(self._fd, 65536);
        if (result && result.code === 0 && result.value) {
          var bytes = _b64dec(result.value);
          var fromParts = (result.from || '').split(':');
          var rinfo = {
            address: fromParts.slice(0, fromParts.length - 1).join(':'),
            family: fromParts[0] && fromParts[0].indexOf(':') !== -1 || fromParts.length > 2 ? 'IPv6' : 'IPv4',
            port: parseInt(fromParts[fromParts.length - 1], 10) || 0,
            size: bytes.length,
          };
          self.emit('message', bytes, rinfo);
        }
      } catch (e) {
        self.emit('error', e);
      }
    }, 10);
  }

  send(msg, offset, length, port, address, callback) {
    if (typeof address === 'function') { callback = address; address = undefined; }
    if (typeof port === 'function') { callback = port; port = undefined; }
    if (typeof offset === 'function') { callback = offset; offset = 0; }
    if (typeof length === 'function') { callback = length; length = undefined; }

    var bytes = _toBytes(msg);
    var off = offset || 0;
    var len = length !== undefined ? length : bytes.length - off;
    var slice = bytes.subarray(off, off + len);
    var target = address || (this.type === 'udp6' ? '::1' : '127.0.0.1');

    this._ensureFd();
    try {
      var result = globalThis.__koss_udp_send(this._fd, _b64c(slice), true, String(target), Number(port) || 0);
      if (result && result.code !== 0) {
        throw new Error('UDP send failed: ' + (result.value || ''));
      }
      if (callback) nextTick(() => callback(null, len));
    } catch (err) {
      if (callback) return nextTick(() => callback(err));
      throw err;
    }
    return this;
  }

  close(callback) {
    if (this._closed) return this;
    this._closed = true;
    if (this._recvInterval) { clearInterval(this._recvInterval); this._recvInterval = null; }
    this._polling = false;
    if (this._fd !== null && typeof globalThis.__koss_udp_close === 'function') {
      try { globalThis.__koss_udp_close(this._fd); } catch (e) {}
      this._fd = null;
    }
    this.emit('close');
    if (callback) nextTick(callback);
    return this;
  }

  address() {
    this._ensureFd();
    var addrStr = globalThis.__koss_udp_address(this._fd);
    if (!addrStr) return { address: '0.0.0.0', family: this.type === 'udp4' ? 'IPv4' : 'IPv6', port: 0 };
    var parts = addrStr.split(':');
    var port = parseInt(parts[parts.length - 1], 10) || 0;
    var address = parts.slice(0, parts.length - 1).join(':');
    return {
      address: address,
      family: address.indexOf(':') !== -1 || parts.length > 2 ? 'IPv6' : 'IPv4',
      port: port,
    };
  }

  setBroadcast(flag) { return this; }
  setTTL(ttl) { return this; }
  setMulticastTTL(ttl) { return this; }
  setMulticastLoopback(flag) { return this; }
  addMembership(multicastInterface) { return this; }
  dropMembership(multicastInterface) { return this; }

  ref() { return this; }
  unref() { return this; }
}

function createSocket(options, callback) {
  if (typeof options === 'string') options = { type: options };
  if (typeof options === 'function') { callback = options; options = {}; }
  return new Socket(options);
}

module.exports = { Socket, createSocket };
