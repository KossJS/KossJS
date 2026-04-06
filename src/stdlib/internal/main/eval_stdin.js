/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

// Stdin is not a TTY, we will read it and execute it.

const {
  prepareMainThreadExecution,
  markBootstrapComplete,
} = require('internal/process/pre_execution');

const { getOptionValue } = require('internal/options');

const {
  evalModuleEntryPoint,
  evalTypeScript,
  parseAndEvalCommonjsTypeScript,
  parseAndEvalModuleTypeScript,
  evalScript,
  readStdin,
} = require('internal/process/execution');

prepareMainThreadExecution();
markBootstrapComplete();

readStdin((code) => {
  // This is necessary for fork() and CJS module compilation.
  // TODO(joyeecheung): pass this with something really internal.
  process._eval = code;

  const print = getOptionValue('--print');
  const shouldLoadESM = getOptionValue('--import').length > 0;
  const inputType = getOptionValue('--input-type');
  const tsEnabled = getOptionValue('--strip-types');
  if (inputType === 'module') {
    evalModuleEntryPoint(code, print);
  } else if (inputType === 'module-typescript' && tsEnabled) {
    parseAndEvalModuleTypeScript(code, print);
  } else {

    let evalFunction;
    if (inputType === 'commonjs') {
      evalFunction = evalScript;
    } else if (inputType === 'commonjs-typescript' && tsEnabled) {
      evalFunction = parseAndEvalCommonjsTypeScript;
    } else if (tsEnabled) {
      evalFunction = evalTypeScript;
    } else {
      // Default to commonjs.
      evalFunction = evalScript;
    }

    evalFunction('[stdin]',
                 code,
                 getOptionValue('--inspect-brk'),
                 print,
                 shouldLoadESM);
  }
});

