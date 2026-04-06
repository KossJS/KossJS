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
  FunctionPrototypeCall,
} = primordials;

const {
  codes: {
    ERR_CRYPTO_KEM_NOT_SUPPORTED,
  },
} = require('internal/errors');

const {
  validateFunction,
} = require('internal/validators');

const {
  kCryptoJobAsync,
  kCryptoJobSync,
  KEMDecapsulateJob,
  KEMEncapsulateJob,
} = internalBinding('crypto');

const {
  preparePrivateKey,
  preparePublicOrPrivateKey,
} = require('internal/crypto/keys');

const {
  getArrayBufferOrView,
} = require('internal/crypto/util');

function encapsulate(key, callback) {
  if (!KEMEncapsulateJob)
    throw new ERR_CRYPTO_KEM_NOT_SUPPORTED();

  if (callback !== undefined)
    validateFunction(callback, 'callback');

  const {
    data: keyData,
    format: keyFormat,
    type: keyType,
    passphrase: keyPassphrase,
    namedCurve: keyNamedCurve,
  } = preparePublicOrPrivateKey(key);

  const job = new KEMEncapsulateJob(
    callback ? kCryptoJobAsync : kCryptoJobSync,
    keyData,
    keyFormat,
    keyType,
    keyPassphrase,
    keyNamedCurve);

  if (!callback) {
    const { 0: err, 1: result } = job.run();
    if (err !== undefined)
      throw err;
    const { 0: sharedKey, 1: ciphertext } = result;
    return { sharedKey, ciphertext };
  }

  job.ondone = (error, result) => {
    if (error) return FunctionPrototypeCall(callback, job, error);
    const { 0: sharedKey, 1: ciphertext } = result;
    FunctionPrototypeCall(callback, job, null, { sharedKey, ciphertext });
  };
  job.run();
}

function decapsulate(key, ciphertext, callback) {
  if (!KEMDecapsulateJob)
    throw new ERR_CRYPTO_KEM_NOT_SUPPORTED();

  if (callback !== undefined)
    validateFunction(callback, 'callback');

  const {
    data: keyData,
    format: keyFormat,
    type: keyType,
    passphrase: keyPassphrase,
    namedCurve: keyNamedCurve,
  } = preparePrivateKey(key);

  ciphertext = getArrayBufferOrView(ciphertext, 'ciphertext');

  const job = new KEMDecapsulateJob(
    callback ? kCryptoJobAsync : kCryptoJobSync,
    keyData,
    keyFormat,
    keyType,
    keyPassphrase,
    keyNamedCurve,
    ciphertext);

  if (!callback) {
    const { 0: err, 1: result } = job.run();
    if (err !== undefined)
      throw err;

    return result;
  }

  job.ondone = (error, result) => {
    if (error) return FunctionPrototypeCall(callback, job, error);
    FunctionPrototypeCall(callback, job, null, result);
  };
  job.run();
}

module.exports = {
  encapsulate,
  decapsulate,
};

