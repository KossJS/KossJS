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
  Symbol,
} = primordials;

const { Duplex } = require('stream');
const assert = require('internal/assert');

const kCallback = Symbol('Callback');
const kInitOtherSide = Symbol('InitOtherSide');

class DuplexSide extends Duplex {
  #otherSide = null;

  constructor(options) {
    super(options);
    this[kCallback] = null;
    this.#otherSide = null;
  }

  [kInitOtherSide](otherSide) {
    // Ensure this can only be set once, to enforce encapsulation.
    if (this.#otherSide === null) {
      this.#otherSide = otherSide;
    } else {
      assert(this.#otherSide === null);
    }
  }

  _read() {
    const callback = this[kCallback];
    if (callback) {
      this[kCallback] = null;
      callback();
    }
  }

  _write(chunk, encoding, callback) {
    assert(this.#otherSide !== null);
    assert(this.#otherSide[kCallback] === null);
    if (chunk.length === 0) {
      process.nextTick(callback);
    } else {
      this.#otherSide.push(chunk);
      this.#otherSide[kCallback] = callback;
    }
  }

  _final(callback) {
    this.#otherSide.on('end', callback);
    this.#otherSide.push(null);
  }
}

function duplexPair(options) {
  const side0 = new DuplexSide(options);
  const side1 = new DuplexSide(options);
  side0[kInitOtherSide](side1);
  side1[kInitOtherSide](side0);
  return [ side0, side1 ];
}
module.exports = duplexPair;

