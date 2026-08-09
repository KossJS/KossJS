// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:deno - Deno 运行时兼容层 (L3)
// Deno v2.0.x API alignment
// Maps to koss:io and koss:crypto standard libraries

var io = require('koss:io');
var kossCrypto = require('koss:crypto');
var kossSystem = require('koss:system');

var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;

var version = { deno: '2.0.6', v8: '12.9', typescript: '5.6' };
var args = [];
var pid = kossSystem.pid();
var noColor = true;

// === File System ===
function readTextFile(path) {
  return io.readText(path);
}

function writeTextFile(path, data) {
  io.writeText(path, data);
}

function readFile(path) {
  return io.read(path);
}

function writeFile(path, data) {
  io.write(path, data);
}

function stat(path) {
  return io.stat(path);
}

function lstat(path) {
  return io.lstat(path);
}

function mkdir(path, options) {
  try {
    io.mkdir(path, options);
  } catch (e) {
    var msg = e && e.message ? e.message : String(e);
    if (msg.indexOf('already exists') !== -1 || msg.indexOf('os error 183') !== -1 || msg.indexOf('os error 17') !== -1) return;
    throw e;
  }
}

function remove(path) {
  io.rm(path);
}

function rename(oldPath, newPath) {
  io.mv(oldPath, newPath);
}

function realPath(path) {
  return io.realpath(path);
}

function cwd() {
  return kossSystem.cwd();
}

function chdir(path) {
  kossSystem.chdir(path);
}

// === Network ===
function serve(handler, options) {
  options = options || {};
  var port = options.port || 8000;
  var hostname = options.hostname || '0.0.0.0';
  // 接线 handler：io.serve 现在支持带 handler 的完整 HTTP 服务器
  var server = io.serve({ port: port, hostname: hostname }, handler);
  return {
    port: Number(port),
    hostname: String(hostname),
    close: function() { server.close(); },
    shutdown: function() { server.close(); return Promise.resolve(); },
  };
}

function listen(options) {
  var port = (options && options.port) || 8000;
  var hostname = (options && options.hostname) || '0.0.0.0';
  return io.serve({ port: port, hostname: hostname });
}

function connect(options) {
  var hostname = (options && options.hostname) || 'localhost';
  var port = options && options.port;
  return io.connect(hostname, Number(port));
}

function resolveDns(host) {
  return io.dns(host);
}

// === Process ===
function exit(code) {
  kossSystem.exit(code || 0);
}

function memoryUsage() {
  var mem = kossSystem.memory();
  return {
    rss: mem.total || 0,
    heapTotal: mem.free || 0,
    heapUsed: mem.used || 0,
    external: 0,
  };
}

// === Env（Deno.Env API） ===
function _envSnapshot() {
  var env = kossSystem.env();
  var copy = {};
  for (var k in env) copy[k] = env[k];
  return copy;
}
var Env = {
  get: function(key) {
    var env = _envSnapshot();
    return Object.prototype.hasOwnProperty.call(env, key) ? env[key] : undefined;
  },
  set: function(key, value) {
    kossSystem.setEnv ? kossSystem.setEnv(key, value) : undefined;
    var env = kossSystem.env();
    if (env && typeof env === 'object') env[key] = value;
  },
  delete: function(key) {
    kossSystem.deleteEnv ? kossSystem.deleteEnv(key) : undefined;
    var env = kossSystem.env();
    if (env && typeof env === 'object') delete env[key];
  },
  toObject: function() { return _envSnapshot(); },
};

// === 文件同步族 ===
function readTextFileSync(path) { return io.readText(path); }
function readFileSync(path) { return io.read(path); }
function writeTextFileSync(path, data) { io.writeText(path, data); }
function writeFileSync(path, data) { io.write(path, data); }
function mkdirSync(path, options) {
  try { io.mkdir(path, options); }
  catch (e) {
    var msg = e && e.message ? e.message : String(e);
    if (msg.indexOf('already exists') !== -1 || msg.indexOf('os error 183') !== -1 || msg.indexOf('os error 17') !== -1) return;
    throw e;
  }
}
function removeSync(path, options) { io.rm(path, options); }
function renameSync(oldPath, newPath) { io.mv(oldPath, newPath); }
function statSync(path) { return io.stat(path); }
function lstatSync(path) { return io.lstat(path); }
function realPathSync(path) { return io.realpath(path); }
function cwdSync() { return kossSystem.cwd(); }
function chdirSync(path) { kossSystem.chdir(path); }
function copyFileSync(from, to) { io.cp(from, to); }
function symlinkSync(target, path, type) { throw new Error('Deno.symlinkSync is not implemented in KossJS'); }
function existsSync(path) { return io.exists(path); }

// === open/close/seek/readAll/writeAll ===
function open(path, options) {
  var opts = options || {};
  var flags;
  if (opts.read && opts.write) flags = 'r+';
  else if (opts.write && opts.append) flags = 'a';
  else if (opts.write) flags = 'w';
  else if (opts.read) flags = 'r';
  else if (opts.create) flags = 'w';
  else flags = 'r';
  var fsMod = require('node:fs');
  var fd = fsMod.openSync(path, flags);
  var f = new FsFile(fd, path);
  return Promise.resolve(f);
}
function FsFile(fd, path) {
  this.rid = fd;
  this.path = path;
}
FsFile.prototype.read = function(buffer) {
  var self = this;
  return new Promise(function(resolve) {
    var fsMod = require('node:fs');
    var n = fsMod.readSync(self.rid, buffer, 0, buffer.length);
    resolve(n === 0 ? null : n);
  });
};
FsFile.prototype.write = function(data) {
  var self = this;
  return new Promise(function(resolve) {
    var fsMod = require('node:fs');
    var n = fsMod.writeSync(self.rid, data);
    resolve(n);
  });
};
FsFile.prototype.close = function() {
  var fsMod = require('node:fs');
  try { fsMod.closeSync(this.rid); } catch (e) {}
  return Promise.resolve();
};
FsFile.prototype.seek = function(offset, whence) {
  // 当前实现基于 fd，seek 通过 position 参数支持有限
  return Promise.resolve(offset);
};
FsFile.prototype.stat = function() {
  var self = this;
  return new Promise(function(resolve) {
    var fsMod = require('node:fs');
    resolve(fsMod.fstatSync(self.rid));
  });
};
FsFile.prototype[Symbol.asyncDispose] = function() {
  return this.close();
};

function readAll(file) {
  return new Promise(function(resolve) {
    var chunks = [];
    function readNext() {
      file.read(new Uint8Array(65536)).then(function(n) {
        if (n === null) {
          var total = 0;
          for (var i = 0; i < chunks.length; i++) total += chunks[i].length;
          var out = new Uint8Array(total);
          var off = 0;
          for (var j = 0; j < chunks.length; j++) { out.set(chunks[j], off); off += chunks[j].length; }
          resolve(out);
          return;
        }
        var buf = new Uint8Array(65536);
        // 从 file.read 已填充的 buffer 读取
        chunks.push(buf.subarray(0, n));
        readNext();
      });
    }
    readNext();
  });
}

function writeAll(file, data) {
  var bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return file.write(bytes);
}

function readTextFileSyncAlias(path) { return readTextFileSync(path); }

// === kill ===
function kill(pid, signal) {
  if (kossSystem.kill) {
    kossSystem.kill(pid, signal);
    return;
  }
  throw new Error('Deno.kill is not implemented in KossJS');
}

// === Timers ===
var setTimeout = globalThis.setTimeout;
var clearTimeout = globalThis.clearTimeout;
var setInterval = globalThis.setInterval;
var clearInterval = globalThis.clearInterval;

// === Crypto ===
function _toBytes(data) {
  if (data instanceof Uint8Array) return data;
  if (Array.isArray(data)) return new Uint8Array(data);
  if (typeof data === 'string') {
    var bytes = new Uint8Array(data.length);
    for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
    return bytes;
  }
  return new Uint8Array(0);
}

var cryptoObj = {
  getRandomValues: function(arr) {
    var bytes = kossCrypto.randomBytes(arr.length);
    for (var i = 0; i < arr.length && i < bytes.length; i++) arr[i] = bytes[i];
    return arr;
  },
  randomUUID: function() { return kossCrypto.uuid(); },
  subtle: {
    digest: async function(algorithm, data) {
      var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'SHA-256';
      var normalized = algo.toLowerCase().replace('-', '');
      return kossCrypto.hash(normalized, _toBytes(data));
    },
    encrypt: async function(algorithm, key, data) {
      var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'AES-GCM';
      var keyBytes = _toBytes(key);
      var ptBytes = _toBytes(data);
      var combined = kossCrypto.encrypt(keyBytes, ptBytes);
      return combined;
    },
    decrypt: async function(algorithm, key, data) {
      var ctBytes = _toBytes(data);
      return kossCrypto.decrypt(_toBytes(key), ctBytes);
    },
    generateKey: async function(algorithm) {
      var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'Ed25519';
      if (algo === 'Ed25519' || algo === 'ed25519') {
        var kp = kossCrypto.ed25519KeyPair();
        return kp;
      }
      var keyLen = (algorithm && algorithm.length) || 32;
      return kossCrypto.randomBytes(keyLen);
    },
    sign: async function(algorithm, key, data) {
      return kossCrypto.sign(_toBytes(key), _toBytes(data));
    },
    verify: async function(algorithm, key, signature, data) {
      return kossCrypto.verify(_toBytes(key), _toBytes(data), _toBytes(signature));
    },
  },
};

// === Errors ===
var errors = {
  NotFound: function(msg) { this.message = msg || 'Not found'; this.name = 'NotFound'; },
  PermissionDenied: function(msg) { this.message = msg || 'Permission denied'; this.name = 'PermissionDenied'; },
  ConnectionRefused: function(msg) { this.message = msg || 'Connection refused'; this.name = 'ConnectionRefused'; },
  AlreadyExists: function(msg) { this.message = msg || 'Already exists'; this.name = 'AlreadyExists'; },
  BadResource: function(msg) { this.message = msg || 'Bad resource'; this.name = 'BadResource'; },
  BrokenPipe: function(msg) { this.message = msg || 'Broken pipe'; this.name = 'BrokenPipe'; },
  ConnectionAborted: function(msg) { this.message = msg || 'Connection aborted'; this.name = 'ConnectionAborted'; },
  ConnectionReset: function(msg) { this.message = msg || 'Connection reset'; this.name = 'ConnectionReset'; },
  InvalidData: function(msg) { this.message = msg || 'Invalid data'; this.name = 'InvalidData'; },
  TimedOut: function(msg) { this.message = msg || 'Timed out'; this.name = 'TimedOut'; },
  Interrupted: function(msg) { this.message = msg || 'Interrupted'; this.name = 'Interrupted'; },
  WriteZero: function(msg) { this.message = msg || 'Write zero'; this.name = 'WriteZero'; },
  UnexpectedEof: function(msg) { this.message = msg || 'Unexpected EOF'; this.name = 'UnexpectedEof'; },
  Other: function(msg) { this.message = msg || 'Other error'; this.name = 'Other'; },
};

// === Signals ===
var signals = {
  SIGABRT: 6, SIGALRM: 14, SIGBUS: 7, SIGCHLD: 17, SIGCONT: 19,
  SIGFPE: 8, SIGHUP: 1, SIGILL: 4, SIGINT: 2, SIGKILL: 9,
  SIGPIPE: 13, SIGQUIT: 3, SIGSEGV: 11, SIGSTOP: 17, SIGTERM: 15,
  SIGTSTP: 20, SIGTTIN: 21, SIGTTOU: 22, SIGUSR1: 10, SIGUSR2: 12,
  SIGPROF: 27, SIGSYS: 31, SIGTRAP: 5, SIGURG: 23, SIGVTALRM: 26,
  SIGXCPU: 24, SIGXFSZ: 25, SIGWINCH: 28, SIGIO: 29, SIGPWR: 30,
  SIGSTKFLT: 16,
};

// === Not implemented ===
function run() { throw new Error('Deno.run is not implemented in KossJS'); }
function spawn() { throw new Error('Deno.spawn is not implemented in KossJS'); }
function permissions() { throw new Error('Deno.permissions is not implemented in KossJS (use Capability bits)'); }

module.exports = {
  version: version,
  env: kossSystem.env(),
  args: args,
  pid: pid,
  noColor: noColor,
  readTextFile: readTextFile,
  writeTextFile: writeTextFile,
  readFile: readFile,
  writeFile: writeFile,
  stat: stat,
  lstat: lstat,
  mkdir: mkdir,
  remove: remove,
  rename: rename,
  realPath: realPath,
  cwd: cwd,
  chdir: chdir,
  serve: serve,
  listen: listen,
  connect: connect,
  resolveDns: resolveDns,
  exit: exit,
  memoryUsage: memoryUsage,
  setTimeout: setTimeout,
  clearTimeout: clearTimeout,
  setInterval: setInterval,
  clearInterval: clearInterval,
  crypto: cryptoObj,
  errors: errors,
  signals: signals,
  run: run,
  spawn: spawn,
  permissions: permissions,
  // 新增
  Env: Env,
  kill: kill,
  readTextFileSync: readTextFileSync,
  readFileSync: readFileSync,
  writeTextFileSync: writeTextFileSync,
  writeFileSync: writeFileSync,
  mkdirSync: mkdirSync,
  removeSync: removeSync,
  renameSync: renameSync,
  statSync: statSync,
  lstatSync: lstatSync,
  realPathSync: realPathSync,
  cwdSync: cwdSync,
  chdirSync: chdirSync,
  copyFileSync: copyFileSync,
  symlinkSync: symlinkSync,
  existsSync: existsSync,
  open: open,
  readAll: readAll,
  writeAll: writeAll,
  FsFile: FsFile,
};
