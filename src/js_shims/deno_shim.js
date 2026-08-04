// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:deno - Deno 运行时兼容层 (L3)
// Deno v2.0.x API alignment
// Maps to koss:io and koss:crypto standard libraries

var io = require('koss:io');
var kossCrypto = require('koss:crypto');
var kossSystem = require('koss:system');

var Buffer = globalThis.Buffer;

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
    if (msg.indexOf('already exists') !== -1 || msg.indexOf('os error 183') !== -1) return;
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
  return path;
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
  var server = io.serve({ port: port, hostname: hostname });
  return {
    port: Number(port),
    hostname: String(hostname),
    close: function() { server.close(); },
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
      var result = kossCrypto.encrypt(keyBytes, ptBytes);
      return result.ciphertext;
    },
    decrypt: async function(algorithm, key, data) {
      var ctBytes = _toBytes(data);
      return kossCrypto.decrypt(_toBytes(key), ctBytes);
    },
    generateKey: async function(algorithm) {
      var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'Ed25519';
      if (algo === 'Ed25519' || algo === 'ed25519') {
        var kp = kossCrypto.internalCrypto.ed25519KeyPair();
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
};
