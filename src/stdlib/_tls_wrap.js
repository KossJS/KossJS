/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const { TLSSocket, Server, createServer, connect } = require('internal/tls/wrap');
module.exports = {
  TLSSocket,
  Server,
  createServer,
  connect,
};
process.emitWarning('The _tls_wrap module is deprecated. Use `node:tls` instead.',
                    'DeprecationWarning', 'DEP0192');

