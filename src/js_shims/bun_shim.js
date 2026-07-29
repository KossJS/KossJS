// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:bun - Bun 运行时兼容层 (L3)
// Bun v1.1.x API alignment
// Maps to koss:io and koss:crypto standard libraries

var io = require('koss:io');
var kossCrypto = require('koss:crypto');
var kossSystem = require('koss:system');

var Buffer = globalThis.Buffer;

var version = '1.1.42';
var build = 'koss-bun-compat';

function write(path, data) {
  if (typeof path !== 'string') {
    throw new Error('Bun.write with file descriptor not supported in KossJS');
  }
  io.write(path, data);
}

function file(path) {
  return {
    path: path,
    size: function() {
      var s = io.stat(path);
      return (s && s.size) || 0;
    },
    text: function() {
      return io.readText(path);
    },
    json: function() {
      return JSON.parse(this.text());
    },
    arrayBuffer: function() {
      var data = io.read(path);
      if (data instanceof ArrayBuffer) return data;
      if (data instanceof Uint8Array) return data.buffer;
      if (data && data.buffer) return data.buffer;
      return new Uint8Array(0).buffer;
    },
    stream: function() {
      throw new Error('ReadableStream is not supported in KossJS (Boa 0.21.x)');
    },
    exists: function() {
      return io.exists(path);
    },
  };
}

function serve(options) {
  var port = (options && options.port) || 3000;
  var hostname = (options && options.hostname) || '0.0.0.0';
  var server = io.serve({ port: port, hostname: hostname });
  return {
    port: Number(port),
    hostname: String(hostname),
    stop: function() { server.close(); },
    reload: function(options) { /* no-op for now */ },
    ref: function() {},
    unref: function() {},
  };
}

function sleep(ms) {
  return new Promise(function(resolve) { setTimeout(resolve, ms); });
}

function inspect(value) {
  return JSON.stringify(value, function(key, val) {
    if (typeof val === 'bigint') return 'BigInt(' + val + ')';
    if (val && val.constructor && val.constructor.name === 'NodeBuffer') return 'Buffer(' + val.length + ')';
    return val;
  }, 2);
}

function peek(iterable) {
  if (iterable && typeof iterable[Symbol.iterator] === 'function') {
    var iterator = iterable[Symbol.iterator]();
    var first = iterator.next();
    return first.done ? undefined : first.value;
  }
  return undefined;
}

function which(cmd) {
  return cmd || null;
}

function randomUUIDv7() {
  return kossCrypto.uuid();
}

function resolvePath(path) {
  return io.readText ? path : path;
}

function readable(path) {
  throw new Error('ReadableStream is not supported in KossJS (Boa 0.21.x)');
}

function hash(algorithm, data) {
  return kossCrypto.hashHex(algorithm, data);
}

function malloc(size) { throw new Error('Bun malloc is not implemented in KossJS'); }
function gc() { throw new Error('Bun.gc is not implemented in KossJS'); }

// === Not implemented ===
function sql() { throw new Error('Bun.sql is not implemented in KossJS (requires SQLite)'); }
function spawn() { throw new Error('Bun.spawn is not implemented in KossJS (requires child_process)'); }
function buildFn() { throw new Error('Bun.build is not implemented in KossJS (no bundler)'); }

module.exports = {
  version: version,
  build: build,
  env: kossSystem.env(),
  argv: [],
  write: write,
  file: file,
  serve: serve,
  sleep: sleep,
  inspect: inspect,
  peek: peek,
  which: which,
  randomUUIDv7: randomUUIDv7,
  resolve: resolvePath,
  readable: readable,
  hash: hash,
  malloc: malloc,
  gc: gc,
  sql: sql,
  spawn: spawn,
  build: buildFn,
};
