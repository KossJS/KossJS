/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const { SecureContext, createSecureContext, translatePeerCertificate } = require('internal/tls/common');
module.exports = {
  SecureContext,
  createSecureContext,
  translatePeerCertificate,
};
process.emitWarning('The _tls_common module is deprecated. Use `node:tls` instead.',
                    'DeprecationWarning', 'DEP0192');

