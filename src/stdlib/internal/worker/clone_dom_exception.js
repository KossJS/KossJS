/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

// Delegate to the actual DOMException/QuotaExceededError implementation.
const messaging = internalBinding('messaging');
module.exports = {
  DOMException: messaging.DOMException,
  QuotaExceededError: messaging.QuotaExceededError,
};

