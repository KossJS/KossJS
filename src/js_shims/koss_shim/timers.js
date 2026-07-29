// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:timers — Koss 原生定时器标准库
// 完整定时器实现：setTimeout/setInterval/setImmediate + Promise + Timeout 类

var nextId = 1;
var activeTimers = {};

function isNumber(val) {
  return typeof val === 'number' && isFinite(val) && Math.floor(val) === val;
}

// ═══════════════════════════════════════════
// Timeout 类
// ═══════════════════════════════════════════

function Timeout(id, isInterval, callback, args) {
  this._id = id;
  this._isInterval = isInterval;
  this._callback = callback;
  this._args = args;
  this._refed = true;
  this._destroyed = false;
}

Timeout.prototype.ref = function() {
  if (!this._destroyed) this._refed = true;
  return this;
};

Timeout.prototype.unref = function() {
  if (!this._destroyed) this._refed = false;
  return this;
};

Timeout.prototype.hasRef = function() {
  return this._refed && !this._destroyed;
};

Timeout.prototype.refresh = function() {
  if (this._destroyed) return this;
  var self = this;
  var existing = activeTimers[this._id];
  if (existing && existing.timerId !== undefined) {
    if (self._isInterval) {
      globalThis.clearInterval(existing.timerId);
    } else {
      globalThis.clearTimeout(existing.timerId);
    }
  }
  var delay = existing ? existing.delay : 1;
  var timerFn = self._isInterval
    ? function() { self._callback.apply(null, self._args); }
    : function() { self._callback.apply(null, self._args); delete activeTimers[self._id]; };
  var newId = self._isInterval
    ? globalThis.setInterval(timerFn, delay)
    : globalThis.setTimeout(timerFn, delay);
  activeTimers[self._id] = { timerId: newId, delay: delay, refed: self._refed };
  return this;
};

Timeout.prototype[Symbol.toPrimitive] = function() {
  return this._id;
};

// ═══════════════════════════════════════════
// 核心定时器函数
// ═══════════════════════════════════════════

function setTimeout(callback, delay) {
  var args = [];
  for (var i = 2; i < arguments.length; i++) args.push(arguments[i]);
  var d = isNumber(delay) ? Math.max(0, Math.floor(delay)) : 1;
  var id = nextId++;
  var wrapped = function() { callback.apply(null, args); };
  var timerId = globalThis.setTimeout(wrapped, d);
  activeTimers[id] = { timerId: timerId, delay: d, refed: true };
  return new Timeout(id, false, callback, args);
}

function clearTimeout(timeout) {
  if (timeout instanceof Timeout) {
    var entry = activeTimers[timeout._id];
    if (entry) {
      globalThis.clearTimeout(entry.timerId);
      timeout._destroyed = true;
      delete activeTimers[timeout._id];
    }
  } else if (typeof timeout === 'number') {
    for (var key in activeTimers) {
      if (activeTimers[key].timerId === timeout) {
        globalThis.clearTimeout(timeout);
        delete activeTimers[key];
        break;
      }
    }
  }
}

function setInterval(callback, delay) {
  var args = [];
  for (var i = 2; i < arguments.length; i++) args.push(arguments[i]);
  var d = isNumber(delay) ? Math.max(1, Math.floor(delay)) : 1;
  var id = nextId++;
  var wrapped = function() { callback.apply(null, args); };
  var timerId = globalThis.setInterval(wrapped, d);
  activeTimers[id] = { timerId: timerId, delay: d, refed: true };
  return new Timeout(id, true, callback, args);
}

function clearInterval(timeout) {
  if (timeout instanceof Timeout) {
    var entry = activeTimers[timeout._id];
    if (entry) {
      globalThis.clearInterval(entry.timerId);
      timeout._destroyed = true;
      delete activeTimers[timeout._id];
    }
  } else if (typeof timeout === 'number') {
    for (var key in activeTimers) {
      if (activeTimers[key].timerId === timeout) {
        globalThis.clearInterval(timeout);
        delete activeTimers[key];
        break;
      }
    }
  }
}

function setImmediate(callback) {
  var args = [];
  for (var i = 1; i < arguments.length; i++) args.push(arguments[i]);
  var id = nextId++;
  var wrapped = function() { callback.apply(null, args); };
  var timerId = globalThis.setTimeout(wrapped, 0);
  activeTimers[id] = { timerId: timerId, delay: 0, refed: true };
  return new Timeout(id, false, callback, args);
}

function clearImmediate(timeout) {
  clearTimeout(timeout);
}

// ═══════════════════════════════════════════
// Promise 版本
// ═══════════════════════════════════════════

function setTimeoutPromise(delay, value, options) {
  return new Promise(function(resolve) {
    var args = [];
    if (options && options.args) args = options.args;
    setTimeout(function() { resolve(value); }, delay);
  });
}

function setImmediatePromise(value) {
  return new Promise(function(resolve) {
    setImmediate(function() { resolve(value); });
  });
}

function setIntervalPromise(delay, value, options) {
  var unconsumedValues = [];
  var unconsumedPromises = [];
  var finished = false;
  var timerId = null;

  function produceValue() {
    if (finished) return;
    var promise = unconsumedPromises.shift();
    if (promise !== undefined) {
      promise.resolve({ value: value, done: false });
    } else {
      unconsumedValues.push(value);
    }
  }

  var iterator = {
    next: function() {
      var val = unconsumedValues.shift();
      if (val !== undefined) {
        return Promise.resolve({ value: val, done: false });
      }
      if (finished) {
        return Promise.resolve({ value: undefined, done: true });
      }
      return new Promise(function(resolve, reject) {
        unconsumedPromises.push({ resolve: resolve, reject: reject });
      });
    },
    return: function() {
      finished = true;
      if (timerId !== null) {
        clearInterval(timerId);
        timerId = null;
      }
      return Promise.resolve({ value: undefined, done: true });
    },
    [Symbol.asyncIterator]: function() { return this; }
  };

  timerId = setInterval(produceValue, delay);

  return iterator;
}

// ═══════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════

function active(timer) {
  if (timer instanceof Timeout) {
    return !timer._destroyed && activeTimers[timer._id] !== undefined;
  }
  return false;
}

function enroll(timer, delay) {
  if (timer instanceof Timeout && isNumber(delay)) {
    timer.refresh();
  }
}

function unenroll(timer) {
  if (timer instanceof Timeout) {
    clearTimeout(timer);
  }
}

// ═══════════════════════════════════════════
// 清理
// ═══════════════════════════════════════════

function clearAllTimers() {
  for (var key in activeTimers) {
    var entry = activeTimers[key];
    if (entry._isInterval) {
      globalThis.clearInterval(entry.timerId);
    } else {
      globalThis.clearTimeout(entry.timerId);
    }
    delete activeTimers[key];
  }
}

// ═══════════════════════════════════════════
// 导出
// ═══════════════════════════════════════════

module.exports = {
  setTimeout: setTimeout,
  clearTimeout: clearTimeout,
  setInterval: setInterval,
  clearInterval: clearInterval,
  setImmediate: setImmediate,
  clearImmediate: clearImmediate,
  setTimeoutPromise: setTimeoutPromise,
  setImmediatePromise: setImmediatePromise,
  setIntervalPromise: setIntervalPromise,
  active: active,
  enroll: enroll,
  unenroll: unenroll,
  clearAllTimers: clearAllTimers,
  Timeout: Timeout,
};
