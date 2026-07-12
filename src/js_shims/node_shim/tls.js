// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/tls - Node.js tls module (L3)

const _net = require('koss:node/net');
const Socket = _net.Socket;

class TLSSocket extends Socket {
  constructor(options) {
    super(options);
    this._authorized = options?.rejectUnauthorized !== false;
  }
  get encrypted() { return true; }
  get authorized() { return this._authorized; }
  get authorizationError() { return this._authorized ? null : new Error('TLS authorization failed'); }
  get alpnProtocol() { return 'http/1.1'; }
}

class Server {
  constructor(options, connectionListener) {
    if (typeof options === 'function') { connectionListener = options; options = {}; }
    this._connectionListener = connectionListener;
  }

  listen(port, host, callback) {
    if (typeof host === 'function') { callback = host; host = '0.0.0.0'; }
    if (callback) process.nextTick(callback);
    return this;
  }

  close(callback) { if (callback) process.nextTick(callback); return this; }
}

function connect(options, callback) {
  const socket = new TLSSocket(typeof options === 'object' ? options : {});
  if (callback) process.nextTick(() => callback(null, socket));
  return socket;
}

function createServer(options, connectionListener) {
  return new Server(options, connectionListener);
}

function createSecureContext(options) {
  return { context: {}, alpnProtocols: ['http/1.1'] };
}

function checkServerIdentity(hostname, cert) {
  return undefined;
}

const rootCertificates = [];

module.exports = { TLSSocket, Server, connect, createServer, createSecureContext, checkServerIdentity, rootCertificates };