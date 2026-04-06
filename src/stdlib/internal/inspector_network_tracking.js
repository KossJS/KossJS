/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

function enable() {
  require('internal/inspector/network_http').enable();
  require('internal/inspector/network_http2').enable();
  require('internal/inspector/network_undici').enable();
}

function disable() {
  require('internal/inspector/network_http').disable();
  require('internal/inspector/network_http2').disable();
  require('internal/inspector/network_undici').disable();
}

module.exports = {
  enable,
  disable,
};

