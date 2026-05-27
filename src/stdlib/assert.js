'use strict';

const { Error: Err, SymbolFor } = globalThis;

function assert(condition, message) {
    if (!condition) {
        throw new Err(message || 'Assertion failed');
    }
}

function assertStrictEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Err(message || `Expected ${expected} but got ${actual}`);
    }
}

function assertDeepStrictEqual(actual, expected, message) {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        throw new Err(message || `Expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`);
    }
}

function assertEqual(actual, expected, message) {
    if (actual != expected) {
        throw new Err(message || `Expected ${expected} but got ${actual}`);
    }
}

function assertFail(message) {
    throw new Err(message || 'Assertion failed');
}

function assertok(condition, message) {
    if (!condition) {
        throw new Err(message || 'Assertion failed');
    }
}

function assertthrows(fn, error, message) {
    let thrown = false;
    try {
        fn();
    } catch (e) {
        thrown = true;
        if (error && !(e instanceof error)) {
            throw new Err(message || `Expected ${error.name} but got ${e.name}`);
        }
    }
    if (!thrown) {
        throw new Err(message || 'Expected exception but none was thrown');
    }
}

function assertdoesnotthrow(fn, message) {
    try {
        fn();
    } catch (e) {
        throw new Err(message || `Unexpected exception: ${e.message}`);
    }
}

module.exports = assert;
module.exports.strict = assertStrictEqual;
module.exports.ok = assertok;
module.exports.equal = assertEqual;
module.exports.notEqual = function(actual, expected, message) {
    if (actual == expected) throw new Err(message || `Expected ${actual} to not equal ${expected}`);
};
module.exports.deepEqual = assertDeepStrictEqual;
module.exports.deepStrictEqual = assertDeepStrictEqual;
module.exports.notDeepStrictEqual = function(actual, expected, message) {
    if (JSON.stringify(actual) === JSON.stringify(expected))
        throw new Err(message || 'Expected values to not be deeply equal');
};
module.exports.strictEqual = assertStrictEqual;
module.exports.notStrictEqual = function(actual, expected, message) {
    if (actual === expected) throw new Err(message || `Expected ${actual} to not strictly equal ${expected}`);
};
module.exports.fail = assertFail;
module.exports.throws = function(fn, expected, message) {
    let thrown = false;
    try { fn(); } catch (e) {
        thrown = true;
        if (expected) {
            if (typeof expected === 'function') {
                if (!(e instanceof expected))
                    throw new Err(message || `Expected error to be instance of ${expected.name || 'Error'}`);
            } else if (expected instanceof RegExp) {
                if (!expected.test(e.message))
                    throw new Err(message || `Expected error message to match ${expected}`);
            }
        }
    }
    if (!thrown) throw new Err(message || 'Expected exception but none was thrown');
};
module.exports.doesNotThrow = assertdoesnotthrow;
module.exports.ifError = function(err) { if (err) throw err; };
