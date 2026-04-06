'use strict';

const { ObjectDefineProperty, ObjectKeys, SymbolFor } = globalThis;

const kIsNodeError = SymbolFor('kIsNodeError');

const messages = new Map();
const codes = {};

const classRegExp = /^[A-Z][a-zA-Z0-9]*$/;

const nodeErrorMap = {
    ERR_AMBIGUOUS_ARGUMENT: 'ERR_AMBIGUOUS_ARGUMENT',
    ERR_CONSTRUCT_CALL_REQUIRED: 'ERR_CONSTRUCT_CALL_REQUIRED',
    ERR_INVALID_ARG_TYPE: 'ERR_INVALID_ARG_TYPE',
    ERR_INVALID_ARG_VALUE: 'ERR_INVALID_ARG_VALUE',
    ERR_INVALID_RETURN_VALUE: 'ERR_INVALID_RETURN_VALUE',
    ERR_MISSING_ARGS: 'ERR_MISSING_ARGS',
    ERR_OUT_OF_RANGE: 'ERR_OUT_OF_RANGE',
    ERR_UNHANDLED_ERROR: 'ERR_UNHANDLED_ERROR',
};

function makeNodeError(constructor, message, code) {
    const error = new constructor(message);
    error.code = code;
    ObjectDefineProperty(error, kIsNodeError, {
        value: true,
        enumerable: false,
        configurable: false,
        writable: false,
    });
    return error;
}

const nodeErrors = {
    codes,
    nodeErrorMap,
    makeNodeError,
};

module.exports = nodeErrors;
module.exports.codes = codes;
module.exports.kIsNodeError = kIsNodeError;
