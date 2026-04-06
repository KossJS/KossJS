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
  Error,
  StringPrototypeStartsWith,
  globalThis,
} = primordials;

process.emitWarning(
  'These APIs are for internal testing only. Do not use them.',
  'internal/test/binding');

function filteredInternalBinding(id) {
  // Disallows internal bindings with names that start with 'internal_only'
  // which means it should not be exposed to users even with
  // --expose-internals.
  if (StringPrototypeStartsWith(id, 'internal_only')) {
    // This code is only intended for internal errors and is not documented.
    // Do not use the normal error system.
    // eslint-disable-next-line no-restricted-syntax
    const error = new Error(`No such binding: ${id}`);
    error.code = 'ERR_INVALID_MODULE';
    throw error;
  }
  return internalBinding(id);
}

if (module.isPreloading) {
  globalThis.internalBinding = filteredInternalBinding;
  globalThis.primordials = primordials;
}

module.exports = { internalBinding: filteredInternalBinding, primordials };

