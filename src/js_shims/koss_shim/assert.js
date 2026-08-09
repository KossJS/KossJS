// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:assert — Koss 原生断言标准库

function AssertionError(options) {
  this.name = 'AssertionError';
  this.message = (options && options.message) || '';
  this.actual = options && options.actual;
  this.expected = options && options.expected;
  this.operator = (options && options.operator) || '==';
  if (Error.captureStackTrace) Error.captureStackTrace(this, options && options.stackStartFn || assert);
}
AssertionError.prototype = Object.create(Error.prototype);
AssertionError.prototype.constructor = AssertionError;
Object.defineProperty(AssertionError.prototype, 'toString', {
  value: function() { return this.name + ': ' + this.message; },
  writable: true,
  configurable: true,
});

function isError(e) { return e instanceof Error || (e && e.name && e.message); }

function fail(actual, expected, message, operator, stackStartFn) {
  if (typeof actual === 'string' && expected === undefined) { message = actual; actual = undefined; }
  throw new AssertionError({ message: message, actual: actual, expected: expected, operator: operator, stackStartFn: stackStartFn });
}

function innerEqual(actual, expected, strict, depth) {
  if (actual === expected) return true;
  if (actual === null || expected === null) return false;
  if (typeof actual !== typeof expected) return false;
  if (typeof actual !== 'object') return actual === expected;
  if (depth > 10) return true;
  var actualKeys = Object.keys(actual);
  var expectedKeys = Object.keys(expected);
  if (actualKeys.length !== expectedKeys.length) return false;
  for (var i = 0; i < actualKeys.length; i++) {
    var key = actualKeys[i];
    if (expectedKeys.indexOf(key) === -1) return false;
    if (!innerEqual(actual[key], expected[key], strict, depth + 1)) return false;
  }
  return true;
}

function assert(value, message) { if (!value) fail(value, true, message, '==', assert); }
assert.ok = assert;

assert.equal = function equal(actual, expected, message) {
  if (actual != expected) fail(actual, expected, message, '==', assert.equal);
};

assert.notEqual = function notEqual(actual, expected, message) {
  if (actual == expected) fail(actual, expected, message, '!=', assert.notEqual);
};

assert.strictEqual = function strictEqual(actual, expected, message) {
  if (!Object.is(actual, expected)) fail(actual, expected, message, '===', assert.strictEqual);
};

assert.notStrictEqual = function notStrictEqual(actual, expected, message) {
  if (Object.is(actual, expected)) fail(actual, expected, message, '!==', assert.notStrictEqual);
};

assert.deepEqual = function deepEqual(actual, expected, message) {
  if (!innerEqual(actual, expected, false, 0)) fail(actual, expected, message, 'deepEqual', assert.deepEqual);
};

assert.notDeepEqual = function notDeepEqual(actual, expected, message) {
  if (innerEqual(actual, expected, false, 0)) fail(actual, expected, message, 'notDeepEqual', assert.notDeepEqual);
};

assert.deepStrictEqual = function deepStrictEqual(actual, expected, message) {
  if (!innerEqual(actual, expected, true, 0)) fail(actual, expected, message, 'deepStrictEqual', assert.deepStrictEqual);
};

assert.notDeepStrictEqual = function notDeepStrictEqual(actual, expected, message) {
  if (innerEqual(actual, expected, true, 0)) fail(actual, expected, message, 'notDeepStrictEqual', assert.notDeepStrictEqual);
};

assert.throws = function throws(block, error, message) {
  try { block(); } catch (e) {
    if (error) {
      if (typeof error === 'function') { if (!(e instanceof error)) fail(e, error, message || 'Expected error of type ' + error.name, 'throws', assert.throws); }
      else if (error instanceof RegExp) { if (!error.test(e.message)) fail(e.message, error, message || 'Expected error matching regex', 'throws', assert.throws); }
    }
    return;
  }
  fail(undefined, error, message || 'Missing expected exception', 'throws', assert.throws);
};

assert.rejects = async function rejects(block, error, message) {
  try { await (typeof block === 'function' ? block() : block); } catch (e) {
    if (error) {
      if (typeof error === 'function') { if (!(e instanceof error)) fail(e, error, message || 'Expected error of type ' + error.name, 'rejects', assert.rejects); }
      else if (error instanceof RegExp) { if (!error.test(e.message)) fail(e.message, error, message || 'Expected error matching regex', 'rejects', assert.rejects); }
    }
    return;
  }
  fail(undefined, error, message || 'Missing expected rejection', 'rejects', assert.rejects);
};

assert.doesNotThrow = function doesNotThrow(block, message) {
  try { block(); } catch (e) { fail(e, undefined, message || 'Unexpected exception', 'doesNotThrow', assert.doesNotThrow); }
};

assert.ifError = function ifError(value) { if (value) throw value; };

assert.match = function match(string, regex, message) {
  if (!regex.test(string)) fail(string, regex, message || 'Input did not match regex', 'match', assert.match);
};

assert.doesNotMatch = function doesNotMatch(string, regex, message) {
  if (regex.test(string)) fail(string, regex, message || 'Input matched regex', 'doesNotMatch', assert.doesNotMatch);
};

assert.fail = fail;
assert.AssertionError = AssertionError;

var strict = {
  equal: assert.strictEqual, notEqual: assert.notStrictEqual,
  deepEqual: assert.deepStrictEqual, notDeepEqual: assert.notDeepStrictEqual,
  throws: assert.throws, rejects: assert.rejects,
  doesNotThrow: assert.doesNotThrow, fail: assert.fail, ifError: assert.ifError,
  match: assert.match, doesNotMatch: assert.doesNotMatch,
};
assert.strict = strict;

module.exports = assert;
module.exports.AssertionError = AssertionError;
module.exports.strict = strict;
