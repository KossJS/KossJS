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
  ObjectSetPrototypeOf,
  SafeMap,
} = primordials;

const {
  getContinuationPreservedEmbedderData,
  setContinuationPreservedEmbedderData,
} = internalBinding('async_context_frame');

let enabled_;

class ActiveAsyncContextFrame extends SafeMap {
  static get enabled() {
    return true;
  }

  static current() {
    return getContinuationPreservedEmbedderData();
  }

  static set(frame) {
    setContinuationPreservedEmbedderData(frame);
  }

  static exchange(frame) {
    const prior = this.current();
    this.set(frame);
    return prior;
  }

  static disable(store) {
    const frame = this.current();
    frame?.disable(store);
  }
}

function checkEnabled() {
  const enabled = require('internal/options')
    .getOptionValue('--async-context-frame');

  // If enabled, swap to active prototype so we don't need to check status
  // on every interaction with the async context frame.
  if (enabled) {
    // eslint-disable-next-line no-use-before-define
    ObjectSetPrototypeOf(AsyncContextFrame, ActiveAsyncContextFrame);
  }

  return enabled;
}

class InactiveAsyncContextFrame extends SafeMap {
  static get enabled() {
    enabled_ ??= checkEnabled();
    return enabled_;
  }

  static current() {}
  static set(frame) {}
  static exchange(frame) {}
  static disable(store) {}
}

class AsyncContextFrame extends InactiveAsyncContextFrame {
  constructor(store, data) {
    super(AsyncContextFrame.current());
    this.set(store, data);
  }

  disable(store) {
    this.delete(store);
  }
}

module.exports = AsyncContextFrame;

