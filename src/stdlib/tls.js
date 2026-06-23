'use strict';

var net = require('net');
var timers = require('timers');

function TLSSocket(socket, options) {
  if (!(this instanceof TLSSocket)) return new TLSSocket(socket, options);
  net.Socket.call(this);
  this._tlsOptions = options || {};
  if (socket) {
    this._fd = socket._fd;
    this.remoteAddress = socket.remoteAddress;
    this.remotePort = socket.remotePort;
    this.encrypted = true;
  }
}
TLSSocket.prototype = Object.create(net.Socket.prototype, { constructor: { value: TLSSocket } });

function connect(options, callback) {
  var port, host, cb;
  if (typeof options === 'object') {
    port = options.port;
    host = options.host || '127.0.0.1';
    cb = options.callback || callback;
  } else {
    port = arguments[0];
    host = arguments[1] || '127.0.0.1';
    cb = callback;
  }
  var socket = net.connect(port, host, function() {
    if (cb) cb(null, socket);
  });
  socket.encrypted = false;
  return socket;
}

function createServer(options, secureConnectionListener) {
  var server = net.createServer(function(socket) {
    socket.encrypted = true;
    if (secureConnectionListener) secureConnectionListener(socket);
  });
  return server;
}

function createSecureContext(options) {
  return { context: {} };
}

function checkServerIdentity(host, cert) {
  return undefined;
}

module.exports = {
  TLSSocket: TLSSocket,
  connect: connect,
  createServer: createServer,
  createSecureContext: createSecureContext,
  checkServerIdentity: checkServerIdentity,
};
