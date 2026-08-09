// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/timers/promises - Node.js timers/promises module (L3)
// setTimeout / setInterval / setImmediate 的 Promise 版本

var timers = require('koss:timers');

function setTimeoutPromise(delay, value, options) {
  var ms = delay === undefined ? 1 : Number(delay);
  var opts = options || {};
  var signal = opts.signal || null;
  return new Promise(function(resolve, reject) {
    if (signal && signal.aborted) {
      reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      return;
    }
    var timer = timers.setTimeout(function() {
      resolve(value);
    }, ms);
    if (signal && typeof signal.addEventListener === 'function') {
      signal.addEventListener('abort', function() {
        try { timers.clearTimeout(timer); } catch (e) {}
        reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      }, { once: true });
    }
  });
}

function setImmediatePromise(value, options) {
  var opts = options || {};
  var signal = opts.signal || null;
  return new Promise(function(resolve, reject) {
    if (signal && signal.aborted) {
      reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      return;
    }
    var timer = timers.setImmediate(function() {
      resolve(value);
    });
    if (signal && typeof signal.addEventListener === 'function') {
      signal.addEventListener('abort', function() {
        try { timers.clearImmediate(timer); } catch (e) {}
        reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      }, { once: true });
    }
  });
}

function setIntervalPromise(delay, value, options) {
  var ms = delay === undefined ? 1 : Number(delay);
  var opts = options || {};
  var signal = opts.signal || null;
  return new Promise(function(resolve, reject) {
    if (signal && signal.aborted) {
      reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      return;
    }
    var timer = timers.setInterval(function() {
      resolve(value);
      try { timers.clearInterval(timer); } catch (e) {}
    }, ms);
    if (signal && typeof signal.addEventListener === 'function') {
      signal.addEventListener('abort', function() {
        try { timers.clearInterval(timer); } catch (e) {}
        reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
      }, { once: true });
    }
  });
}

// Async 迭代器版本
function timersAsyncGenerator(factory, args) {
  var current = null;
  return {
    [Symbol.asyncIterator]: function() { return this; },
    next: function() {
      var self = this;
      if (current) return current;
      current = factory.apply(null, args).then(function(value) {
        current = null;
        return { value: value, done: false };
      });
      return current;
    },
    return: function() {
      current = null;
      return Promise.resolve({ value: undefined, done: true });
    },
  };
}

function setTimeoutAsync(delay, value, options) {
  return timersAsyncGenerator(setTimeoutPromise, [delay, value, options]);
}

function setIntervalAsync(delay, value, options) {
  return timersAsyncGenerator(setIntervalPromise, [delay, value, options]);
}

function setImmediateAsync(value, options) {
  return timersAsyncGenerator(setImmediatePromise, [value, options]);
}

module.exports = {
  setTimeout: setTimeoutPromise,
  setImmediate: setImmediatePromise,
  setInterval: setIntervalPromise,
  scheduler: {
    wait: function(delay, options) { return setTimeoutPromise(delay, undefined, options); },
    yield: function() { return setTimeoutPromise(0); },
    signal: function(signal, options) {
      return new Promise(function(resolve, reject) {
        if (signal.aborted) reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
        else signal.addEventListener('abort', function() {
          reject(signal.reason instanceof Error ? signal.reason : new Error('The operation was aborted'));
        }, { once: true });
      });
    },
  },
};
