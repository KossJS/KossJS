// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:io — Koss 原生统一 I/O 模块
// 文件操作 + 网络操作，全部同步 API

var internalFs = require('koss:internal/fs');
var internalNet = require('koss:internal/net');
var streamModule = require('koss:internal/stream');
var dataModule = require('koss:data');

var decode = dataModule.decode;
var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;

// ═══════════════════════════════════════════
// 文件操作
// ═══════════════════════════════════════════

function read(path) {
  var data = internalFs.readFileSync(path);
  if (data instanceof Uint8Array) return data;
  if (typeof data === 'string') {
    var bytes = new Uint8Array(data.length);
    for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
    return bytes;
  }
  return new Uint8Array(0);
}

function readText(path) {
  return internalFs.readFileSyncUtf8(path);
}

function write(path, data) {
  if (typeof data === 'string') {
    return internalFs.writeFileSync(path, data);
  }
  if (data instanceof Uint8Array) {
    return internalFs.writeFileSync(path, data);
  }
  return internalFs.writeFileSync(path, String(data));
}

function writeText(path, text) {
  return internalFs.writeFileSync(path, String(text));
}

function stat(path) {
  var raw = internalFs.statSync(path);
  return {
    size: (raw && raw.size) || 0,
    mtime: (raw && raw.mtime) || 0,
    ctime: (raw && raw.ctime) || 0,
    isFile: raw ? raw.isFile : true,
    isDir: raw ? raw.isDirectory : false,
    isSymlink: raw ? raw.isSymlink : false,
  };
}

function list(path) {
  var entries = internalFs.readdirSync(path);
  return Array.isArray(entries) ? entries : [];
}

function mkdir(path, options) {
  return internalFs.mkdirSync(path, options);
}

function _joinPath(base, name) {
  if (!base) return name;
  if (base.charAt(base.length - 1) === '/' || base.charAt(base.length - 1) === '\\') return base + name;
  return base + '/' + name;
}

function rm(path, options) {
  if (options && options.recursive) {
    try {
      var entries = internalFs.readdirSync(path);
      if (Array.isArray(entries)) {
        for (var i = 0; i < entries.length; i++) {
          rm(_joinPath(path, entries[i]), { recursive: true });
        }
      }
    } catch(e) {}
    try { internalFs.unlinkSync(path); } catch(e) {}
    try { internalFs.rmdirSync(path); } catch(e) {}
    return;
  }
  try {
    return internalFs.unlinkSync(path);
  } catch (e) {
    return internalFs.rmdirSync(path);
  }
}

function cp(src, dst) {
  return internalFs.copyFileSync(src, dst);
}

function mv(src, dst) {
  return internalFs.renameSync(src, dst);
}

function exists(path) {
  return internalFs.existsSync(path);
}

function append(path, data) {
  if (typeof globalThis.__koss_fs_append === 'function') {
    if (typeof data === 'string') {
      globalThis.__koss_fs_append(String(path), data, false);
      return;
    }
    var u8 = data instanceof Uint8Array ? data : (data && data._data instanceof Uint8Array ? data._data : new Uint8Array(0));
    var b64 = dataModule.toBase64 ? dataModule.toBase64(u8) : _b64c(u8);
    globalThis.__koss_fs_append(String(path), b64, true);
    return;
  }
  var existing = read(path);
  var newData;
  if (typeof data === 'string') {
    var existingStr = existing.length > 0 ? decode(existing) : '';
    writeText(path, existingStr + data);
    return;
  }
  if (data instanceof Uint8Array) {
    newData = data;
  } else {
    newData = new Uint8Array(0);
  }
  var combined = new Uint8Array(existing.length + newData.length);
  combined.set(existing, 0);
  combined.set(newData, existing.length);
  write(path, combined);
}

function _b64c(bytes) {
  var out = '';
  var i = 0;
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  for (; i + 2 < bytes.length; i += 3) {
    var n = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2];
    out += chars[(n >> 18) & 63] + chars[(n >> 12) & 63] + chars[(n >> 6) & 63] + chars[n & 63];
  }
  var rem = bytes.length - i;
  if (rem === 1) {
    var m = bytes[i] << 16;
    out += chars[(m >> 18) & 63] + chars[(m >> 12) & 63] + '==';
  } else if (rem === 2) {
    var k = (bytes[i] << 16) | (bytes[i + 1] << 8);
    out += chars[(k >> 18) & 63] + chars[(k >> 12) & 63] + chars[(k >> 6) & 63] + '=';
  }
  return out;
}

function chmod(path, mode) {
  return internalFs.chmodSync(String(path), Number(mode) || 0);
}

function truncate(path, len) {
  if (len === 0) {
    writeText(path, '');
  } else if (len && len > 0) {
    var data = read(path);
    write(path, data.slice(0, len));
  }
}

function mkdtemp(prefix) {
  var dir = internalFs.realpathSync('.') + '/' + (prefix || 'tmp-') + Date.now();
  internalFs.mkdirSync(dir);
  return dir;
}

function lstat(path) {
  return internalFs.statSync(String(path));
}

function realpath(path) {
  return internalFs.realpathSync(String(path));
}

function symlink(target, path) {
  throw new Error('symlink not implemented');
}

function readlink(path) {
  throw new Error('readlink not implemented');
}

// ═══════════════════════════════════════════
// 文件监控（简化实现）
// ═══════════════════════════════════════════

function watch(path, callback) {
  var running = true;
  var lastStat = null;
  try { lastStat = internalFs.statSync(path); } catch (e) { /* ignore */ }

  var interval = setInterval(function() {
    if (!running) return;
    try {
      var current = internalFs.statSync(path);
      if (lastStat && current && current.mtime !== lastStat.mtime) {
        if (callback) callback('change', path);
      }
      lastStat = current;
    } catch (e) {
      if (lastStat !== null) {
        if (callback) callback('remove', path);
        lastStat = null;
      }
    }
  }, 1000);

  return {
    close: function() {
      running = false;
      clearInterval(interval);
    },
  };
}

// ═══════════════════════════════════════════
// 网络操作
// ═══════════════════════════════════════════

function connect(host, port) {
  return internalNet.tcpConnect(host, Number(port));
}

function serve(options, handler) {
  var opts = options || {};
  var hostname = opts.hostname || '0.0.0.0';
  var port = opts.port || 3000;
  // 如果提供了 handler，接线到 koss:http 的完整 HTTP 服务器
  if (typeof handler === 'function') {
    var httpMod = require('koss:http');
    var server = httpMod.createServer(function(req, res) {
      try {
        var result = handler(req);
        if (result && typeof result.then === 'function') {
          result.then(function(val) { _sendHandlerResponse(res, val); }, function(err) {
            res.writeHead(500); res.end(String(err && err.message || err));
          });
        } else {
          _sendHandlerResponse(res, result);
        }
      } catch (err) {
        res.writeHead(500); res.end(String(err && err.message || err));
      }
    });
    server.listen(port, hostname);
    return {
      port: Number(port),
      hostname: String(hostname),
      accept: function() { return null; },
      close: function() { server.close(); },
    };
  }
  var server = internalNet.tcpListen(hostname, Number(port));
  return {
    port: Number(port),
    hostname: String(hostname),
    accept: function() { return server.accept(); },
    close: function() { server.close(); },
  };
}

function _sendHandlerResponse(res, val) {
  if (val instanceof Response) {
    res.writeHead(val.status || 200, val.headers);
    var body = val._body !== undefined ? String(val._body) : '';
    res.end(body);
  } else if (val !== undefined && val !== null && typeof val === 'object') {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify(val));
  } else {
    res.writeHead(200, { 'content-type': 'text/plain' });
    res.end(val === undefined || val === null ? '' : String(val));
  }
}

function fetch(url, options) {
  return internalNet.httpFetch(url, options || {});
}

function dns(hostname) {
  return internalNet.dnsLookup(String(hostname));
}

// ═══════════════════════════════════════════
// 流（重新导出）
// ═══════════════════════════════════════════

var ReadStream = streamModule.ReadStream;
var WriteStream = streamModule.WriteStream;
var createReadStream = streamModule.createReadStream;
var createWriteStream = streamModule.createWriteStream;
var pipeline = streamModule.pipeline;

module.exports = {
  read: read, readText: readText, write: write, writeText: writeText,
  stat: stat, lstat: lstat, realpath: realpath, symlink: symlink, readlink: readlink,
  list: list, mkdir: mkdir, rm: rm, cp: cp, mv: mv,
  exists: exists, append: append, chmod: chmod, truncate: truncate, mkdtemp: mkdtemp,
  watch: watch,
  connect: connect, serve: serve, fetch: fetch, dns: dns,
  ReadStream: ReadStream, WriteStream: WriteStream,
  createReadStream: createReadStream, createWriteStream: createWriteStream, pipeline: pipeline,
};
