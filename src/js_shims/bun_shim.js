// koss:bun - Bun 运行时兼容层 (L3)
// Bun v1.1.x API alignment

var internalFs = require('koss:internal/fs');
var internalNet = require('koss:internal/net');
var internalCrypto = require('koss:internal/crypto');

var Buffer = globalThis.Buffer;
var process = globalThis.process;

var version = '1.1.42';
var build = 'koss-bun-compat';
var env = (process && process.env) || {};
var argv = (process && process.argv) || [];

function write(path, data) {
  if (typeof path !== 'string') {
    throw new Error('Bun.write with file descriptor not supported in KossJS');
  }
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

function file(path) {
  return {
    path: path,
    size: function() {
      var stat = internalFs.statSync(path);
      return (stat && stat.size) || 0;
    },
    text: function() {
      return internalFs.readFileSyncUtf8(path);
    },
    json: function() {
      return JSON.parse(this.text());
    },
    arrayBuffer: function() {
      var data = internalFs.readFileSync(path);
      if (data instanceof ArrayBuffer) return data;
      if (data && data.buffer) return data.buffer;
      var str = typeof data === 'string' ? data : String(data);
      var buf = new ArrayBuffer(str.length);
      var view = new Uint8Array(buf);
      for (var i = 0; i < str.length; i++) view[i] = str.charCodeAt(i) & 0xff;
      return buf;
    },
    stream: function() {
      throw new Error('ReadableStream is not supported in KossJS (Boa 0.21.x)');
    },
    exists: function() {
      try {
        internalFs.statSync(path);
        return true;
      } catch(e) { return false; }
    },
  };
}

function serve(options) {
  var port = (options && options.port) || 3000;
  var hostname = (options && options.hostname) || '0.0.0.0';
  var server = internalNet.tcpListen(hostname, port);
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
  try { return internalCrypto.randomUUID(); }
  catch(e) { return crypto.randomUUID(); }
}

function resolvePath(path) {
  try { return internalFs.realpathSync(path); }
  catch(e) { return path; }
}

function readable(path) {
  throw new Error('ReadableStream is not supported in KossJS (Boa 0.21.x)');
}

// === Not implemented ===
function sql() { throw new Error('Bun.sql is not implemented in KossJS (requires SQLite)'); }
function spawn() { throw new Error('Bun.spawn is not implemented in KossJS (requires child_process)'); }
function buildFn() { throw new Error('Bun.build is not implemented in KossJS (no bundler)'); }

module.exports = {
  version: version,
  build: build,
  env: env,
  argv: argv,
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
  sql: sql,
  spawn: spawn,
  build: buildFn,
};
