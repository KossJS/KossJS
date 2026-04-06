/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const { isMainThread } = require('worker_threads');
const {
  ERR_WORKER_UNSUPPORTED_OPERATION,
} = require('internal/errors').codes;

let sigintWatchdog;
function getSigintWatchdog() {
  if (!sigintWatchdog) {
    const { SigintWatchdog } = require('internal/watchdog');
    sigintWatchdog = new SigintWatchdog();
  }
  return sigintWatchdog;
}

function setTraceSigInt(enable) {
  if (!isMainThread)
    throw new ERR_WORKER_UNSUPPORTED_OPERATION('Calling util.setTraceSigInt');
  if (enable) {
    getSigintWatchdog().start();
  } else {
    getSigintWatchdog().stop();
  }
};

module.exports = {
  setTraceSigInt,
};

