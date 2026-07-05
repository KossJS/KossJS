// koss:node/timers - Node.js timers module (L3)
// Wraps global setTimeout/setInterval

function setTimeout(callback, delay, ...args) {
  return globalThis.setTimeout(() => callback(...args), delay || 1);
}

function clearTimeout(id) { globalThis.clearTimeout(id); }

function setInterval(callback, delay, ...args) {
  return globalThis.setInterval(() => callback(...args), delay || 1);
}

function clearInterval(id) { globalThis.clearInterval(id); }

function setImmediate(callback, ...args) {
  return globalThis.setTimeout(() => callback(...args), 0);
}

function clearImmediate(id) { globalThis.clearTimeout(id); }

function active(timer) {}

function enroll(timer, delay) {}

function unenroll(timer) {}

const promises = {
  setTimeout: function(delay, value, options) {
    return new Promise((resolve) => {
      globalThis.setTimeout(() => resolve(value), delay || 1);
    });
  },
  setImmediate: function(value) {
    return new Promise((resolve) => {
      globalThis.setTimeout(() => resolve(value), 0);
    });
  },
  setInterval: function(delay, value, options) {
    const iter = {
      [Symbol.asyncIterator]() {
        return {
          _id: null,
          _resolve: null,
          _value: value,
          _delay: delay,
          next() {
            return new Promise((resolve) => {
              this._resolve = resolve;
              this._id = globalThis.setTimeout(() => {
                resolve({ value: this._value, done: false });
              }, this._delay);
            });
          },
          return() {
            if (this._id) globalThis.clearTimeout(this._id);
            return Promise.resolve({ value: undefined, done: true });
          }
        };
      }
    };
    return iter;
  },
  scheduler: {
    wait: function(delay) {
      return new Promise((resolve) => globalThis.setTimeout(resolve, delay || 1));
    },
    yield: function() {
      return new Promise((resolve) => globalThis.setTimeout(resolve, 0));
    },
  },
};

module.exports = { setTimeout, clearTimeout, setInterval, clearInterval, setImmediate, clearImmediate, active, enroll, unenroll, promises };
