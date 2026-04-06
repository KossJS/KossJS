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
  ReflectConstruct,
  SafeMap,
  Symbol,
  SymbolDispose,
} = primordials;

const {
  codes: {
    ERR_ILLEGAL_CONSTRUCTOR,
    ERR_INVALID_THIS,
  },
} = require('internal/errors');

const {
  createELDHistogram,
} = internalBinding('performance');

const {
  validateInteger,
  validateObject,
} = require('internal/validators');

const {
  Histogram,
  kHandle,
  kMap,
} = require('internal/histogram');

const {
  kEmptyObject,
} = require('internal/util');

const {
  markTransferMode,
} = require('internal/worker/js_transferable');

const kEnabled = Symbol('kEnabled');

class ELDHistogram extends Histogram {
  constructor() {
    throw new ERR_ILLEGAL_CONSTRUCTOR();
  }

  /**
   * @returns {boolean}
   */
  enable() {
    if (this[kEnabled] === undefined)
      throw new ERR_INVALID_THIS('ELDHistogram');
    if (this[kEnabled]) return false;
    this[kEnabled] = true;
    this[kHandle].start();
    return true;
  }

  /**
   * @returns {boolean}
   */
  disable() {
    if (this[kEnabled] === undefined)
      throw new ERR_INVALID_THIS('ELDHistogram');
    if (!this[kEnabled]) return false;
    this[kEnabled] = false;
    this[kHandle].stop();
    return true;
  }

  [SymbolDispose]() {
    this.disable();
  }
}

/**
 * @param {{
 *   resolution : number
 * }} [options]
 * @returns {ELDHistogram}
 */
function monitorEventLoopDelay(options = kEmptyObject) {
  validateObject(options, 'options');

  const { resolution = 10 } = options;
  validateInteger(resolution, 'options.resolution', 1);

  return ReflectConstruct(
    function() {
      markTransferMode(this, true, false);
      this[kEnabled] = false;
      this[kHandle] = createELDHistogram(resolution);
      this[kMap] = new SafeMap();
    }, [], ELDHistogram);
}

module.exports = monitorEventLoopDelay;

