// koss:node/events - Node.js events module (L3)
// Pure JS implementation of EventEmitter

var kMaxListeners = 16;

function isFunction(fn) { return typeof fn === 'function'; }
function isObject(obj) { return obj !== null && typeof obj === 'object'; }
function isNumber(n) { return typeof n === 'number'; }
function isString(s) { return typeof s === 'string'; }

function once(emitter, name) {
  return new Promise(function(resolve, reject) {
    function eventListener() {
      var args = Array.prototype.slice.call(arguments);
      if (errorListener !== undefined) {
        emitter.removeListener('error', errorListener);
      }
      resolve(args.length > 1 ? args : args[0]);
    }
    var errorListener;
    if (name !== 'error') {
      errorListener = function(err) {
        emitter.removeListener(name, eventListener);
        reject(err);
      };
      emitter.once('error', errorListener);
    }
    emitter.once(name, eventListener);
  });
}

function on(emitter, event) {
  var unconsumedEventValues = [];
  var unconsumedPromises = [];
  var error = null;
  var finished = false;

  var iterator = {
    next: function() {
      var value = unconsumedEventValues.shift();
      if (value !== undefined) {
        return Promise.resolve({ done: false, value: value });
      }
      if (finished) {
        return Promise.resolve({ done: true, value: undefined });
      }
      return new Promise(function(resolve, reject) {
        unconsumedPromises.push({ resolve: resolve, reject: reject });
      });
    },
    return: function() {
      emitter.removeListener(event, eventHandler);
      emitter.removeListener('error', errorHandler);
      finished = true;
      return Promise.resolve({ done: true });
    }
  };

  function eventHandler() {
    var args = Array.prototype.slice.call(arguments);
    var promise = unconsumedPromises.shift();
    if (promise !== undefined) {
      promise.resolve({ done: false, value: args.length > 1 ? args : args[0] });
    } else {
      unconsumedEventValues.push(args.length > 1 ? args : args[0]);
    }
  }

  function errorHandler(err) {
    finished = true;
    var promise = unconsumedPromises.shift();
    if (promise !== undefined) {
      promise.reject(err);
    } else {
      error = err;
    }
  }

  emitter.on(event, eventHandler);
  emitter.on('error', errorHandler);

  return iterator;
}

class EventEmitter {
  constructor() {
    this._events = {};
    this._maxListeners = EventEmitter.defaultMaxListeners;
  }

  _addListener(event, listener, prepend) {
    if (!isFunction(listener)) {
      throw new TypeError('listener must be a function');
    }
    if (this._events[event] === undefined) {
      this._events[event] = listener;
    } else if (Array.isArray(this._events[event])) {
      if (prepend) {
        this._events[event].unshift(listener);
      } else {
        this._events[event].push(listener);
      }
    } else {
      this._events[event] = prepend
        ? [listener, this._events[event]]
        : [this._events[event], listener];
    }

    var maxListeners = this._maxListeners;
    if (maxListeners > 0 && this.listenerCount(event) > maxListeners) {
      console.warn('MaxListenersExceededWarning: Possible EventEmitter memory leak detected. ' + this.listenerCount(event) + ' ' + String(event) + ' listeners added. Use emitter.setMaxListeners() to increase limit.');
    }

    return this;
  }

  addListener(event, listener) { return this._addListener(event, listener, false); }
  on(event, listener) { return this._addListener(event, listener, false); }
  prependListener(event, listener) { return this._addListener(event, listener, true); }

  once(event, listener) {
    if (!isFunction(listener)) throw new TypeError('listener must be a function');

    function wrappedListener() {
      var args = Array.prototype.slice.call(arguments);
      this.removeListener(event, wrappedListener);
      return listener.apply(this, args);
    }
    wrappedListener.listener = listener;
    this.on(event, wrappedListener);
    return this;
  }

  prependOnceListener(event, listener) {
    if (!isFunction(listener)) throw new TypeError('listener must be a function');

    function wrappedListener() {
      var args = Array.prototype.slice.call(arguments);
      this.removeListener(event, wrappedListener);
      return listener.apply(this, args);
    }
    wrappedListener.listener = listener;
    this.prependListener(event, wrappedListener);
    return this;
  }

  removeListener(event, listener) {
    if (!isFunction(listener)) return this;
    var list = this._events[event];
    if (list === undefined) return this;

    if (Array.isArray(list)) {
      var index = list.indexOf(listener);
      if (index !== -1) {
        list.splice(index, 1);
        if (list.length === 1) {
          this._events[event] = list[0];
        }
      }
    } else if (list === listener) {
      delete this._events[event];
    }

    return this;
  }

  off(event, listener) { return this.removeListener(event, listener); }

  removeAllListeners(event) {
    if (event === undefined) {
      this._events = {};
      return this;
    }
    delete this._events[event];
    return this;
  }

  setMaxListeners(n) {
    if (!isNumber(n) || n < 0) throw new RangeError('n must be a non-negative number');
    this._maxListeners = n;
    return this;
  }

  getMaxListeners() { return this._maxListeners; }

  listeners(event) {
    var list = this._events[event];
    if (list === undefined) return [];
    if (Array.isArray(list)) return list.slice();
    return [list];
  }

  rawListeners(event) { return this.listeners(event); }

  emit(event) {
    var list = this._events[event];
    if (list === undefined) return false;

    var args = Array.prototype.slice.call(arguments, 1);

    if (Array.isArray(list)) {
      var handlers = list.slice();
      for (var i = 0; i < handlers.length; i++) {
        handlers[i].apply(this, args);
      }
    } else {
      list.apply(this, args);
    }

    return true;
  }

  eventNames() { return Object.keys(this._events); }

  listenerCount(event) {
    var list = this._events[event];
    if (list === undefined) return 0;
    if (Array.isArray(list)) return list.length;
    return 1;
  }
}

EventEmitter.defaultMaxListeners = kMaxListeners;

var eventEmitter = new EventEmitter();
function getEventListeners(emitterOrTarget, event) {
  if (emitterOrTarget && typeof emitterOrTarget.listeners === 'function') {
    return emitterOrTarget.listeners(event);
  }
  return [];
}

function triggerAsyncId() { return 0; }
function executionAsyncId() { return 0; }

module.exports = EventEmitter;
module.exports.once = once;
module.exports.on = on;
module.exports.EventEmitter = EventEmitter;
module.exports.getEventListeners = getEventListeners;
module.exports.triggerAsyncId = triggerAsyncId;
module.exports.executionAsyncId = executionAsyncId;