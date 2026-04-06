'use strict';

const {
  MathMax,
  MathTrunc,
  NumberIsFinite,
  NumberIsNaN,
  NumberMIN_SAFE_INTEGER,
  Symbol,
  Promise,
  setTimeout: jsSetTimeout,
  clearTimeout: jsClearTimeout,
} = globalThis;

const async_id_symbol = Symbol('asyncId');
const trigger_async_id_symbol = Symbol('triggerId');
const kHasPrimitive = Symbol('kHasPrimitive');
const kRefed = Symbol('refed');

const knownTimersById = {};

const TIMEOUT_MAX = 2 ** 31 - 1;

let timerListId = NumberMIN_SAFE_INTEGER;
let nextExpiry = Infinity;

let warnedNegativeNumber = false;
let warnedNotNumber = false;

const timerListMap = {};
const timerListQueue = [];

class Timeout {
  constructor(callback, after, args, isRepeat, isRefed) {
    if (after === undefined) {
      after = 1;
    } else {
      after *= 1;
    }

    if (!(after >= 1 && after <= TIMEOUT_MAX)) {
      if (after > TIMEOUT_MAX) {
        after = 1;
      } else if (after < 0 && !warnedNegativeNumber) {
        warnedNegativeNumber = true;
        after = 1;
      } else if (NumberIsNaN(after) && !warnedNotNumber) {
        warnedNotNumber = true;
        after = 1;
      }
      after = 1;
    }

    this._idleTimeout = after;
    this._idlePrev = this;
    this._idleNext = this;
    this._idleStart = null;
    this._onTimeout = callback;
    this._timerArgs = args;
    this._repeat = isRepeat ? after : null;
    this._destroyed = false;
    this[kRefed] = isRefed;
    this[kHasPrimitive] = false;
    
    this[async_id_symbol] = Date.now() + Math.random();
    this[trigger_async_id_symbol] = 0;
  }

  refresh() {
    return this;
  }

  unref() {
    return this;
  }

  ref() {
    return this;
  }

  hasRef() {
    return this[kRefed];
  }
}

class TimersList {
  constructor(expiry, msecs) {
    this._idleNext = this;
    this._idlePrev = this;
    this.expiry = expiry;
    this.id = timerListId++;
    this.msecs = msecs;
    this.priorityQueuePosition = null;
  }
}

class ImmediateList {
  constructor() {
    this.head = null;
    this.tail = null;
  }

  append(item) {
    if (this.tail !== null) {
      this.tail._idleNext = item;
      item._idlePrev = this.tail;
    } else {
      this.head = item;
    }
    this.tail = item;
  }

  remove(item) {
    if (item._idleNext) {
      item._idleNext._idlePrev = item._idlePrev;
    }
    if (item._idlePrev) {
      item._idlePrev._idleNext = item._idleNext;
    }
    if (item === this.head) {
      this.head = item._idleNext;
    }
    if (item === this.tail) {
      this.tail = item._idlePrev;
    }
    item._idleNext = null;
    item._idlePrev = null;
  }
}

const immediateQueue = new ImmediateList();

function active(item) {
  insert(item, item._idleTimeout);
}

function unrefActive(item) {
  insert(item, item._idleTimeout);
}

function insert(item, msecs, start = Date.now()) {
  msecs = MathTrunc(msecs);
  item._idleStart = start;

  let list = timerListMap[msecs];
  if (list === undefined) {
    const expiry = start + msecs;
    timerListMap[msecs] = list = new TimersList(expiry, msecs);
    timerListQueue.push(list);
    timerListQueue.sort((a, b) => a.expiry - b.expiry);
  }

  list._idlePrev = item;
  item._idleNext = list._idleNext;
  if (list._idleNext) {
    list._idleNext._idlePrev = item;
  }
  list._idleNext = item;
  item._idlePrev = list;
}

function setUnrefTimeout(callback, after) {
  const timer = new Timeout(callback, after, undefined, false, false);
  insert(timer, timer._idleTimeout);
  return timer;
}

function compareTimersLists(a, b) {
  return a.expiry - b.expiry;
}

function getTimerCallbacks(runNextTicks) {
  const outstandingQueue = new ImmediateList();

  function processImmediate() {
    const queue = outstandingQueue.head !== null ? outstandingQueue : immediateQueue;
    let immediate = queue.head;

    if (queue !== outstandingQueue) {
      queue.head = queue.tail = null;
    }

    while (immediate !== null) {
      if (immediate._destroyed) {
        immediate = immediate._idleNext;
        continue;
      }

      immediate._destroyed = true;

      try {
        const argv = immediate._argv;
        if (!argv) {
          immediate._onImmediate();
        } else {
          immediate._onImmediate(...argv);
        }
      } finally {
        immediate._onImmediate = null;
      }

      immediate = immediate._idleNext;
    }
  }

  function processTimers(now) {
    nextExpiry = Infinity;
    let list;
    while ((list = timerListQueue[0]) != null) {
      if (list.expiry > now) {
        nextExpiry = list.expiry;
        return nextExpiry;
      }
      listOnTimeout(list, now);
    }
    return 0;
  }

  function listOnTimeout(list, now) {
    const msecs = list.msecs;
    let timer = list._idleNext;

    while (timer && timer !== list) {
      const diff = now - timer._idleStart;

      if (diff < msecs) {
        break;
      }

      const nextTimer = timer._idleNext;
      
      timer._idlePrev._idleNext = timer._idleNext;
      if (timer._idleNext) {
        timer._idleNext._idlePrev = timer._idlePrev;
      }

      if (!timer._onTimeout) {
        timer._destroyed = true;
        if (timer[kHasPrimitive]) {
          delete knownTimersById[timer[async_id_symbol]];
        }
        timer = nextTimer;
        continue;
      }

      try {
        const args = timer._timerArgs;
        if (args === undefined) {
          timer._onTimeout();
        } else {
          timer._onTimeout(...args);
        }
      } finally {
        if (timer._repeat && timer._idleTimeout !== -1) {
          timer._idleTimeout = timer._repeat;
          insert(timer, timer._idleTimeout);
        } else {
          timer._destroyed = true;
          if (timer[kHasPrimitive]) {
            delete knownTimersById[timer[async_id_symbol]];
          }
        }
      }

      timer = nextTimer;
    }

    if (!list._idleNext || list._idleNext === list) {
      delete timerListMap[list.msecs];
      const idx = timerListQueue.indexOf(list);
      if (idx !== -1) {
        timerListQueue.splice(idx, 1);
      }
    }
  }

  return {
    processImmediate,
    processTimers,
  };
}

class Immediate {
  constructor(callback, args) {
    this._idleNext = null;
    this._idlePrev = null;
    this._onImmediate = callback;
    this._argv = args;
    this._destroyed = false;
    this[kRefed] = false;
    
    this[async_id_symbol] = Date.now() + Math.random();
    this[trigger_async_id_symbol] = 0;

    this.ref();
    immediateQueue.append(this);
  }

  ref() {
    return this;
  }

  unref() {
    return this;
  }

  hasRef() {
    return !!this[kRefed];
  }
}

module.exports = {
  TIMEOUT_MAX,
  kTimeout: Symbol('timeout'),
  async_id_symbol,
  trigger_async_id_symbol,
  Timeout,
  Immediate,
  kRefed,
  kHasPrimitive,
  setUnrefTimeout,
  immediateQueue,
  getTimerCallbacks,
  immediateInfoFields: {
    kCount: 0,
    kRefCount: 1,
    kHasOutstanding: 2,
  },
  active,
  unrefActive,
  insert,
  timerListMap,
  timerListQueue,
  knownTimersById,
};