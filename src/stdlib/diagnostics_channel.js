'use strict';

var channels = {};

function hasSubscribers(name) {
  var channel = channels[name];
  return channel && channel._subscribers && channel._subscribers.length > 0;
}

function channel(name) {
  if (channels[name]) return channels[name];
  var ch = new Channel(name);
  channels[name] = ch;
  return ch;
}

class Channel {
  constructor(name) {
    this._name = name;
    this._subscribers = [];
  }

  get name() { return this._name; }
  get hasSubscribers() { return this._subscribers.length > 0; }

  subscribe(subscriber) {
    if (typeof subscriber !== 'function') {
      throw new TypeError('subscriber must be a function');
    }
    if (this._subscribers.indexOf(subscriber) === -1) {
      this._subscribers.push(subscriber);
    }
  }

  unsubscribe(subscriber) {
    var idx = this._subscribers.indexOf(subscriber);
    if (idx !== -1) {
      this._subscribers.splice(idx, 1);
    }
  }

  publish(data) {
    for (var i = 0; i < this._subscribers.length; i++) {
      try {
        this._subscribers[i](data, this._name);
      } catch (e) {
        if (typeof process !== 'undefined' && typeof process.nextTick === 'function') {
          process.nextTick(function() { throw e; });
        }
      }
    }
  }

  bindStore(store) {
    return store;
  }

  runStores(data, fn) {
    if (typeof fn === 'function') return fn();
    return undefined;
  }
}

class ActiveChannel extends Channel {
  constructor(name) {
    super(name);
    this._stores = new Map();
  }
}

var dc = {
  channel: channel,
  hasSubscribers: hasSubscribers,
  Channel: Channel,
  ActiveChannel: ActiveChannel,
  subscribe: function(name, subscriber) {
    return channel(name).subscribe(subscriber);
  },
  unsubscribe: function(name, subscriber) {
    var ch = channels[name];
    if (ch) ch.unsubscribe(subscriber);
  },
  _channel: function(name) {
    return channels[name];
  },
};

module.exports = dc;
