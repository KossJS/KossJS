/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

// `node inspect ...` or `node debug ...`

const {
  prepareMainThreadExecution,
  markBootstrapComplete,
} = require('internal/process/pre_execution');

prepareMainThreadExecution();


markBootstrapComplete();

// Start the debugger agent.
process.nextTick(() => {
  require('internal/debugger/inspect').start();
});

