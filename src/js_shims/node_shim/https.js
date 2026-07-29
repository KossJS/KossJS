// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/https - Node.js https module (L3)
// Maps to koss:http standard library (HTTPS is HTTP over TLS)

var http = require('koss:http');
var net = require('koss:net');

class TLSSocket extends net.Socket {
  constructor(options) {
    super(options);
    this._authorized = options?.rejectUnauthorized !== false;
  }
  get encrypted() { return true; }
  get authorized() { return this._authorized; }
  get authorizationError() { return this._authorized ? null : new Error('TLS authorization failed'); }
  get alpnProtocol() { return 'http/1.1'; }
}

class Server extends http.Server {
  constructor(options, requestListener) {
    super(options, requestListener);
  }
}

function createServer(options, requestListener) {
  return new Server(options, requestListener);
}

function request(url, options, callback) {
  return http.request(url, options, callback);
}

function get(url, options, callback) {
  return http.get(url, options, callback);
}

const globalAgent = { maxSockets: Infinity };

module.exports = { createServer, request, get, Server, TLSSocket, globalAgent };
