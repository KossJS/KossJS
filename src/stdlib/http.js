'use strict';

var net = require('net');

function sliceLines(str) {
  var lines = [];
  var start = 0;
  for (var i = 0; i < str.length; i++) {
    if (str[i] === '\r' && str[i + 1] === '\n') {
      lines.push(str.substring(start, i));
      i++;
      start = i + 1;
    } else if (str[i] === '\n') {
      lines.push(str.substring(start, i));
      start = i + 1;
    }
  }
  if (start < str.length) lines.push(str.substring(start));
  return lines;
}

function IncomingMessage(socket) {
  this.socket = socket;
  this.headers = {};
  this.url = '';
  this.method = '';
  this.statusCode = 200;
  this.statusMessage = 'OK';
  this.httpVersion = '1.1';
  this._body = '';
  this._buffer = '';
  this._parsed = false;
  this.complete = false;
}
IncomingMessage.prototype._parse = function(data) {
  this._buffer += data;
  var idx = this._buffer.indexOf('\r\n\r\n');
  if (idx === -1) idx = this._buffer.indexOf('\n\n');
  if (idx === -1) return false;

  var headerEnd = idx;
  if (this._buffer[headerEnd] === '\r') headerEnd += 4;
  else headerEnd += 2;

  var headerPart = this._buffer.substring(0, headerEnd - (this._buffer[headerEnd - 1] === '\n' ? (this._buffer[headerEnd - 2] === '\r' ? 2 : 1) : 0));
  var lines = sliceLines(headerPart);
  if (lines.length === 0) return false;

  var first = lines[0].split(' ');
  if (first.length >= 3 && first[0].indexOf('HTTP') === 0) {
    this.statusCode = parseInt(first[1], 10) || 200;
    this.statusMessage = first.slice(2).join(' ');
  } else if (first.length >= 2) {
    this.method = first[0];
    this.url = first[1];
  }

  for (var i = 1; i < lines.length; i++) {
    var colon = lines[i].indexOf(':');
    if (colon > 0) {
      var key = lines[i].substring(0, colon).trim().toLowerCase();
      var val = lines[i].substring(colon + 1).trim();
      this.headers[key] = val;
    }
  }

  this._body = this._buffer.substring(headerEnd);
  this._parsed = true;
  return true;
};

function ServerResponse(socket) {
  this.socket = socket;
  this.statusCode = 200;
  this.statusMessage = 'OK';
  this._headers = {};
  this._headerSent = false;
  this._chunked = true;
  this.finished = false;
}
ServerResponse.prototype.setHeader = function(name, value) {
  this._headers[name.toLowerCase()] = value;
};
ServerResponse.prototype.getHeader = function(name) {
  return this._headers[name.toLowerCase()];
};
ServerResponse.prototype.removeHeader = function(name) {
  delete this._headers[name.toLowerCase()];
};
ServerResponse.prototype.writeHead = function(code, reason, headers) {
  if (typeof reason === 'object') { headers = reason; reason = undefined; }
  this.statusCode = code || this.statusCode;
  if (reason) this.statusMessage = reason;
  if (headers) { for (var k in headers) this._headers[k] = headers[k]; }
  this._sendHead();
};
ServerResponse.prototype._sendHead = function() {
  if (this._headerSent) return;
  this._headerSent = true;
  var head = 'HTTP/1.1 ' + this.statusCode + ' ' + (this.statusMessage || 'OK') + '\r\n';
  for (var k in this._headers) {
    head += k + ': ' + this._headers[k] + '\r\n';
  }
  head += '\r\n';
  this.socket.write(head);
};
ServerResponse.prototype.write = function(data) {
  this._sendHead();
  this.socket.write(data);
};
ServerResponse.prototype.end = function(data) {
  if (data !== undefined) this.write(data);
  this._sendHead();
  this.finished = true;
  this.socket.end();
};
ServerResponse.prototype.addTrailers = function() {};

function Server(requestListener) {
  var self = this;
  this._server = net.createServer(function(socket) {
    var req = new IncomingMessage(socket);
    var res = new ServerResponse(socket);
    var buf = '';

    socket.on('data', function(data) {
      buf += data;
      if (!req._parsed) {
        if (!req._parse(buf)) return;
        buf = req._body;
      }
      if (requestListener) requestListener(req, res);
    });

    socket.on('close', function() { req.complete = true; });
  });
}
Server.prototype.listen = function(port, host, cb) {
  return this._server.listen(port, host, cb);
};
Server.prototype.close = function(cb) {
  return this._server.close(cb);
};
Server.prototype.address = function() {
  return this._server.address();
};

function createServer(requestListener) {
  return new Server(requestListener);
}

function ClientRequest() {
  throw new Error('http.ClientRequest not yet implemented in KossJS shim');
}

function get(url, cb) {
  throw new Error('http.get not yet implemented in KossJS shim');
}

function request() {
  throw new Error('http.request not yet implemented in KossJS shim');
}

module.exports = {
  Server: Server,
  createServer: createServer,
  ClientRequest: ClientRequest,
  get: get,
  request: request,
  IncomingMessage: IncomingMessage,
  ServerResponse: ServerResponse,
  METHODS: ['GET', 'POST', 'PUT', 'DELETE', 'HEAD', 'OPTIONS', 'PATCH'],
  STATUS_CODES: {
    200: 'OK', 201: 'Created', 204: 'No Content',
    301: 'Moved Permanently', 302: 'Found', 304: 'Not Modified',
    400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden',
    404: 'Not Found', 500: 'Internal Server Error', 502: 'Bad Gateway',
    503: 'Service Unavailable',
  },
};
