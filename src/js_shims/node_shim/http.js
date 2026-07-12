// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/http - Node.js http module (L3)

const _events = require('koss:node/events');
const EventEmitter = _events.EventEmitter;
const _net = require('koss:node/net');
const NetServer = _net.Server;
const Socket = _net.Socket;

class IncomingMessage extends EventEmitter {
  constructor(socket) {
    super();
    this.socket = socket;
    this.connection = socket;
    this.httpVersion = '1.1';
    this.httpVersionMajor = 1;
    this.httpVersionMinor = 1;
    this.statusCode = 200;
    this.statusMessage = 'OK';
    this.headers = {};
    this.rawHeaders = [];
    this.trailers = {};
    this.method = 'GET';
    this.url = '/';
    this.complete = false;
    this._body = '';
  }

  _addHeaderLine(key, value) { this.headers[key.toLowerCase()] = value; }

  push(chunk) {
    if (chunk === null) { this.complete = true; this.emit('end'); return; }
    this._body += typeof chunk === 'string' ? chunk : String(chunk);
    this.emit('data', chunk);
  }

  setEncoding(enc) { return this; }

  get rawTrailers() { return []; }
}

class ServerResponse extends EventEmitter {
  constructor(req) {
    super();
    this.req = req;
    this.statusCode = 200;
    this.statusMessage = 'OK';
    this._headers = {};
    this._headerSent = false;
    this._body = '';
    this.finished = false;
    this.connection = req?.socket || null;
    this.socket = this.connection;
  }

  setHeader(name, value) { this._headers[name.toLowerCase()] = String(value); }
  getHeader(name) { return this._headers[name.toLowerCase()]; }
  getHeaders() { return { ...this._headers }; }
  removeHeader(name) { delete this._headers[name.toLowerCase()]; }
  hasHeader(name) { return name.toLowerCase() in this._headers; }

  writeHead(statusCode, statusMessage, headers) {
    this.statusCode = statusCode;
    if (typeof statusMessage === 'object') { headers = statusMessage; statusMessage = undefined; }
    if (statusMessage) this.statusMessage = statusMessage;
    if (headers) Object.assign(this._headers, headers);
    this._headerSent = true;
    return this;
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    const str = typeof chunk === 'string' ? chunk : new TextDecoder(encoding || 'utf8').decode(chunk);
    this._body += str;
    if (callback) callback();
    return true;
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === 'function') { callback = chunk; chunk = undefined; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (chunk) this.write(chunk, encoding);
    this.finished = true;
    this.emit('finish');
    if (callback) callback();
    return this;
  }

  get headersSent() { return this._headerSent; }
  get writableEnded() { return this.finished; }
  get writableFinished() { return this.finished; }
}

class Server extends NetServer {
  constructor(options, requestListener) {
    super(options);
    if (typeof options === 'function') { requestListener = options; options = {}; }
    if (requestListener) this.on('request', requestListener);
    this.on('connection', (socket) => {
      const req = new IncomingMessage(socket);
      const res = new ServerResponse(req);
      process.nextTick(() => {
        try { this.emit('request', req, res); }
        catch (err) { res.statusCode = 500; res.end(`Internal Server Error: ${err.message}`); }
      });
    });
  }
}

function createServer(options, requestListener) {
  return new Server(options, requestListener);
}

function request() { throw new Error('http.request (client) not implemented'); }
function get() { throw new Error('http.get (client) not implemented'); }

const METHODS = ['GET', 'HEAD', 'POST', 'PUT', 'DELETE', 'OPTIONS', 'TRACE', 'PATCH', 'CONNECT'];

const STATUS_CODES = {
  100: 'Continue', 101: 'Switching Protocols', 200: 'OK', 201: 'Created', 202: 'Accepted',
  204: 'No Content', 301: 'Moved Permanently', 302: 'Found', 303: 'See Other',
  304: 'Not Modified', 307: 'Temporary Redirect', 400: 'Bad Request', 401: 'Unauthorized',
  403: 'Forbidden', 404: 'Not Found', 405: 'Method Not Allowed', 406: 'Not Acceptable',
  408: 'Request Timeout', 409: 'Conflict', 410: 'Gone', 411: 'Length Required',
  413: 'Payload Too Large', 414: 'URI Too Long', 415: 'Unsupported Media Type',
  429: 'Too Many Requests', 500: 'Internal Server Error', 501: 'Not Implemented',
  502: 'Bad Gateway', 503: 'Service Unavailable', 504: 'Gateway Timeout', 505: 'HTTP Version Not Supported',
};

const maxHeaderSize = 16384;

const globalAgent = {};

module.exports = { createServer, request, get, Server, IncomingMessage, ServerResponse, METHODS, STATUS_CODES, maxHeaderSize, globalAgent };