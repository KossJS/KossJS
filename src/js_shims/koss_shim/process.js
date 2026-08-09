// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:process — Koss 标准库进程模块
// 进程信息、环境变量、事件发射、内存/CPU使用等

var _system;
try { _system = require('koss:system'); } catch (e) { _system = null; }

var _nativeProcess = (typeof globalThis !== 'undefined' && globalThis.process) || null;

function _envObj() {
  if (_system) return _system.env();
  if (_nativeProcess && _nativeProcess.env) return _nativeProcess.env;
  return {};
}

function _cwd() {
  if (_system) return _system.cwd();
  if (_nativeProcess && typeof _nativeProcess.cwd === 'function') return _nativeProcess.cwd();
  return '.';
}

function _chdir(path) {
  if (_system) { _system.chdir(path); return; }
  if (_nativeProcess && typeof _nativeProcess.chdir === 'function') { _nativeProcess.chdir(path); }
}

function _exit(code) {
  if (_system) { _system.exit(code || 0); }
  if (_nativeProcess && typeof _nativeProcess.exit === 'function') { _nativeProcess.exit(code || 0); }
  throw new Error('Process exit: ' + (code || 0));
}

function _kill(pid, signal) {
  if (_nativeProcess && typeof _nativeProcess.kill === 'function') {
    return _nativeProcess.kill(pid, signal);
  }
  throw new Error('process.kill not available');
}

// ─── EventEmitter（复用 koss:events）───
var EventEmitter = require('koss:events');
var _ee = new EventEmitter();
_ee.setMaxListeners(16);

function _addListener(event, listener, prepend) {
  if (typeof listener !== 'function') throw new TypeError('listener must be a function');
  if (prepend) {
    _ee.prependListener(event, listener);
  } else {
    _ee.on(event, listener);
  }
  return kossProcess;
}

function _removeListener(event, listener) {
  _ee.removeListener(event, listener);
  return kossProcess;
}

function _emit(event) {
  return _ee.emit.apply(_ee, arguments);
}

function _listenerCount(event) {
  return _ee.listenerCount(event);
}

function _eventNames() { return _ee.eventNames(); }

function _setMaxListeners(n) {
  if (typeof n !== 'number' || n < 0) throw new RangeError('n must be a non-negative number');
  _ee.setMaxListeners(n);
  return kossProcess;
}

function _getMaxListeners() { return _ee.getMaxListeners(); }

function _once(event, listener) {
  if (typeof listener !== 'function') throw new TypeError('listener must be a function');
  _ee.once(event, listener);
  return kossProcess;
}

function _prependListener(event, listener) {
  return _addListener(event, listener, true);
}

function _prependOnceListener(event, listener) {
  if (typeof listener !== 'function') throw new TypeError('listener must be a function');
  _ee.prependOnceListener(event, listener);
  return kossProcess;
}

function _removeAllListeners(event) {
  _ee.removeAllListeners(event);
  return kossProcess;
}

function _listeners(event) {
  return _ee.listeners(event);
}

// ─── Stream stubs ───
function _writeStream(name) {
  return {
    isTTY: false,
    write: function(chunk, encoding, cb) {
      if (typeof encoding === 'function') { cb = encoding; encoding = undefined; }
      if (_nativeProcess && name === 'stdout' && _nativeProcess.stdout && _nativeProcess.stdout.write) {
        return _nativeProcess.stdout.write(chunk, encoding, cb);
      }
      if (_nativeProcess && name === 'stderr' && _nativeProcess.stderr && _nativeProcess.stderr.write) {
        return _nativeProcess.stderr.write(chunk, encoding, cb);
      }
      if (typeof cb === 'function') cb();
      return true;
    },
    end: function() {},
    on: function() { return this; },
    once: function() { return this; },
    emit: function() { return false; },
    pipe: function(dest) { return dest; },
  };
}

function _readStream(name) {
  return {
    isTTY: false,
    read: function() { return null; },
    on: function() { return this; },
    once: function() { return this; },
    emit: function() { return false; },
    pipe: function(dest) { return dest; },
  };
}

// ─── Memory / CPU ───
function _memoryUsage() {
  if (_system) {
    var m = _system.memory();
    return {
      rss: m.total || 0,
      heapTotal: m.total || 0,
      heapUsed: m.used || 0,
      external: 0,
      arrayBuffers: 0,
    };
  }
  if (_nativeProcess && typeof _nativeProcess.memoryUsage === 'function') {
    return _nativeProcess.memoryUsage();
  }
  return { rss: 0, heapTotal: 0, heapUsed: 0, external: 0, arrayBuffers: 0 };
}

function _cpuUsage(prev) {
  if (_nativeProcess && typeof _nativeProcess.cpuUsage === 'function') {
    return _nativeProcess.cpuUsage(prev);
  }
  var cur = { user: 0, system: 0 };
  if (prev) {
    return { user: cur.user - prev.user, system: cur.system - prev.system };
  }
  return cur;
}

function _uptime() {
  if (_system) return _system.uptime();
  if (_nativeProcess && typeof _nativeProcess.uptime === 'function') return _nativeProcess.uptime();
  return 0;
}

// ─── nextTick ───
function _nextTick(fn) {
  if (_nativeProcess && typeof _nativeProcess.nextTick === 'function') {
    _nativeProcess.nextTick(fn);
  } else if (typeof queueMicrotask === 'function') {
    queueMicrotask(fn);
  } else {
    Promise.resolve().then(fn);
  }
}

// ─── Platform info ───
function _platform() {
  if (_system) return _system.platform();
  if (_nativeProcess) return _nativeProcess.platform || 'unknown';
  return 'unknown';
}

function _arch() {
  if (_system) return _system.arch();
  if (_nativeProcess) return _nativeProcess.arch || 'unknown';
  return 'unknown';
}

function _version() {
  if (_system) return _system.version();
  if (_nativeProcess) return _nativeProcess.version || 'unknown';
  return 'unknown';
}

function _execPath() {
  if (_nativeProcess) return _nativeProcess.execPath || '';
  return '';
}

function _execArgv() {
  if (_nativeProcess && _nativeProcess.execArgv) return _nativeProcess.execArgv.slice();
  return [];
}

// ─── process object ───
var kossProcess = {};

// 核心属性
Object.defineProperty(kossProcess, 'argv', {
  get: function() {
    if (_nativeProcess && _nativeProcess.argv) return _nativeProcess.argv.slice();
    return [''];
  },
  enumerable: true,
});

Object.defineProperty(kossProcess, 'env', {
  get: function() {
    var obj = _envObj();
    var copy = {};
    for (var k in obj) { copy[k] = obj[k]; }
    return copy;
  },
  enumerable: true,
});

Object.defineProperty(kossProcess, 'pid', {
  get: function() {
    if (_nativeProcess) return _nativeProcess.pid || 0;
    return 0;
  },
  enumerable: true,
});

Object.defineProperty(kossProcess, 'ppid', {
  get: function() {
    if (_nativeProcess && _nativeProcess.ppid !== undefined) return _nativeProcess.ppid;
    return 0;
  },
  enumerable: true,
});

kossProcess.cwd = _cwd;
kossProcess.chdir = _chdir;
kossProcess.exit = _exit;
kossProcess.kill = _kill;

// 流
kossProcess.stdout = _writeStream('stdout');
kossProcess.stderr = _writeStream('stderr');
kossProcess.stdin = _readStream('stdin');

// 平台信息
kossProcess.platform = _platform();
kossProcess.arch = _arch();
kossProcess.version = _version();
kossProcess.versions = (_nativeProcess && _nativeProcess.versions) || {};
kossProcess.execPath = _execPath();
kossProcess.execArgv = _execArgv();

// 事件
kossProcess.on = function(event, listener) { return _addListener(event, listener, false); };
kossProcess.once = function(event, listener) { return _once(event, listener); };
kossProcess.off = function(event, listener) { return _removeListener(event, listener); };
kossProcess.emit = function(event) { return _emit.apply(null, arguments); };
kossProcess.removeAllListeners = _removeAllListeners;
kossProcess.listeners = _listeners;
kossProcess.rawListeners = _listeners;
kossProcess.listenerCount = _listenerCount;
kossProcess.eventNames = _eventNames;
kossProcess.setMaxListeners = _setMaxListeners;
kossProcess.getMaxListeners = _getMaxListeners;
kossProcess.prependListener = _prependListener;
kossProcess.prependOnceListener = _prependOnceListener;

// 方法
kossProcess.nextTick = _nextTick;
kossProcess.memoryUsage = _memoryUsage;
kossProcess.cpuUsage = _cpuUsage;
kossProcess.uptime = _uptime;

// 标题
kossProcess.title = (_nativeProcess && _nativeProcess.title) || 'koss';

// 特性
kossProcess.features = (_nativeProcess && _nativeProcess.features) || {};

// 信号常量
kossProcess.signals = {
  SIGHUP: 1, SIGINT: 2, SIGQUIT: 3, SIGILL: 4, SIGTRAP: 5,
  SIGABRT: 6, SIGBUS: 7, SIGFPE: 8, SIGKILL: 9, SIGUSR1: 10,
  SIGSEGV: 11, SIGUSR2: 12, SIGPIPE: 13, SIGALRM: 14, SIGTERM: 15,
};

module.exports = kossProcess;
module.exports.default = kossProcess;
