// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:diagnostics_channel — Koss 原生诊断频道标准库

var channels = {};

function Channel(name) {
  this.name = name;
  this._subscribers = new Set();
  this._store = {};
}

Channel.prototype.subscribe = function(subscriber) {
  if (typeof subscriber !== 'function') throw new TypeError('subscriber must be a function');
  this._subscribers.add(subscriber);
};

Channel.prototype.unsubscribe = function(subscriber) {
  this._subscribers.delete(subscriber);
};

Channel.prototype.hasSubscribers = function() {
  return this._subscribers.size > 0;
};

Channel.prototype.publish = function(data) {
  var iter = this._subscribers.values();
  var result = iter.next();
  while (!result.done) {
    try { result.value(data, this.name); } catch (e) { console.error('diagnostics_channel subscriber error:', e); }
    result = iter.next();
  }
};

Object.defineProperty(Channel.prototype, 'hasSubscribers', {
  get: function() { return this._subscribers.size > 0; }
});

Channel.prototype.bindStore = function(store) { this._store = store || {}; };
Channel.prototype.getStore = function() { return this._store; };

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

module.exports = { channel: channel, subscribe: subscribe, unsubscribe: unsubscribe, hasSubscribers: hasSubscribers, Channel: Channel };
