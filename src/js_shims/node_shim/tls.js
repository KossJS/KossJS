// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/tls - Node.js tls module (L3)
// 注意：KossJS 目前未实现真实 TLS 握手与证书验证。
// 为避免调用方误以为连接已加密（安全隐患），所有 TLS 操作显式抛错。

function _unsupported(name) {
  throw new Error(
    'tls.' + name + ' is not implemented in KossJS. ' +
    'Real TLS (handshake, certificate verification) requires rustls integration ' +
    'which is not yet wired up. Do NOT rely on this module for security.'
  );
}

class TLSSocket {
  constructor(options) { _unsupported('TLSSocket'); }
  get encrypted() { return false; }
  get authorized() { return false; }
  get authorizationError() { return new Error('TLS not implemented in KossJS'); }
  get alpnProtocol() { return undefined; }
}

class Server {
  constructor(options, connectionListener) { _unsupported('Server'); }
  listen() { _unsupported('Server.listen'); }
  close() { _unsupported('Server.close'); }
}

function connect(options, callback) { _unsupported('connect'); }
function createServer(options, connectionListener) { _unsupported('createServer'); }
function createSecureContext(options) { _unsupported('createSecureContext'); }
function checkServerIdentity(hostname, cert) {
  throw new Error(
    'tls.checkServerIdentity is not implemented in KossJS. ' +
    'Certificate identity validation cannot be safely skipped.'
  );
}

const rootCertificates = [];

module.exports = { TLSSocket, Server, connect, createServer, createSecureContext, checkServerIdentity, rootCertificates };
