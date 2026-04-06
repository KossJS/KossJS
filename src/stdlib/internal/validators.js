'use strict';

const {
  ArrayIsArray,
  NumberIsFinite,
  NumberIsInteger,
  NumberIsNaN,
  NumberMAX_SAFE_INTEGER,
  NumberMIN_SAFE_INTEGER,
} = globalThis;

const {
  codes: {
    ERR_INVALID_ARG_TYPE,
    ERR_INVALID_ARG_VALUE,
    ERR_OUT_OF_RANGE,
  },
  hideStackFrames,
} = require('internal/errors');

function isInt32(value) {
  return value === (value | 0);
}

function validateNumber(value, name, min = NumberMIN_SAFE_INTEGER, max = NumberMAX_SAFE_INTEGER) {
  if (typeof value !== 'number') {
    throw new ERR_INVALID_ARG_TYPE(name, 'number', value);
  }
  if (NumberIsNaN(value) || !NumberIsFinite(value) || value < min || value > max) {
    throw new ERR_OUT_OF_RANGE(name, `>= ${min} and <= ${max}`, value);
  }
}

function validateInteger(value, name, min = NumberMIN_SAFE_INTEGER, max = NumberMAX_SAFE_INTEGER) {
  if (!NumberIsInteger(value)) {
    throw new ERR_INVALID_ARG_TYPE(name, 'an integer', value);
  }
  if (value < min || value > max) {
    throw new ERR_OUT_OF_RANGE(name, `>= ${min} and <= ${max}`, value);
  }
}

function validateString(value, name) {
  if (typeof value !== 'string') {
    throw new ERR_INVALID_ARG_TYPE(name, 'string', value);
  }
}

function validateBoolean(value, name) {
  if (typeof value !== 'boolean') {
    throw new ERR_INVALID_ARG_TYPE(name, 'boolean', value);
  }
}

function validateFunction(value, name) {
  if (typeof value !== 'function') {
    throw new ERR_INVALID_ARG_TYPE(name, 'function', value);
  }
}

function validateObject(value, name, nullable = false) {
  if (nullable && (value === null || value === undefined)) {
    return;
  }
  if (typeof value !== 'object' || value === null) {
    throw new ERR_INVALID_ARG_TYPE(name, 'object', value);
  }
}

function validateArray(value, name) {
  if (!ArrayIsArray(value)) {
    throw new ERR_INVALID_ARG_TYPE(name, 'array', value);
  }
}

function validateOneOf(value, name, values) {
  if (!values.includes(value)) {
    throw new ERR_INVALID_ARG_VALUE(name, value, `must be one of: ${values.join(', ')}`);
  }
}

function validateBuffer(buffer, name = 'buffer') {
  if (!buffer || typeof buffer !== 'object') {
    throw new ERR_INVALID_ARG_TYPE(name, 'Buffer', buffer);
  }
}

function validateEncoding(value, name) {
  const validEncodings = ['utf8', 'utf-8', 'ascii', 'latin1', 'binary', 'base64', 'hex', 'ucs2', 'utf16le'];
  if (value && !validEncodings.includes(value.toLowerCase())) {
    throw new ERR_INVALID_ARG_TYPE(name, 'string', value);
  }
}

function validateOffset(value, name, min = 0, max = 2147483647) {
  validateInteger(value, name, min, max);
}

function validateBufferArray(buffers) {
  if (!ArrayIsArray(buffers)) {
    throw new ERR_INVALID_ARG_TYPE('buffers', 'array', buffers);
  }
  for (let i = 0; i < buffers.length; i++) {
    if (!buffers[i] || typeof buffers[i] !== 'object') {
      throw new ERR_INVALID_ARG_TYPE(`buffers[${i}]`, 'Buffer', buffers[i]);
    }
  }
}

function validateStringAfterArrayBufferView(buffer, name) {
  if (typeof buffer !== 'string') {
    throw new ERR_INVALID_ARG_TYPE(name, 'string or Buffer', buffer);
  }
}

function validatePosition(position, name, length) {
  if (position !== null && position !== undefined) {
    if (typeof position === 'bigint') {
      position = Number(position);
    }
    if (!NumberIsInteger(position)) {
      throw new ERR_INVALID_ARG_TYPE(name, 'integer', position);
    }
    if (position < -1 || (length !== undefined && position > length)) {
      throw new ERR_OUT_OF_RANGE(name, `>= -1 and <= ${length}`, position);
    }
  }
}

module.exports = {
  isInt32,
  validateNumber,
  validateInteger,
  validateString,
  validateBoolean,
  validateFunction,
  validateObject,
  validateArray,
  validateOneOf,
  validateBuffer,
  validateEncoding,
  validateOffset,
  validateBufferArray,
  validateStringAfterArrayBufferView,
  validatePosition,
  kValidateObjectAllowNullable: { __proto__: null },
};