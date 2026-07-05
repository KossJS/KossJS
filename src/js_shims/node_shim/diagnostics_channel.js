// koss:node/diagnostics_channel - Node.js diagnostics_channel module (L3)

const channels = {};

class Channel {
  constructor(name) {
    this.name = name;
    this._subscribers = new Set();
    this._store = {};
  }

  subscribe(subscriber) {
    if (typeof subscriber !== 'function') throw new TypeError('subscriber must be a function');
    this._subscribers.add(subscriber);
  }

  unsubscribe(subscriber) {
    this._subscribers.delete(subscriber);
  }

  hasSubscribers() {
    return this._subscribers.size > 0;
  }

  publish(data) {
    for (const sub of this._subscribers) {
      try { sub(data, this.name); } catch (e) { console.error('diagnostics_channel subscriber error:', e); }
    }
  }

  get hasSubscribers() { return this._subscribers.size > 0; }

  bindStore(store) { this._store = store || {}; }
  getStore() { return this._store; }
}

function channel(name) {
  if (!channels[name]) channels[name] = new Channel(name);
  return channels[name];
}

function subscribe(name, subscriber) {
  return channel(name).subscribe(subscriber);
}

function unsubscribe(name, subscriber) {
  return channel(name).unsubscribe(subscriber);
}

function hasSubscribers(name) {
  if (!channels[name]) return undefined;
  return channels[name].hasSubscribers();
}

module.exports = { channel, subscribe, unsubscribe, hasSubscribers, Channel };
