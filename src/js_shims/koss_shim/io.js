// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:io — Koss 原生统一 I/O 模块
// 文件操作 + 网络操作，全部同步 API

var internalFs = require('koss:internal/fs');
var internalNet = require('koss:internal/net');
var streamModule = require('koss:internal/stream');

var Buffer = globalThis.Buffer;

// ═══════════════════════════════════════════
// 文件操作
// ═══════════════════════════════════════════

function read(path) {
  var data = internalFs.readFileSync(path);
  if (data instanceof Uint8Array) return data;
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
    var chars = [];
    for (var i = 0; i < data.length; i++) chars.push(String.fromCharCode(data[i]));
    return internalFs.writeFileSync(path, chars.join(''));
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

function rm(path, options) {
  if (options && options.recursive) {
    try {
      var entries = internalFs.readdirSync(path);
      if (Array.isArray(entries)) {
        for (var i = 0; i < entries.length; i++) {
          rm(path + '/' + entries[i], { recursive: true });
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
  var server = internalNet.tcpListen(hostname, Number(port));
  return {
    port: Number(port),
    hostname: String(hostname),
    accept: function() { return server.accept(); },
    close: function() { server.close(); },
  };
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
  stat: stat, list: list, mkdir: mkdir, rm: rm, cp: cp, mv: mv, exists: exists, watch: watch,
  connect: connect, serve: serve, fetch: fetch, dns: dns,
  ReadStream: ReadStream, WriteStream: WriteStream,
  createReadStream: createReadStream, createWriteStream: createWriteStream, pipeline: pipeline,
};
