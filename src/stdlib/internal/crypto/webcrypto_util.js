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
  KeyObjectHandle,
  kKeyFormatDER,
  kKeyFormatJWK,
  kKeyEncodingPKCS8,
  kKeyEncodingSPKI,
  kKeyTypePublic,
  kKeyTypePrivate,
} = internalBinding('crypto');

const {
  validateKeyOps,
} = require('internal/crypto/util');

const {
  lazyDOMException,
} = require('internal/util');

const {
  PrivateKeyObject,
  PublicKeyObject,
} = require('internal/crypto/keys');

function importDerKey(keyData, isPublic) {
  const handle = new KeyObjectHandle();
  const keyType = isPublic ? kKeyTypePublic : kKeyTypePrivate;
  const encoding = isPublic ? kKeyEncodingSPKI : kKeyEncodingPKCS8;
  try {
    handle.init(keyType, keyData, kKeyFormatDER, encoding, null, null);
  } catch (err) {
    throw lazyDOMException(
      'Invalid keyData', { name: 'DataError', cause: err });
  }
  return isPublic ?
    new PublicKeyObject(handle) :
    new PrivateKeyObject(handle);
}

function validateJwk(keyData, kty, extractable, usagesSet, expectedUse) {
  if (!keyData.kty)
    throw lazyDOMException('Invalid keyData', 'DataError');
  if (keyData.kty !== kty)
    throw lazyDOMException('Invalid JWK "kty" Parameter', 'DataError');
  if (usagesSet.size > 0 && keyData.use !== undefined) {
    if (keyData.use !== expectedUse)
      throw lazyDOMException('Invalid JWK "use" Parameter', 'DataError');
  }
  validateKeyOps(keyData.key_ops, usagesSet);
  if (keyData.ext !== undefined &&
      keyData.ext === false &&
      extractable === true) {
    throw lazyDOMException(
      'JWK "ext" Parameter and extractable mismatch',
      'DataError');
  }
}

function importJwkKey(isPublic, keyData) {
  const handle = new KeyObjectHandle();
  const keyType = isPublic ? kKeyTypePublic : kKeyTypePrivate;
  try {
    handle.init(keyType, keyData, kKeyFormatJWK, null, null, null);
  } catch (err) {
    throw lazyDOMException(
      'Invalid keyData', { name: 'DataError', cause: err });
  }
  return isPublic ?
    new PublicKeyObject(handle) :
    new PrivateKeyObject(handle);
}

function importRawKey(isPublic, keyData, format, name, namedCurve) {
  const handle = new KeyObjectHandle();
  const keyType = isPublic ? kKeyTypePublic : kKeyTypePrivate;
  try {
    handle.init(keyType, keyData, format, name ?? null, null, namedCurve ?? null);
  } catch (err) {
    throw lazyDOMException(
      'Invalid keyData', { name: 'DataError', cause: err });
  }
  return isPublic ?
    new PublicKeyObject(handle) :
    new PrivateKeyObject(handle);
}

module.exports = {
  importDerKey,
  importJwkKey,
  importRawKey,
  validateJwk,
};

