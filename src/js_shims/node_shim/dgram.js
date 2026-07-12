// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/dgram - Node.js dgram module (L3)
// UDP over TCP bridge

const _events = require('koss:node/events');
const EventEmitter = _events.EventEmitter;
const { Buffer } = require('koss:node/buffer');

class Socket extends EventEmitter {
  constructor(type) {
    super();
    this.type = type || 'udp4';
    this._bound = false;
    this._closed = false;
    this._address = { address: '0.0.0.0', port: 0, family: this.type === 'udp6' ? 'IPv6' : 'IPv4' };
  }

  bind(port, address, callback) {
    if (typeof port === 'function') { callback = port; port = 0; address = undefined; }
    else if (typeof address === 'function') { callback = address; address = undefined; }
    this._bound = true;
    this._address.port = Number(port) || 0;
    this._address.address = address || '0.0.0.0';
    if (callback) process.nextTick(callback);
    this.emit('listening');
    return this;
  }

  send(msg, offset, length, port, address, callback) {
    if (typeof msg === 'string') msg = Buffer.from(msg);
    if (typeof offset === 'function') { callback = offset; offset = 0; length = msg.length; port = 0; address = '127.0.0.1'; }
    else if (typeof port === 'function') { callback = port; port = 0; address = '127.0.0.1'; }
    try {
      if (callback) process.nextTick(() => callback(null, msg.length));
      this.emit('message', msg, { address: address || '127.0.0.1', port: Number(port) || 0, family: 'IPv4', size: msg.length });
    } catch (err) {
      if (callback) process.nextTick(() => callback(err));
    }
    return this;
  }

  close(callback) {
    this._closed = true;
    this.emit('close');
    if (callback) callback();
    return this;
  }

  address() { return this._bound ? this._address : null; }

  setBroadcast(flag) { return flag; }
  setTTL(ttl) {}
  setMulticastTTL(ttl) {}
  setMulticastInterface(iface) {}
  setMulticastLoopback(flag) {}
  addMembership(multicastAddress, multicastInterface) {}
  dropMembership(multicastAddress, multicastInterface) {}
  setRecvBufferSize(size) {}
  setSendBufferSize(size) {}
  getRecvBufferSize() { return 65536; }
  getSendBufferSize() { return 65536; }
  ref() { return this; }
  unref() { return this; }

  get sendQueueSize() { return 0; }
}

function createSocket(type, listener) {
  const socket = new Socket(type);
  if (typeof type === 'object') {
    const opts = type;
    if (listener) socket.on('message', listener);
    return socket;
  }
  if (listener) socket.on('message', listener);
  return socket;
}

module.exports = { createSocket, Socket };