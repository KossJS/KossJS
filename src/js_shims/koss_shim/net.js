// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:net — Koss 原生 TCP 网络标准库
// Socket、Server、createServer、createConnection

var io = require('koss:io');
var EventEmitter = require('koss:events');

// ── IP 工具函数 ──

function isIP(input) {
  if (typeof input !== 'string') return 0;
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(input)) {
    var parts = input.split('.');
    for (var i = 0; i < parts.length; i++) {
      var n = Number(parts[i]);
      if (n < 0 || n > 255 || parts[i] === '') return 0;
    }
    return 4;
  }
  if (/^[0-9a-fA-F:]+$/.test(input)) {
    var count = input.split(':').length;
    if (count >= 3 && count <= 8) return 6;
  }
  return 0;
}

function isIPv4(input) { return isIP(input) === 4; }
function isIPv6(input) { return isIP(input) === 6; }

function _detectFamily(host) {
  if (!host) return 'IPv4';
  var fam = isIP(host);
  if (fam === 6) return 'IPv6';
  return 'IPv4';
}

// ── Socket ──

var Socket = (function() {
  function Socket(options) {
    EventEmitter.call(this);
    this._fd = null;
    this._connected = false;
    this._destroyed = false;
    this._buffer = [];
    this._readInterval = null;
    this.timeout = 0;
    this._timeoutId = null;
    this.remoteAddress = '';
    this.remotePort = 0;
    this.localAddress = '';
    this.localPort = 0;
    this.family = 'IPv4';
  }
  Socket.prototype = Object.create(EventEmitter.prototype);
  Socket.prototype.constructor = Socket;

  Socket.prototype.connect = function(port, host, connectListener) {
    if (typeof host === 'function') { connectListener = host; host = 'localhost'; }
    if (typeof port === 'object' && port !== null) {
      var opts = port;
      port = opts.port;
      host = opts.host || opts.hostname || 'localhost';
    }
    if (connectListener) this.once('connect', connectListener);

    host = host || 'localhost';
    this.family = _detectFamily(host);

    var self = this;
    try {
      var sock = io.connect(host, Number(port));
      self._fd = sock._fd;
      self._connected = true;
      self.remoteAddress = host;
      self.remotePort = Number(port);
      self.localAddress = '0.0.0.0';
      self.localPort = 0;
      self.emit('connect');
      self._startReading();
    } catch (err) {
      self.emit('error', err);
    }
    return this;
  };

  Socket.prototype._startReading = function() {
    var self = this;
    if (self._readInterval) return;
    self._readInterval = setInterval(function() {
      if (!self._connected || self._destroyed) return;
      try {
        if (typeof globalThis.__koss_tcp_read === 'function' && self._fd !== null) {
          var data = globalThis.__koss_tcp_read(self._fd);
          if (data !== undefined && data !== null) {
            if (typeof data === 'string') {
              var bytes = new Uint8Array(data.length);
              for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
              self.emit('data', bytes);
            } else if (data instanceof Uint8Array && data.length > 0) {
              self.emit('data', data);
            }
          }
        }
      } catch (e) {
        if (e && e.message && e.message.indexOf('closed') !== -1) {
          self._cleanup();
          self.emit('end');
          self.emit('close');
        }
      }
    }, 10);
  };

  Socket.prototype._cleanup = function() {
    if (this._readInterval) {
      clearInterval(this._readInterval);
      this._readInterval = null;
    }
    if (this._timeoutId) {
      clearTimeout(this._timeoutId);
      this._timeoutId = null;
    }
    this._connected = false;
  };

  Socket.prototype.write = function(data, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    if (this._destroyed) {
      if (callback) callback(new Error('Socket is destroyed'));
      return false;
    }
    try {
      var str;
      if (typeof data === 'string') {
        str = data;
      } else if (data instanceof Uint8Array) {
        str = '';
        for (var i = 0; i < data.length; i++) str += String.fromCharCode(data[i]);
      } else {
        str = String(data);
      }
      if (this._fd !== null && typeof globalThis.__koss_tcp_write === 'function') {
        globalThis.__koss_tcp_write(this._fd, str);
      } else {
        this._buffer.push(str);
      }
      if (callback) callback();
    } catch (err) {
      if (callback) callback(err);
      else this.emit('error', err);
    }
    return true;
  };

  Socket.prototype.end = function(data, encoding, callback) {
    if (typeof data === 'function') { callback = data; data = undefined; encoding = undefined; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (data !== undefined && data !== null) this.write(data, encoding);
    this.destroy();
    if (callback) callback();
    return this;
  };

  Socket.prototype.destroy = function(err) {
    if (this._destroyed) return this;
    this._destroyed = true;
    this._cleanup();
    if (this._fd !== null && typeof globalThis.__koss_tcp_close === 'function') {
      try { globalThis.__koss_tcp_close(this._fd); } catch (e) { /* ignore */ }
    }
    if (err) this.emit('error', err);
    this.emit('close');
    this._connected = false;
    return this;
  };

  Socket.prototype.setTimeout = function(ms, callback) {
    this.timeout = Number(ms) || 0;
    if (this._timeoutId) { clearTimeout(this._timeoutId); this._timeoutId = null; }
    if (callback) this.once('timeout', callback);
    if (this.timeout > 0 && this._connected) {
      var self = this;
      this._timeoutId = setTimeout(function() {
        self._timeoutId = null;
        self.emit('timeout');
      }, this.timeout);
    }
    return this;
  };

  Socket.prototype.setEncoding = function(encoding) { return this; };
  Socket.prototype.setKeepAlive = function(enable, initialDelay) { return this; };
  Socket.prototype.setNoDelay = function(noDelay) { return this; };
  Socket.prototype.ref = function() { return this; };
  Socket.prototype.unref = function() { return this; };

  Socket.prototype.address = function() {
    return {
      address: this.localAddress || '127.0.0.1',
      port: this.localPort || 0,
      family: this.family || 'IPv4',
    };
  };

  Object.defineProperty(Socket.prototype, 'readable', {
    get: function() { return this._connected && !this._destroyed; }
  });
  Object.defineProperty(Socket.prototype, 'writable', {
    get: function() { return this._connected && !this._destroyed; }
  });
  Object.defineProperty(Socket.prototype, 'pending', {
    get: function() { return !this._connected; }
  });
  Object.defineProperty(Socket.prototype, 'connecting', {
    get: function() { return false; }
  });
  Object.defineProperty(Socket.prototype, 'destroyed', {
    get: function() { return this._destroyed; }
  });

  return Socket;
})();

// ── Server ──

var Server = (function() {
  function Server(options, connectionListener) {
    EventEmitter.call(this);
    if (typeof options === 'function') {
      connectionListener = options;
      options = {};
    }
    if (connectionListener) this.on('connection', connectionListener);
    this._server = null;
    this._listening = false;
    this._pollInterval = null;
    this._connections = 0;
    this._maxConnections = 0;
  }
  Server.prototype = Object.create(EventEmitter.prototype);
  Server.prototype.constructor = Server;

  Server.prototype.listen = function(port, host, backlog, callback) {
    if (typeof port === 'object' && port !== null) {
      var opts = port;
      port = opts.port;
      host = opts.host || opts.hostname;
      backlog = opts.backlog;
    }
    if (typeof host === 'function') { callback = host; host = '0.0.0.0'; }
    if (typeof backlog === 'function') { callback = backlog; backlog = undefined; }
    if (callback) this.on('listening', callback);

    host = host || '0.0.0.0';

    var self = this;
    try {
      self._server = io.serve({ port: Number(port) || 0, hostname: host });
      self._listening = true;
      self.emit('listening');
      self._pollInterval = setInterval(function() {
        if (self._server) {
          var client = self._server.accept();
          if (client) {
            var socket = new Socket();
            socket._fd = client._fd;
            socket._connected = true;
            socket.remoteAddress = '127.0.0.1';
            socket.remotePort = 0;
            socket.localAddress = host;
            socket.localPort = self._server.port || 0;
            socket.family = _detectFamily(host);
            self._connections++;
            socket._startReading();
            self.emit('connection', socket);
          }
        }
      }, 50);
    } catch (err) {
      self.emit('error', err);
    }
    return this;
  };

  Server.prototype.close = function(callback) {
    if (callback) this.once('close', callback);
    if (this._pollInterval) {
      clearInterval(this._pollInterval);
      this._pollInterval = null;
    }
    if (this._server) {
      try { this._server.close(); } catch (e) { /* ignore */ }
    }
    this._listening = false;
    this._connections = 0;
    this.emit('close');
    return this;
  };

  Server.prototype.address = function() {
    if (!this._server) return null;
    return {
      address: this._server.hostname || '0.0.0.0',
      port: this._server.port || 0,
      family: _detectFamily(this._server.hostname),
    };
  };

  Object.defineProperty(Server.prototype, 'listening', {
    get: function() { return this._listening; }
  });
  Object.defineProperty(Server.prototype, 'connections', {
    get: function() { return this._connections; },
    set: function(val) { this._connections = Number(val) || 0; }
  });
  Object.defineProperty(Server.prototype, 'maxConnections', {
    get: function() { return this._maxConnections; },
    set: function(val) { this._maxConnections = Number(val) || 0; }
  });

  Server.prototype.ref = function() { return this; };
  Server.prototype.unref = function() { return this; };

  return Server;
})();

// ── 工厂函数 ──

function createServer(options, connectionListener) {
  return new Server(options, connectionListener);
}

function connect(port, host, connectListener) {
  var socket = new Socket();
  socket.connect(port, host, connectListener);
  return socket;
}

function createConnection(port, host, connectListener) {
  return connect(port, host, connectListener);
}

module.exports = {
  createServer: createServer,
  connect: connect,
  createConnection: createConnection,
  Socket: Socket,
  Server: Server,
  isIP: isIP,
  isIPv4: isIPv4,
  isIPv6: isIPv6,
};
