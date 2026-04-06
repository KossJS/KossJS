/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const { getOptionValue } = require('internal/options');
const { validateString } = require('internal/validators');
const { putNetworkResource } = internalBinding('inspector');

/**
 * Registers a resource for the inspector using the internal 'putNetworkResource' binding.
 * @param {string} url - The URL of the resource.
 * @param {string} data - The content of the resource to provide.
 */
function put(url, data) {
  if (!getOptionValue('--experimental-inspector-network-resource')) {
    process.emitWarning(
      'The --experimental-inspector-network-resource option is not enabled. ' +
        'Please enable it to use the putNetworkResource function');
    return;
  }
  validateString(url, 'url');
  validateString(data, 'data');

  putNetworkResource(url, data);
}

module.exports = {
  put,
};

