// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/dgram - Node.js dgram module (L3)

var events = require('koss:events');
var EventEmitter = events.EventEmitter;

class Socket extends EventEmitter {
  constructor(options) {
    super();
    this.type = options?.type || 'udp4';
    this.reuseAddr = options?.reuseAddr || false;
    this.ipv6Only = options?.ipv6Only || false;
    this._fd = null;
    this._bound = false;
    this._closed = false;
  }

  bind(port, address, callback) {
    if (typeof address === 'function') { callback = address; address = '0.0.0.0'; }
    if (typeof port === 'function') { callback = port; port = 0; }
    this._bound = true;
    if (callback) process.nextTick(callback);
    this.emit('listening');
    return this;
  }

  send(msg, offset, length, port, address, callback) {
    if (typeof address === 'function') { callback = address; address = '0.0.0.0'; }
    if (typeof port === 'function') { callback = port; port = 0; }
    if (typeof offset === 'function') { callback = offset; offset = 0; }
    if (typeof length === 'function') { callback = length; length = msg.length; }
    if (callback) process.nextTick(callback);
    return this;
  }

  close(callback) {
    this._closed = true;
    if (callback) process.nextTick(callback);
    this.emit('close');
    return this;
  }

  address() {
    return { address: '0.0.0.0', family: this.type === 'udp4' ? 'IPv4' : 'IPv6', port: 0 };
  }

  setBroadcast(flag) {}
  setTTL(ttl) {}
  setMulticastTTL(ttl) {}
  setMulticastLoopback(flag) {}
  addMembership(multicastInterface) {}
  dropMembership(multicastInterface) {}

  ref() { return this; }
  unref() { return this; }
}

function createSocket(options, callback) {
  if (typeof options === 'string') options = { type: options };
  return new Socket(options);
}

module.exports = { Socket, createSocket };
