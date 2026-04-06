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

class RunScope {
  #storage;
  #previousStore;
  #disposed = false;

  constructor(storage, store) {
    this.#storage = storage;
    this.#previousStore = storage.getStore();
    storage.enterWith(store);
  }

  dispose() {
    if (this.#disposed) {
      return;
    }
    this.#disposed = true;
    this.#storage.enterWith(this.#previousStore);
  }

  [SymbolDispose]() {
    this.dispose();
  }
}

module.exports = RunScope;

