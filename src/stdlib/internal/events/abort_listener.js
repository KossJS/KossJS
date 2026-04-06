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
  SymbolDispose,
} = primordials;
const {
  validateAbortSignal,
  validateFunction,
} = require('internal/validators');
const {
  codes: {
    ERR_INVALID_ARG_TYPE,
  },
} = require('internal/errors');

let queueMicrotask;
let kResistStopPropagation;

/**
 * @param {AbortSignal} signal
 * @param {EventListener} listener
 * @returns {Disposable}
 */
function addAbortListener(signal, listener) {
  if (signal === undefined) {
    throw new ERR_INVALID_ARG_TYPE('signal', 'AbortSignal', signal);
  }
  validateAbortSignal(signal, 'signal');
  validateFunction(listener, 'listener');

  let removeEventListener;
  if (signal.aborted) {
    queueMicrotask ??= require('internal/process/task_queues').queueMicrotask;
    queueMicrotask(() => listener());
  } else {
    kResistStopPropagation ??= require('internal/event_target').kResistStopPropagation;
    // TODO(atlowChemi) add { subscription: true } and return directly
    signal.addEventListener('abort', listener, { __proto__: null, once: true, [kResistStopPropagation]: true });
    removeEventListener = () => {
      signal.removeEventListener('abort', listener);
    };
  }
  return {
    __proto__: null,
    [SymbolDispose]() {
      removeEventListener?.();
    },
  };
}

module.exports = {
  __proto__: null,
  addAbortListener,
};

