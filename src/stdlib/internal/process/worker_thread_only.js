/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

// This file contains process bootstrappers that can only be
// run in the worker thread.

const {
  codes: {
    ERR_WORKER_UNSUPPORTED_OPERATION,
  },
} = require('internal/errors');

function unavailable(name) {
  function unavailableInWorker() {
    throw new ERR_WORKER_UNSUPPORTED_OPERATION(name);
  }

  unavailableInWorker.disabled = true;
  return unavailableInWorker;
}

module.exports = {
  unavailable,
};

