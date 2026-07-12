// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:deno - Deno 运行时兼容层 (L3)
// Deno v2.0.x API alignment

var internalFs = require('koss:internal/fs');
var internalNet = require('koss:internal/net');
var internalCrypto = require('koss:internal/crypto');

var Buffer = globalThis.Buffer;
var process = globalThis.process;

var version = { deno: '2.0.6', v8: '12.9', typescript: '5.6' };
var env = (process && process.env) || {};
var args = (process && process.argv && process.argv.slice(2)) || [];
var pid = (process && process.pid) || 0;
var noColor = true;

// === File System ===
function readTextFile(path) {
  return internalFs.readFileSyncUtf8(path);
}

function writeTextFile(path, data) {
  if (typeof data === 'string') {
    return internalFs.writeFileSync(path, data);
  }
  if (data instanceof Uint8Array) {
    var chars = [];
    for (var i = 0; i < data.length; i++) chars.push(String.fromCharCode(data[i]));
    return internalFs.writeFileSync(path, chars.join(''));
  }
  return internalFs.writeFileSync(path, String(data));
}

function readFile(path) {
  return internalFs.readFileSync(path);
}

function writeFile(path, data) {
  if (typeof data === 'string') {
    return internalFs.writeFileSync(path, data);
  }
  if (data instanceof Uint8Array) {
    var chars = [];
    for (var i = 0; i < data.length; i++) chars.push(String.fromCharCode(data[i]));
    return internalFs.writeFileSync(path, chars.join(''));
  }
  return internalFs.writeFileSync(path, String(data));
}

function stat(path) {
  return internalFs.statSync(path);
}

function lstat(path) {
  return internalFs.statSync(path);
}

function mkdir(path, options) {
  try {
    internalFs.mkdirSync(path);
    return Promise.resolve(undefined);
  } catch (err) {
    if (options && options.recursive && err.message && err.message.includes('EEXIST')) {
      return Promise.resolve(undefined);
    }
    return Promise.reject(err);
  }
}

function remove(path) {
  try {
    internalFs.unlinkSync(path);
    return Promise.resolve();
  } catch (err) {
    return Promise.reject(err);
  }
}

function rename(oldPath, newPath) {
  try {
    internalFs.renameSync(oldPath, newPath);
    return Promise.resolve();
  } catch (err) {
    return Promise.reject(err);
  }
}

function realPath(path) {
  return internalFs.realpathSync(path);
}

function cwd() {
  return internalFs.realpathSync('.');
}

function chdir(path) {
  try {
    internalFs.realpathSync(path);
    return Promise.resolve();
  } catch (err) {
    return Promise.reject(err);
  }
}

// === Network ===
function serve(handler, options) {
  options = options || {};
  var port = options.port || 8000;
  var hostname = options.hostname || '0.0.0.0';
  var server = internalNet.tcpListen(hostname, port);
  return {
    port: Number(port),
    hostname: String(hostname),
    close: function() { server.close(); },
  };
}

function listen(options) {
  var port = (options && options.port) || 8000;
  var hostname = (options && options.hostname) || '0.0.0.0';
  return internalNet.tcpListen(hostname, port);
}

function connect(options) {
  var hostname = (options && options.hostname) || 'localhost';
  var port = options && options.port;
  return internalNet.tcpConnect(hostname, Number(port));
}

function resolveDns(host) {
  return internalNet.dnsLookup(host);
}

// === Process ===
function exit(code) {
  code = code || 0;
  if (process && process.exit) process.exit(code);
  throw new Error('Process exit: ' + code);
}

function memoryUsage() {
  if (process && process.memoryUsage) return process.memoryUsage();
  return { rss: 0, heapTotal: 0, heapUsed: 0, external: 0 };
}

// === Timers ===
var setTimeout = globalThis.setTimeout;
var clearTimeout = globalThis.clearTimeout;
var setInterval = globalThis.setInterval;
var clearInterval = globalThis.clearInterval;

// === Crypto ===
var cryptoObj = {
  getRandomValues: function(arr) {
    var bytes = internalCrypto.randomBytes(arr.length);
    for (var i = 0; i < arr.length && i < bytes.length; i++) arr[i] = bytes[i];
    return arr;
  },
  randomUUID: function() { return internalCrypto.randomUUID(); },
  subtle: {
    digest: async function(algorithm, data) {
      var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'SHA-256';
      var dataStr = typeof data === 'string' ? data : new TextDecoder().decode(data);
      return internalCrypto.hash(algo, dataStr);
    },
  },
};

// === Not implemented ===
function run() { throw new Error('Deno.run is not implemented in KossJS'); }
function spawn() { throw new Error('Deno.spawn is not implemented in KossJS'); }
function permissions() { throw new Error('Deno.permissions is not implemented in KossJS (use Capability bits)'); }
var errors = {};
var signals = {};

module.exports = {
  version: version,
  env: env,
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
  run: run,
  spawn: spawn,
  permissions: permissions,
  errors: errors,
  signals: signals,
};
