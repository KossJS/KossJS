// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/https - Node.js https module (L3)
// 注意：KossJS 未实现真实 TLS。https.request/get 若静默降级为明文 HTTP，
// 会让调用方误以为流量已加密。因此所有 https 操作显式抛错，
// 直到真实 rustls 集成完成。

function _unsupported(name) {
  throw new Error(
    'https.' + name + ' is not implemented in KossJS. ' +
    'TLS (rustls integration) is not yet wired up. ' +
    'Using this module would silently downgrade to plaintext HTTP, which is unsafe. ' +
    'Do NOT rely on this module for security.'
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
  constructor(options, requestListener) { _unsupported('Server'); }
  listen() { _unsupported('Server.listen'); }
  close() { _unsupported('Server.close'); }
}

function createServer(options, requestListener) { _unsupported('createServer'); }
function request(url, options, callback) { _unsupported('request'); }
function get(url, options, callback) { _unsupported('get'); }

const globalAgent = { maxSockets: Infinity };

module.exports = { createServer, request, get, Server, TLSSocket, globalAgent };
