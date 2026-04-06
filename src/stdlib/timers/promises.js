'use strict';

const {
  Promise,
  PromiseReject,
  Symbol,
} = globalThis;

const {
  Timeout,
  Immediate,
  insert,
} = require('internal/timers');
const {
  clearImmediate,
  clearInterval,
  clearTimeout,
} = require('timers');

const kScheduler = Symbol('kScheduler');

class AbortError extends Error {
  constructor(message, options) {
    super(message, options);
    this.name = 'AbortError';
  }
}

function cancelListenerHandler(clear, reject, signal) {
  if (!this._destroyed) {
    clear(this);
    reject(new AbortError(undefined, { cause: signal?.reason }));
  }
}

function setTimeout(after, value, options = {}) {
  const signal = options?.signal;
  const ref = options?.ref !== false;

  if (signal?.aborted) {
    return Promise.reject(new AbortError(undefined, { cause: signal.reason }));
  }

  return new Promise((resolve, reject) => {
    const timeout = new Timeout(resolve, after, [value], false, ref);
    insert(timeout, timeout._idleTimeout);
    
    if (signal) {
      const oncancel = cancelListenerHandler.bind(timeout, clearTimeout, reject, signal);
      signal.addEventListener('abort', oncancel, { once: true });
    }
  });
}

function setImmediate(value, options = {}) {
  const signal = options?.signal;
  const ref = options?.ref !== false;

  if (signal?.aborted) {
    return Promise.reject(new AbortError(undefined, { cause: signal.reason }));
  }

  return new Promise((resolve, reject) => {
    const immediate = new Immediate(resolve, [value]);
    if (!ref) immediate.unref();
    
    if (signal) {
      const oncancel = cancelListenerHandler.bind(immediate, clearImmediate, reject, signal);
      signal.addEventListener('abort', oncancel, { once: true });
    }
  });
}

async function* setInterval(after, value, options = {}) {
  const signal = options?.signal;
  const ref = options?.ref !== false;

  if (signal?.aborted) {
    throw new AbortError(undefined, { cause: signal.reason });
  }

  let interval;
  let resolver;
  let notYielded = 0;
  
  try {
    interval = new Timeout(() => {
      notYielded++;
      if (resolver) {
        resolver();
        resolver = undefined;
      }
    }, after, undefined, true, ref);
    insert(interval, interval._idleTimeout);

    while (!signal?.aborted) {
      if (notYielded === 0) {
        await new Promise((resolve) => { resolver = resolve; });
      }
      for (; notYielded > 0; notYielded--) {
        yield value;
      }
    }
    throw new AbortError(undefined, { cause: signal?.reason });
  } finally {
    if (interval) clearInterval(interval);
  }
}

class Scheduler {
  constructor() {
    throw new TypeError('Illegal constructor');
  }

  static yield() {
    return setImmediate();
  }

  static wait(delay, options) {
    return setTimeout(delay, undefined, options);
  }
}

const scheduler = new Proxy(Scheduler, {
  construct(target, args) {
    return new target();
  }
});

module.exports = {
  setTimeout,
  setImmediate,
  setInterval,
  scheduler,
};