'use strict';

const {
  MathTrunc,
  ObjectDefineProperties,
  ObjectDefineProperty,
  SymbolDispose,
  SymbolToPrimitive,
  Promise,
  setTimeout: jsSetTimeout,
  clearTimeout: jsClearTimeout,
} = globalThis;

const knownTimersById = {};
let nextTimerId = 1;

const async_id_symbol = Symbol('async_id');
const kHasPrimitive = Symbol('has_primitive');
const kRefed = Symbol('refed');

class Timeout {
  constructor(callback, idleTimeout, args, isInterval, isRefed) {
    this[async_id_symbol] = nextTimerId++;
    this._onTimeout = callback;
    this._idleTimeout = idleTimeout;
    this._args = args;
    this._isInterval = isInterval;
    this[kRefed] = isRefed;
    this[kHasPrimitive] = false;
    this._destroyed = false;
    this._timerId = null;
    this._started = false;
  }
}

function insert(item, timeout) {
  if (item._started) return;
  item._started = true;
  
  const delay = MathTrunc(timeout);
  const id = item[async_id_symbol];
  
  knownTimersById[id] = item;
  
  const callback = () => {
    if (item._destroyed) return;
    if (item._onTimeout) {
      try {
        if (item._args) {
          item._onTimeout(...item._args);
        } else {
          item._onTimeout();
        }
      } catch (e) {
        console.error('Timer callback error:', e);
      }
    }
    
    if (!item._isInterval) {
      unenroll(item);
    }
  };
  
  item._timerId = jsSetTimeout(callback, delay);
}

function unenroll(item) {
  if (item._destroyed) return;
  item._destroyed = true;
  
  if (item[kHasPrimitive]) {
    delete knownTimersById[item[async_id_symbol]];
  }
  
  if (item._timerId) {
    jsClearTimeout(item._timerId);
    item._timerId = null;
  }
  
  item._idleTimeout = -1;
}

function setTimeout(callback, after = 0, ...args) {
  if (typeof callback !== 'function') {
    throw new TypeError('callback must be a function');
  }
  const timeout = new Timeout(callback, after, args.length ? args : undefined, false, true);
  insert(timeout, timeout._idleTimeout);
  return timeout;
}

ObjectDefineProperty(setTimeout, 'promises', {
  __proto__: null,
  enumerable: true,
  configurable: true,
  get() {
    return require('timers/promises').setTimeout;
  },
});

function clearTimeout(timer) {
  if (!timer) return;
  
  if (timer._onTimeout) {
    timer._onTimeout = null;
    unenroll(timer);
    return;
  }
  
  if (typeof timer === 'number' || typeof timer === 'string') {
    const timerInstance = knownTimersById[timer];
    if (timerInstance !== undefined) {
      timerInstance._onTimeout = null;
      unenroll(timerInstance);
    }
  }
}

function setInterval(callback, repeat = 0, ...args) {
  if (typeof callback !== 'function') {
    throw new TypeError('callback must be a function');
  }
  const timeout = new Timeout(callback, repeat, args.length ? args : undefined, true, true);
  insert(timeout, timeout._idleTimeout);
  return timeout;
}

function clearInterval(timer) {
  clearTimeout(timer);
}

Timeout.prototype.close = function() {
  clearTimeout(this);
  return this;
};

Timeout.prototype[SymbolDispose] = function() {
  clearTimeout(this);
};

Timeout.prototype[SymbolToPrimitive] = function() {
  const id = this[async_id_symbol];
  if (!this[kHasPrimitive]) {
    this[kHasPrimitive] = true;
    knownTimersById[id] = this;
  }
  return id;
};

class Immediate {
  constructor(callback, args) {
    this[async_id_symbol] = nextTimerId++;
    this._onImmediate = callback;
    this._args = args;
    this[kRefed] = true;
    this._destroyed = false;
    this._started = false;
  }
}

const immediateQueue = [];
let immediateId = 0;

Immediate.prototype[SymbolDispose] = function() {
  clearImmediate(this);
};

function setImmediate(callback, ...args) {
  if (typeof callback !== 'function') {
    throw new TypeError('callback must be a function');
  }
  
  const immediate = new Immediate(callback, args.length ? args : undefined);
  immediateQueue.push(immediate);
  
  jsSetTimeout(() => {
    if (!immediate._destroyed && immediate._onImmediate) {
      try {
        if (immediate._args) {
          immediate._onImmediate(...immediate._args);
        } else {
          immediate._onImmediate();
        }
      } catch (e) {
        console.error('Immediate callback error:', e);
      }
    }
  }, 0);
  
  return immediate;
}

ObjectDefineProperty(setImmediate, 'promises', {
  __proto__: null,
  enumerable: true,
  configurable: true,
  get() {
    return require('timers/promises').setImmediate;
  },
});

function clearImmediate(immediate) {
  if (!immediate || !immediate._onImmediate || immediate._destroyed) return;
  
  immediate._destroyed = true;
  immediate._onImmediate = null;
  
  const idx = immediateQueue.indexOf(immediate);
  if (idx !== -1) {
    immediateQueue.splice(idx, 1);
  }
}

const timers = {
  setTimeout,
  clearTimeout,
  setImmediate,
  clearImmediate,
  setInterval,
  clearInterval,
  promisify: {
    setTimeout: null,
    setImmediate: null,
  },
};

ObjectDefineProperties(timers, {
  promises: {
    __proto__: null,
    configurable: true,
    enumerable: true,
    get() {
      return require('timers/promises');
    },
  },
});

module.exports = timers;