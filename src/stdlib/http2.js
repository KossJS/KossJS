/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const {
  connect,
  constants,
  createServer,
  createSecureServer,
  getDefaultSettings,
  getPackedSettings,
  getUnpackedSettings,
  performServerHandshake,
  sensitiveHeaders,
  Http2ServerRequest,
  Http2ServerResponse,
} = require('internal/http2/core');

module.exports = {
  connect,
  constants,
  createServer,
  createSecureServer,
  getDefaultSettings,
  getPackedSettings,
  getUnpackedSettings,
  performServerHandshake,
  sensitiveHeaders,
  Http2ServerRequest,
  Http2ServerResponse,
};

