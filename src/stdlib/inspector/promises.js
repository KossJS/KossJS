/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const inspector = require('inspector');
const { promisify } = require('internal/util');

class Session extends inspector.Session {}
Session.prototype.post = promisify(inspector.Session.prototype.post);

module.exports = {
  ...inspector,
  Session,
};

