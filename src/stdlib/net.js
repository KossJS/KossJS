'use strict';

var EventEmitter = require('events');
var timers = require('timers');
var setTimeout = timers.setTimeout;
var clearTimeout = timers.clearTimeout;

function isIP(input) {
  if (typeof input !== 'string') return 0;
  var parts = input.split('.');
  if (parts.length === 4) {
    for (var i = 0; i < 4; i++) {
      var n = parseInt(parts[i], 10);
      if (isNaN(n) || n < 0 || n > 255) return 0;
    }
    return 4;
  }
  if (input.indexOf(':') !== -1) {
    var v6parts = input.split(':');
    if (v6parts.length >= 2 && v6parts.length <= 8) return 6;
  }
  return 0;
}

function isIPv4(input) { return isIP(input) === 4; }
function isIPv6(input) { return isIP(input) === 6; }

function Socket(options) {
  if (!(this instanceof Socket)) return new Socket(options);
  EventEmitter.call(this);
  this._fd = null;
  this._connecting = false;
  this._destroyed = false;
  this._readPaused = false;
  this._readTimer = null;
  this._writeBuffer = '';
  this._writeCallback = null;
  this.connecting = false;
  this.remoteAddress = undefined;
  this.remotePort = undefined;
  this.localAddress = undefined;
  this.localPort = undefined;
  this.bytesRead = 0;
  this.bytesWritten = 0;
  this.readable = true;
  this.writable = true;
  this.destroyed = false;
}
Socket.prototype = Object.create(EventEmitter.prototype, { constructor: { value: Socket } });

Socket.prototype.connect = function(port, host, connectListener) {
  if (typeof host === 'function') {
    connectListener = host;
    host = '127.0.0.1';
  }
  if (typeof port === 'object' && port !== null) {
    var opts = port;
    port = opts.port;
    host = opts.host || '127.0.0.1';
  }
  if (typeof host !== 'string') host = '127.0.0.1';
  if (connectListener) this.once('connect', connectListener);

  this.connecting = true;
  this._connecting = true;

  var self = this;
  try {
    var fd = __koss_tcp_connect(host, port);
    if (typeof fd === 'number' && fd > 0) {
      self._fd = fd;
      self.connecting = false;
      self._connecting = false;
      self.remoteAddress = host;
      self.remotePort = port;
      self.localAddress = '0.0.0.0';
      self.localPort = 0;
      process.nextTick(function() {
        self.emit('connect');
        self._startRead();
      });
    } else {
      self._connecting = false;
      process.nextTick(function() {
        self.emit('error', new Error('connect ECONNREFUSED ' + host + ':' + port));
      });
    }
  } catch (e) {
    self._connecting = false;
    process.nextTick(function() {
      self.emit('error', e);
    });
  }
  return this;
};

Socket.prototype._startRead = function() {
  if (this._destroyed || !this._fd) return;
  var self = this;
  function poll() {
    if (self._destroyed || !self._fd || self._readPaused) {
      self._readTimer = null;
      return;
    }
    var hadData = false;
    try {
      while (true) {
        var data = __koss_tcp_read(self._fd);
        if (data === null || data === undefined) break;
        self.bytesRead += data.length;
        hadData = true;
        self.emit('data', data);
      }
    } catch (e) {
      self._readTimer = null;
      self.destroy(e);
      return;
    }
    if (!hadData && !self._destroyed && self._fd) {
      self._readTimer = setTimeout(poll, 10);
    } else if (!self._destroyed && self._fd) {
      self._readTimer = setTimeout(poll, 10);
    } else {
      self._readTimer = null;
    }
  }
  self._readTimer = setTimeout(poll, 10);
};

Socket.prototype.pause = function() {
  this._readPaused = true;
  return this;
};

Socket.prototype.resume = function() {
  this._readPaused = false;
  if (this._fd && !this._readTimer) this._startRead();
  return this;
};

Socket.prototype.setEncoding = function(enc) { return this; };
Socket.prototype.setNoDelay = function(v) { return this; };
Socket.prototype.setKeepAlive = function(v, d) { return this; };
Socket.prototype.ref = function() { return this; };
Socket.prototype.unref = function() { return this; };

Socket.prototype.write = function(data, encoding, callback) {
  if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (this._destroyed || !this._fd) {
    if (callback) process.nextTick(callback);
    return false;
  }
  var str = (typeof data === 'string') ? data : data.toString(encoding || 'utf8');
  try {
    var n = __koss_tcp_write(this._fd, str);
    this.bytesWritten += n;
    if (callback) process.nextTick(callback);
  } catch (e) {
    if (callback) process.nextTick(function() { callback(e); });
    return false;
  }
  return true;
};

Socket.prototype.end = function(data, encoding, callback) {
  if (typeof data === 'function') { callback = data; data = undefined; }
  if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (data !== undefined) this.write(data, encoding);
  this.destroy();
  if (callback) this.once('close', callback);
};

Socket.prototype.destroySoon = Socket.prototype.destroy = function(err) {
  if (this._destroyed) return;
  this._destroyed = true;
  this.destroyed = true;
  this.readable = false;
  this.writable = false;
  if (this._readTimer) {
    clearTimeout(this._readTimer);
    this._readTimer = null;
  }
  if (this._fd) {
    try { __koss_tcp_close(this._fd); } catch(e) {}
    this._fd = null;
  }
  if (err) this.emit('error', err);
  this.emit('close', !!err);
};

Socket.prototype.address = function() {
  return { address: this.localAddress || '0.0.0.0', port: this.localPort || 0, family: 'IPv4' };
};

Socket.prototype.setTimeout = function(ms, cb) {
  if (cb) this.once('timeout', cb);
  var self = this;
  setTimeout(function() { self.emit('timeout'); }, ms);
  return this;
};

function Server(options, connectionListener) {
  if (!(this instanceof Server)) return new Server(options, connectionListener);
  EventEmitter.call(this);
  if (typeof options === 'function') {
    connectionListener = options;
    options = {};
  }
  this._options = options || {};
  this._serverFd = null;
  this._listening = false;
  this._acceptTimer = null;
  this._connections = 0;
  if (connectionListener) this.on('connection', connectionListener);
}
Server.prototype = Object.create(EventEmitter.prototype, { constructor: { value: Server } });

Server.prototype.listen = function(port, host, backlog, callback) {
  if (typeof host === 'function') {
    callback = host;
    host = '0.0.0.0';
  }
  if (typeof port === 'object' && port !== null) {
    var opts = port;
    port = opts.port;
    host = opts.host || '0.0.0.0';
  }
  if (typeof host !== 'string') host = '0.0.0.0';
  if (typeof callback === 'function') this.once('listening', callback);

  var self = this;
  try {
    var fd = __koss_tcp_listen(host, port);
    if (typeof fd === 'number' && fd > 0) {
      self._serverFd = fd;
      self._listening = true;
      process.nextTick(function() {
        self.emit('listening');
        self._startAccept();
      });
    } else {
      var err = new Error('listen EADDRINUSE: address already in use :::' + port);
      err.code = 'EADDRINUSE';
      err.errno = -48;
      err.syscall = 'listen';
      process.nextTick(function() { self.emit('error', err); });
    }
  } catch (e) {
    process.nextTick(function() { self.emit('error', e); });
  }
  return this;
};

Server.prototype._startAccept = function() {
  if (!this._listening || !this._serverFd) return;
  var self = this;
  function poll() {
    if (!self._listening || !self._serverFd) {
      self._acceptTimer = null;
      return;
    }
    try {
      while (true) {
        var clientFd = __koss_tcp_accept(self._serverFd);
        if (clientFd === null || clientFd === undefined || typeof clientFd !== 'number') break;
        var sock = new Socket();
        sock._fd = clientFd;
        sock.remoteAddress = '127.0.0.1';
        sock.remotePort = 0;
        self._connections++;
        process.nextTick(function(s, sk) {
          s.emit('connection', sk);
        }, self, sock);
      }
    } catch (e) {
      self._acceptTimer = null;
      self.emit('error', e);
      return;
    }
    self._acceptTimer = setTimeout(poll, 10);
  }
  self._acceptTimer = setTimeout(poll, 10);
};

Server.prototype.close = function(callback) {
  this._listening = false;
  if (this._acceptTimer) {
    clearTimeout(this._acceptTimer);
    this._acceptTimer = null;
  }
  if (this._serverFd) {
    try { __koss_tcp_close(this._serverFd); } catch(e) {}
    this._serverFd = null;
  }
  if (callback) this.once('close', callback);
  var self = this;
  process.nextTick(function() { self.emit('close'); });
};

Server.prototype.getConnections = function(cb) {
  if (cb) process.nextTick(function() { cb(null, this._connections); }.bind(this));
};

Server.prototype.address = function() {
  if (!this._listening) return null;
  return { address: '0.0.0.0', port: 0, family: 'IPv4' };
};
Server.prototype.ref = function() { return this; };
Server.prototype.unref = function() { return this; };

function createServer(options, connectionListener) {
  return new Server(options, connectionListener);
}

function connect(port, host, connectListener) {
  var sock = new Socket();
  sock.connect(port, host, connectListener);
  return sock;
}

function createConnection(port, host, connectListener) {
  return connect(port, host, connectListener);
}

module.exports = {
  Socket: Socket,
  Server: Server,
  createServer: createServer,
  connect: connect,
  createConnection: createConnection,
  isIP: isIP,
  isIPv4: isIPv4,
  isIPv6: isIPv6,
};
