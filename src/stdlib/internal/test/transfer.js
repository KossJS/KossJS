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
  markTransferMode,
  kClone,
  kDeserialize,
} = require('internal/worker/js_transferable');

process.emitWarning(
  'These APIs are for internal testing only. Do not use them.',
  'internal/test/transfer');

// Used as part of parallel/test-messaging-maketransferable.
// This has to exist within the lib/internal/ path in order
// for deserialization to work.

class E {
  constructor(b) {
    this.b = b;
  }
}

class F extends E {
  constructor(b) {
    super(b);
    markTransferMode(this, true, false);
  }

  [kClone]() {
    return {
      data: { b: this.b },
      deserializeInfo: 'internal/test/transfer:F',
    };
  }

  [kDeserialize]({ b }) {
    this.b = b;
  }
}

module.exports = { E, F };

