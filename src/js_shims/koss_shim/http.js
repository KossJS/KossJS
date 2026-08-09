// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:http — Koss 原生 HTTP 客户端/服务端标准库
// 兼容 Node.js http 模块 API，基于 koss:net TCP 网络层

var EventEmitter = require('koss:events').EventEmitter;
var net = require('koss:net');
var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;

// ═══════════════════════════════════════════
// HTTP 状态码与状态文本
// ═══════════════════════════════════════════

var STATUS_CODES = {
  100: 'Continue',
  101: 'Switching Protocols',
  102: 'Processing',
  103: 'Early Hints',
  200: 'OK',
  201: 'Created',
  202: 'Accepted',
  203: 'Non-Authoritative Information',
  204: 'No Content',
  205: 'Reset Content',
  206: 'Partial Content',
  207: 'Multi-Status',
  208: 'Already Reported',
  226: 'IM Used',
  300: 'Multiple Choices',
  301: 'Moved Permanently',
  302: 'Found',
  303: 'See Other',
  304: 'Not Modified',
  305: 'Use Proxy',
  307: 'Temporary Redirect',
  308: 'Permanent Redirect',
  400: 'Bad Request',
  401: 'Unauthorized',
  402: 'Payment Required',
  403: 'Forbidden',
  404: 'Not Found',
  405: 'Method Not Allowed',
  406: 'Not Acceptable',
  407: 'Proxy Authentication Required',
  408: 'Request Timeout',
  409: 'Conflict',
  410: 'Gone',
  411: 'Length Required',
  412: 'Precondition Failed',
  413: 'Payload Too Large',
  414: 'URI Too Long',
  415: 'Unsupported Media Type',
  416: 'Range Not Satisfiable',
  417: 'Expectation Failed',
  418: "I'm a Teapot",
  421: 'Misdirected Request',
  422: 'Unprocessable Entity',
  423: 'Locked',
  424: 'Failed Dependency',
  425: 'Too Early',
  426: 'Upgrade Required',
  428: 'Precondition Required',
  429: 'Too Many Requests',
  431: 'Request Header Fields Too Large',
  451: 'Unavailable For Legal Reasons',
  500: 'Internal Server Error',
  501: 'Not Implemented',
  502: 'Bad Gateway',
  503: 'Service Unavailable',
  504: 'Gateway Timeout',
  505: 'HTTP Version Not Supported',
  506: 'Variant Also Negotiates',
  507: 'Insufficient Storage',
  508: 'Loop Detected',
  510: 'Not Extended',
  511: 'Network Authentication Required',
};

// ═══════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════

function _encodeUtf8(str) {
  var bytes = [];
  for (var i = 0; i < str.length; i++) {
    var c = str.charCodeAt(i);
    if (c < 0x80) {
      bytes.push(c);
    } else if (c < 0x800) {
      bytes.push(0xc0 | (c >> 6));
      bytes.push(0x80 | (c & 0x3f));
    } else if (c >= 0xd800 && c <= 0xdbff) {
      var hi = c;
      var lo = str.charCodeAt(++i);
      c = ((hi - 0xd800) * 0x400) + (lo - 0xdc00) + 0x10000;
      bytes.push(0xf0 | (c >> 18));
      bytes.push(0x80 | ((c >> 12) & 0x3f));
      bytes.push(0x80 | ((c >> 6) & 0x3f));
      bytes.push(0x80 | (c & 0x3f));
    } else {
      bytes.push(0xe0 | (c >> 12));
      bytes.push(0x80 | ((c >> 6) & 0x3f));
      bytes.push(0x80 | (c & 0x3f));
    }
  }
  return new Uint8Array(bytes);
}

function _decodeUtf8(bytes) {
  var str = '';
  var i = 0;
  while (i < bytes.length) {
    var b = bytes[i];
    if (b < 0x80) {
      str += String.fromCharCode(b);
      i++;
    } else if ((b & 0xe0) === 0xc0) {
      str += String.fromCharCode(((b & 0x1f) << 6) | (bytes[i + 1] & 0x3f));
      i += 2;
    } else if ((b & 0xf0) === 0xe0) {
      str += String.fromCharCode(((b & 0x0f) << 12) | ((bytes[i + 1] & 0x3f) << 6) | (bytes[i + 2] & 0x3f));
      i += 3;
    } else if ((b & 0xf8) === 0xf0) {
      var cp = ((b & 0x07) << 18) | ((bytes[i + 1] & 0x3f) << 12) | ((bytes[i + 2] & 0x3f) << 6) | (bytes[i + 3] & 0x3f);
      cp -= 0x10000;
      str += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3ff));
      i += 4;
    } else {
      str += String.fromCharCode(b);
      i++;
    }
  }
  return str;
}

function _bufferConcat(arrays) {
  var totalLen = 0;
  for (var i = 0; i < arrays.length; i++) totalLen += arrays[i].length;
  var result = new Uint8Array(totalLen);
  var offset = 0;
  for (var j = 0; j < arrays.length; j++) {
    result.set(arrays[j], offset);
    offset += arrays[j].length;
  }
  return result;
}

function _socketReadLoop(socket, onData, onEnd) {
  var chunks = [];
  function readNext() {
    try {
      var data = socket.read();
      if (data === null || data === undefined) {
        onEnd();
        return;
      }
      var bytes;
      if (typeof data === 'string') {
        bytes = _encodeUtf8(data);
      } else if (data instanceof Uint8Array) {
        bytes = data;
      } else {
        bytes = new Uint8Array(0);
      }
      onData(bytes);
      readNext();
    } catch (e) {
      onEnd();
    }
  }
  readNext();
}

// ═══════════════════════════════════════════
// IncomingMessage
// ═══════════════════════════════════════════

function IncomingMessage(socketOrOpts) {
  EventEmitter.call(this);
  var opts = socketOrOpts || {};
  if (opts && typeof opts.socket !== 'undefined') {
    this.socket = opts.socket;
    this.connection = opts.socket;
  } else if (opts && typeof opts._fd !== 'undefined') {
    this.socket = opts;
    this.connection = opts;
  }
  this.httpVersion = '1.1';
  this.httpVersionMajor = 1;
  this.httpVersionMinor = 1;
  this.complete = false;
  this.headers = {};
  this.trailers = {};
  this.rawHeaders = [];
  this.rawTrailers = [];
  this.method = 'GET';
  this.url = '/';
  this.statusCode = 200;
  this.statusMessage = 'OK';
  this._readableState = { ended: false };
  this._headersParsed = false;
  this._bodyChunks = [];
  this._bodyComplete = false;
}
IncomingMessage.prototype = Object.create(EventEmitter.prototype);
IncomingMessage.prototype.constructor = IncomingMessage;

IncomingMessage.prototype._parseFirstLine = function(line) {
  var parts = line.split(' ');
  if (parts.length >= 3) {
    this.method = parts[0];
    this.url = parts[1];
    this.httpVersion = parts[2].replace('HTTP/', '');
    var vparts = this.httpVersion.split('.');
    this.httpVersionMajor = parseInt(vparts[0], 10) || 1;
    this.httpVersionMinor = parseInt(vparts[1], 10) || 1;
  } else if (parts.length >= 2) {
    this.statusCode = parseInt(parts[1], 10) || 200;
    this.statusMessage = parts.slice(2).join(' ');
  }
};

IncomingMessage.prototype._parseHeaderLine = function(line) {
  var idx = line.indexOf(':');
  if (idx > 0) {
    var key = line.substring(0, idx).trim().toLowerCase();
    var value = line.substring(idx + 1).trim();
    this.headers[key] = value;
    this.rawHeaders.push(line);
  }
};

IncomingMessage.prototype._emitData = function(chunk) {
  this._bodyChunks.push(chunk);
  this.emit('data', chunk);
};

IncomingMessage.prototype._emitEnd = function() {
  this.complete = true;
  this._bodyComplete = true;
  this._readableState.ended = true;
  this.emit('end');
};

IncomingMessage.prototype.read = function() {
  if (this._bodyChunks.length === 0) return null;
  return this._bodyChunks.shift();
};

IncomingMessage.prototype.unshift = function(chunk) {
  this._bodyChunks.unshift(chunk);
};

IncomingMessage.prototype.destroy = function() {
  if (this.socket) {
    try { this.socket.close(); } catch (e) {}
  }
  this.emit('close');
};

Object.defineProperty(IncomingMessage.prototype, 'readable', {
  get: function() { return !this._bodyComplete; },
});

IncomingMessage.prototype.setEncoding = function() { return this; };
IncomingMessage.prototype.pause = function() { return this; };
IncomingMessage.prototype.resume = function() { return this; };
IncomingMessage.prototype.isPaused = function() { return false; };
IncomingMessage.prototype[Symbol.asyncIterator] = function() {
  var self = this;
  var done = false;
  return {
    next: function() {
      if (done) return Promise.resolve({ done: true, value: undefined });
      return new Promise(function(resolve) {
        var chunk = self.read();
        if (chunk) {
          resolve({ done: false, value: chunk });
        } else if (self._bodyComplete) {
          done = true;
          resolve({ done: true, value: undefined });
        } else {
          self.once('data', function(d) {
            resolve({ done: false, value: d });
          });
          self.once('end', function() {
            done = true;
            resolve({ done: true, value: undefined });
          });
        }
      });
    },
  };
};

// ═══════════════════════════════════════════
// ServerResponse
// ═══════════════════════════════════════════

function ServerResponse(socket) {
  EventEmitter.call(this);
  this.socket = socket;
  this.connection = socket;
  this.statusCode = 200;
  this.statusMessage = 'OK';
  this._headers = {};
  this._headersSent = false;
  this._writableState = { ended: false };
  this._finished = false;
}
ServerResponse.prototype = Object.create(EventEmitter.prototype);
ServerResponse.prototype.constructor = ServerResponse;

ServerResponse.prototype.setHeader = function(name, value) {
  if (this._headersSent) throw new Error('Cannot set headers after they are sent');
  this._headers[name.toLowerCase()] = value;
  return this;
};

ServerResponse.prototype.getHeader = function(name) {
  return this._headers[name.toLowerCase()];
};

ServerResponse.prototype.removeHeader = function(name) {
  delete this._headers[name.toLowerCase()];
};

ServerResponse.prototype.writeHead = function(statusCode, statusMessage, headers) {
  if (typeof statusMessage === 'object') { headers = statusMessage; statusMessage = undefined; }
  if (typeof statusCode === 'object') {
    var opts = statusCode;
    statusCode = opts.statusCode || 200;
    statusMessage = opts.statusMessage;
    headers = opts.headers;
  }
  this.statusCode = statusCode;
  if (statusMessage) this.statusMessage = statusMessage;
  if (headers) {
    var keys = Object.keys(headers);
    for (var i = 0; i < keys.length; i++) {
      this._headers[keys[i].toLowerCase()] = headers[keys[i]];
    }
  }
  return this;
};

ServerResponse.prototype._sendHeaders = function() {
  if (this._headersSent) return;
  this._headersSent = true;
  var statusLine = 'HTTP/1.1 ' + this.statusCode + ' ' + (this.statusMessage || STATUS_CODES[this.statusCode] || '') + '\r\n';
  var headerLines = statusLine;
  var keys = Object.keys(this._headers);
  for (var i = 0; i < keys.length; i++) {
    headerLines += keys[i] + ': ' + this._headers[keys[i]] + '\r\n';
  }
  headerLines += '\r\n';
  var encoded = _encodeUtf8(headerLines);
  this._writeRaw(encoded);
};

ServerResponse.prototype._writeRaw = function(data) {
  if (this.socket && this.socket._fd !== null && typeof globalThis.__koss_tcp_write === 'function') {
    globalThis.__koss_tcp_write(this.socket._fd, data);
  }
};

ServerResponse.prototype.write = function(data, encoding, callback) {
  if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (!this._headersSent) this._sendHeaders();
  var encoded;
  if (typeof data === 'string') {
    encoded = _encodeUtf8(data);
  } else if (data instanceof Uint8Array) {
    encoded = data;
  } else {
    encoded = _encodeUtf8(String(data));
  }
  this._writeRaw(encoded);
  if (callback) callback();
  return true;
};

ServerResponse.prototype.end = function(data, encoding, callback) {
  if (typeof data === 'function') { callback = data; data = undefined; }
  else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (data) this.write(data, encoding);
  this._finished = true;
  this._writableState.ended = true;
  this.emit('finish');
  if (callback) callback();
  return this;
};

ServerResponse.prototype.destroy = function() {
  if (this.socket) {
    try { this.socket.close(); } catch (e) {}
  }
  this.emit('close');
};

Object.defineProperty(ServerResponse.prototype, 'writable', {
  get: function() { return !this._finished; },
});

ServerResponse.prototype.setEncoding = function() { return this; };
ServerResponse.prototype.flush = function() {};
ServerResponse.prototype.flushHeaders = function() { this._sendHeaders(); };
ServerResponse.prototype.addTrailers = function() {};

ServerResponse.prototype.setTimeout = function(ms, callback) {
  if (callback) this.on('timeout', callback);
  return this;
};

// ═══════════════════════════════════════════
// ClientRequest
// ═══════════════════════════════════════════

function ClientRequest(options, callback) {
  EventEmitter.call(this);
  var opts = options || {};
  if (typeof opts === 'string') opts = new URL(opts);
  this.method = opts.method || 'GET';
  this.path = opts.path || opts.pathname || '/';
  this.host = opts.hostname || opts.host || 'localhost';
  this.port = Number(opts.port) || 80;
  this.headers = opts.headers || {};
  this._body = opts.body || null;
  this._callback = callback;
  this._response = null;
  this._socket = null;
  this._sent = false;
  this._aborted = false;
}
ClientRequest.prototype = Object.create(EventEmitter.prototype);
ClientRequest.prototype.constructor = ClientRequest;

ClientRequest.prototype.setHeader = function(name, value) {
  this.headers[name.toLowerCase()] = value;
  return this;
};

ClientRequest.prototype.getHeader = function(name) {
  return this.headers[name.toLowerCase()];
};

ClientRequest.prototype.removeHeader = function(name) {
  delete this.headers[name.toLowerCase()];
};

ClientRequest.prototype.write = function(data, encoding, callback) {
  if (!this._body) this._body = '';
  if (typeof data === 'string') {
    this._body += data;
  } else if (data instanceof Uint8Array) {
    this._body = data;
  }
  if (callback) callback();
  return true;
};

ClientRequest.prototype.end = function(data, encoding, callback) {
  if (typeof data === 'function') { callback = data; data = undefined; }
  else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (data) this.write(data, encoding);
  this._sendRequest();
  if (callback) callback();
  return this;
};

ClientRequest.prototype.abort = function() {
  this._aborted = true;
  if (this._socket) {
    try { this._socket.close(); } catch (e) {}
  }
  this.emit('abort');
};

ClientRequest.prototype.setTimeout = function(ms, callback) {
  if (callback) this.on('timeout', callback);
  return this;
};

ClientRequest.prototype._sendRequest = function() {
  if (this._sent) return;
  this._sent = true;

  var self = this;
  var socket;
  try {
    socket = net.connect(this.port, this.host);
    this._socket = socket;
  } catch (err) {
    this.emit('error', err);
    return;
  }

  if (!this.headers['host']) {
    this.headers['host'] = this.port === 80 ? this.host : this.host + ':' + this.port;
  }

  if (this._body && !this.headers['content-length'] && !this.headers['transfer-encoding']) {
    if (typeof this._body === 'string') {
      this.headers['content-length'] = String(_encodeUtf8(this._body).length);
    } else if (this._body instanceof Uint8Array) {
      this.headers['content-length'] = String(this._body.length);
    }
  }

  var requestLine = this.method + ' ' + this.path + ' HTTP/1.1\r\n';
  var headerLines = '';
  var keys = Object.keys(this.headers);
  for (var i = 0; i < keys.length; i++) {
    headerLines += keys[i] + ': ' + this.headers[keys[i]] + '\r\n';
  }
  headerLines += 'connection: close\r\n';
  headerLines += '\r\n';

  var headerData = _encodeUtf8(requestLine + headerLines);
  try {
    if (socket._fd !== null && typeof globalThis.__koss_tcp_write === 'function') {
      globalThis.__koss_tcp_write(socket._fd, headerData);
    }
  } catch (err) {
    this.emit('error', err);
    return;
  }

  if (this._body) {
    var bodyData;
    if (typeof this._body === 'string') {
      bodyData = _encodeUtf8(this._body);
    } else if (this._body instanceof Uint8Array) {
      bodyData = this._body;
    } else {
      bodyData = _encodeUtf8(String(this._body));
    }
    try {
      if (socket._fd !== null && typeof globalThis.__koss_tcp_write === 'function') {
        globalThis.__koss_tcp_write(socket._fd, bodyData);
      }
    } catch (err) {
      this.emit('error', err);
      return;
    }
  }

  var response = new IncomingMessage(socket);
  self._response = response;

  var rawResponse = [];
  _socketReadLoop(socket, function(chunk) {
    rawResponse.push(chunk);
  }, function() {
    self._parseResponse(rawResponse, response);
  });
};

ClientRequest.prototype._parseResponse = function(rawChunks, response) {
  var totalLen = 0;
  for (var i = 0; i < rawChunks.length; i++) totalLen += rawChunks[i].length;
  var data = _bufferConcat(rawChunks);
  var text = _decodeUtf8(data);

  var headerEnd = text.indexOf('\r\n\r\n');
  if (headerEnd === -1) {
    response.statusCode = 502;
    response.statusMessage = 'Bad Gateway';
    this.emit('error', new Error('Invalid HTTP response'));
    return;
  }

  var headerPart = text.substring(0, headerEnd);
  var lines = headerPart.split('\r\n');
  if (lines.length > 0) {
    response._parseFirstLine(lines[0]);
  }
  for (var j = 1; j < lines.length; j++) {
    if (lines[j]) response._parseHeaderLine(lines[j]);
  }

  var bodyStr = text.substring(headerEnd + 4);
  if (bodyStr.length > 0) {
    var bodyBytes = _encodeUtf8(bodyStr);
    response._emitData(bodyBytes);
  }

  if (response.headers['transfer-encoding'] === 'chunked') {
    var decoded = _decodeChunkedBody(bodyStr);
    if (decoded) {
      response._bodyChunks = [];
      response._emitData(_encodeUtf8(decoded));
    }
  }

  response._emitEnd();

  if (this._callback) {
    this._callback(response);
  }
  this.emit('response', response);
};

function _decodeChunkedBody(raw) {
  var result = '';
  var lines = raw.split('\r\n');
  var idx = 0;
  while (idx < lines.length) {
    var sizeStr = lines[idx].trim();
    var size = parseInt(sizeStr, 16);
    if (isNaN(size) || size === 0) break;
    idx++;
    var chunk = '';
    var collected = 0;
    while (collected < size && idx < lines.length) {
      chunk += lines[idx];
      collected += _encodeUtf8(lines[idx]).length;
      idx++;
    }
    result += chunk;
    idx++;
  }
  return result;
}

// ═══════════════════════════════════════════
// Server
// ═══════════════════════════════════════════

function Server(options, requestListener) {
  EventEmitter.call(this);
  if (typeof options === 'function') { requestListener = options; options = {}; }
  this._options = options || {};
  this._requestListener = requestListener || null;
  this._server = null;
  this._listening = false;
  this._connections = 0;
  this._timeout = 0;
  this._keepAliveTimeout = 5000;
}
Server.prototype = Object.create(EventEmitter.prototype);
Server.prototype.constructor = Server;

Server.prototype.listen = function(port, host, backlog, callback) {
  var opts = this._options;
  if (typeof port === 'function') { callback = port; port = undefined; host = undefined; }
  if (typeof host === 'function') { callback = host; host = undefined; }
  if (typeof backlog === 'function') { callback = backlog; backlog = undefined; }
  if (typeof port === 'object' && port !== null) {
    opts = port;
    port = opts.port;
    host = opts.host;
    backlog = opts.backlog;
  }
  port = Number(port) || opts.port || 0;
  host = String(host || opts.host || '0.0.0.0');

  if (callback) this.on('listening', callback);

  var self = this;
  try {
    this._server = net.createServer(host, port);
    this._listening = true;
    this.emit('listening');

    this._pollInterval = setInterval(function() {
      if (self._server) {
        var client;
        try {
          client = self._server.accept();
        } catch (e) {
          return;
        }
        if (client) {
          self._connections++;
          self._handleRequest(client);
        }
      }
    }, 20);
  } catch (err) {
    this.emit('error', err);
  }
  return this;
};

Server.prototype._handleRequest = function(socket) {
  var self = this;
  var rawChunks = [];
  var headerParsed = false;
  var contentLength = 0;
  var receivedBody = 0;

  _socketReadLoop(socket, function(chunk) {
    rawChunks.push(chunk);
  }, function() {
    var data = _bufferConcat(rawChunks);
    if (data.length === 0) {
      try { socket.close(); } catch (e) {}
      self._connections--;
      return;
    }

    var text = _decodeUtf8(data);
    var headerEnd = text.indexOf('\r\n\r\n');
    if (headerEnd === -1) {
      try { socket.close(); } catch (e) {}
      self._connections--;
      return;
    }

    var headerPart = text.substring(0, headerEnd);
    var lines = headerPart.split('\r\n');

    var req = new IncomingMessage(socket);
    var res = new ServerResponse(socket);

    if (lines.length > 0) {
      req._parseFirstLine(lines[0]);
    }
    for (var i = 1; i < lines.length; i++) {
      if (lines[i]) req._parseHeaderLine(lines[i]);
    }

    var bodyPart = text.substring(headerEnd + 4);
    if (bodyPart.length > 0) {
      req._emitData(_encodeUtf8(bodyPart));
    }
    req._emitEnd();

    if (self._requestListener) {
      self._requestListener(req, res);
    }
    self.emit('request', req, res);
  });
};

Server.prototype.close = function(callback) {
  if (callback) this.on('close', callback);
  if (this._pollInterval) clearInterval(this._pollInterval);
  if (this._server) {
    try { this._server.close(); } catch (e) {}
  }
  this._listening = false;
  this._connections = 0;
  this.emit('close');
  return this;
};

Server.prototype.address = function() {
  if (this._server) {
    return { address: '0.0.0.0', port: this._server.port || 0, family: 'IPv4' };
  }
  return null;
};

Server.prototype.ref = function() { return this; };
Server.prototype.unref = function() { return this; };

Object.defineProperty(Server.prototype, 'listening', {
  get: function() { return this._listening; },
});

Object.defineProperty(Server.prototype, 'connections', {
  get: function() { return this._connections; },
  set: function(val) { this._connections = val; },
});

// ═══════════════════════════════════════════
// 高级 API — Agent
// ═══════════════════════════════════════════

function Agent(options) {
  EventEmitter.call(this);
  this._options = options || {};
  this._sockets = {};
  this._maxSockets = Infinity;
  this._maxFreeSockets = 256;
  this._connections = 0;
}
Agent.prototype = Object.create(EventEmitter.prototype);
Agent.prototype.constructor = Agent;

Agent.prototype.createConnection = function(options, callback) {
  var opts = options || {};
  var host = opts.host || opts.hostname || 'localhost';
  var port = Number(opts.port) || 80;
  var socket = net.connect(port, host);
  if (callback) callback();
  return socket;
};

Agent.prototype.destroy = function() {
  var keys = Object.keys(this._sockets);
  for (var i = 0; i < keys.length; i++) {
    var socks = this._sockets[keys[i]];
    for (var j = 0; j < socks.length; j++) {
      try { socks[j].close(); } catch (e) {}
    }
  }
  this._sockets = {};
};

var defaultAgent = new Agent();

// ═══════════════════════════════════════════
// 全局 Agent 管理
// ═══════════════════════════════════════════

var globalAgent = defaultAgent;

// ═══════════════════════════════════════════
// URL 解析
// ═══════════════════════════════════════════

function _parseUrl(urlOrOptions) {
  if (typeof urlOrOptions === 'string') {
    var urlMod = require('koss:url');
    var parsed = urlMod.parse(urlOrOptions);
    return {
      protocol: parsed.protocol,
      hostname: parsed.hostname,
      port: parsed.port ? parseInt(parsed.port, 10) : 80,
      path: parsed.pathname + (parsed.search || ''),
    };
  }
  return {
    protocol: urlOrOptions.protocol || 'http:',
    hostname: urlOrOptions.hostname || urlOrOptions.host || 'localhost',
    port: Number(urlOrOptions.port) || 80,
    path: urlOrOptions.path || urlOrOptions.pathname || '/',
  };
}

// ═══════════════════════════════════════════
// 客户端请求 API
// ═══════════════════════════════════════════

function request(urlOrOptions, optionsOrCallback, callback) {
  var opts, cb;
  if (typeof urlOrOptions === 'string' || (typeof urlOrOptions === 'object' && urlOrOptions && urlOrOptions.href)) {
    var parsed = _parseUrl(urlOrOptions);
    opts = typeof optionsOrCallback === 'object' ? optionsOrCallback : {};
    opts.hostname = parsed.hostname;
    opts.port = parsed.port;
    opts.path = parsed.path;
    opts.protocol = parsed.protocol;
    cb = callback || optionsOrCallback;
    if (typeof cb !== 'function') cb = null;
  } else {
    opts = urlOrOptions || {};
    cb = optionsOrCallback;
    if (typeof cb !== 'function') cb = null;
  }

  var req = new ClientRequest(opts, cb);
  return req;
}

function get(urlOrOptions, optionsOrCallback, callback) {
  var opts, cb;
  if (typeof urlOrOptions === 'string' || (typeof urlOrOptions === 'object' && urlOrOptions && urlOrOptions.href)) {
    var parsed = _parseUrl(urlOrOptions);
    opts = typeof optionsOrCallback === 'object' ? optionsOrCallback : {};
    opts.hostname = parsed.hostname;
    opts.port = parsed.port;
    opts.path = parsed.path;
    opts.protocol = parsed.protocol;
    cb = callback || optionsOrCallback;
    if (typeof cb !== 'function') cb = null;
  } else {
    opts = urlOrOptions || {};
    cb = optionsOrCallback;
    if (typeof cb !== 'function') cb = null;
  }

  opts.method = 'GET';
  var req = new ClientRequest(opts, cb);
  req.end();
  return req;
}

// ═══════════════════════════════════════════
// 服务端创建 API
// ═══════════════════════════════════════════

function createServer(optionsOrListener, listener) {
  var opts, requestListener;
  if (typeof optionsOrListener === 'function') {
    opts = {};
    requestListener = optionsOrListener;
  } else {
    opts = optionsOrListener || {};
    requestListener = listener || null;
  }
  return new Server(opts, requestListener);
}

// ═══════════════════════════════════════════
// 导出
// ═══════════════════════════════════════════

module.exports = {
  request: request,
  get: get,
  createServer: createServer,
  Server: Server,
  ServerResponse: ServerResponse,
  IncomingMessage: IncomingMessage,
  ClientRequest: ClientRequest,
  Agent: Agent,
  globalAgent: globalAgent,
  STATUS_CODES: STATUS_CODES,
  STATUS_CODE: STATUS_CODES,
};
