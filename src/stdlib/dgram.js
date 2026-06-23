'use strict';

var EventEmitter = require('events');
var Buffer = require('buffer').Buffer;

var _sockets = {};
var _nextFd = 1;

function Socket(type, reuseAddr) {
  if (!(this instanceof Socket)) return new Socket(type, reuseAddr);
  EventEmitter.call(this);
  this.type = type;
  this._fd = _nextFd++;
  this._bound = false;
  this._closed = false;
  this._recvTimer = null;
  _sockets[this._fd] = this;
}
Socket.prototype = Object.create(EventEmitter.prototype, { constructor: { value: Socket } });

Socket.prototype.bind = function(port, address, callback) {
  if (typeof address === 'function') { callback = address; address = '0.0.0.0'; }
  if (typeof port === 'function') { callback = port; port = 0; address = '0.0.0.0'; }
  if (typeof port === 'object' && port !== null) {
    var opts = port;
    port = opts.port || 0;
    address = opts.address || '0.0.0.0';
  }
  if (typeof address !== 'string') address = '0.0.0.0';
  if (callback) this.once('listening', callback);

  var self = this;
  try {
    var result = __koss_tcp_listen(address, port || 0);
    if (typeof result === 'number' && result > 0) {
      __koss_tcp_close(result);
    }
    self._bound = true;
    self._port = port || 0;
    self._address = address;
    process.nextTick(function() {
      self.emit('listening');
      self._startRecv();
    });
  } catch (e) {
    process.nextTick(function() { self.emit('error', e); });
  }
  return this;
};

Socket.prototype._startRecv = function() {
  if (this._closed || !this._bound) return;
};

Socket.prototype.close = function(callback) {
  this._closed = true;
  if (this._recvTimer) { clearTimeout(this._recvTimer); this._recvTimer = null; }
  delete _sockets[this._fd];
  if (callback) this.once('close', callback);
  var self = this;
  process.nextTick(function() { self.emit('close'); });
};

Socket.prototype.address = function() {
  return { address: this._address || '0.0.0.0', port: this._port || 0, family: 'IPv4' };
};

Socket.prototype.send = function(msg, offset, length, port, address, callback) {
  if (typeof offset === 'function') { callback = offset; offset = 0; }
  if (typeof length === 'function') { callback = length; length = 0; }
  if (typeof port === 'function') { callback = port; port = 0; address = '127.0.0.1'; }
  if (typeof address === 'function') { callback = address; address = '127.0.0.1'; }
  if (typeof length !== 'number' || length === 0) length = msg.length - (offset || 0);

  var self = this;
  try {
    var buf = Buffer.isBuffer(msg) ? msg : Buffer.from(msg);
    var data = buf.slice(offset || 0, (offset || 0) + length);
    var fd = __koss_tcp_connect(address, port || 0);
    if (typeof fd === 'number') {
      __koss_tcp_write(fd, data.toString());
      __koss_tcp_close(fd);
    }
    if (callback) process.nextTick(function() { callback(null, length); });
    return length;
  } catch (e) {
    if (callback) process.nextTick(function() { callback(e); });
    return 0;
  }
};

Socket.prototype.setMulticastLoopback = function(flag) { return this; };
Socket.prototype.setMulticastTTL = function(ttl) { return this; };
Socket.prototype.setBroadcast = function(flag) { return this; };
Socket.prototype.addMembership = function(multicastAddress, multicastInterface) { return this; };
Socket.prototype.dropMembership = function(multicastAddress, multicastInterface) { return this; };
Socket.prototype.unref = function() { return this; };
Socket.prototype.ref = function() { return this; };
Socket.prototype.setTTL = function(ttl) { return this; };

function createSocket(type, callback) {
  if (typeof type === 'object' && type !== null) {
    var opts = type;
    type = opts.type || 'udp4';
  }
  if (typeof type !== 'string') type = 'udp4';
  var sock = new Socket(type);
  if (callback) sock.on('message', callback);
  return sock;
}

module.exports = {
  Socket: Socket,
  createSocket: createSocket,
};
