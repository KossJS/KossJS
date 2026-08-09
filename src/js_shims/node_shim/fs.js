// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/fs - Node.js fs module (L3)
// Maps to koss:io standard library + fd-level native functions

var io = require('koss:io');

var { Buffer } = globalThis;

var nextTick = (typeof process !== 'undefined' && process.nextTick) ? process.nextTick : function(fn) { setTimeout(fn, 0); };

function getOptions(options, defaultEncoding) {
  if (options === null || options === undefined) return { encoding: defaultEncoding };
  if (typeof options === 'string') return { encoding: options };
  if (typeof options === 'object') return { encoding: options.encoding || defaultEncoding, ...options };
  return { encoding: defaultEncoding };
}

function _wrapBuffer(data) {
  if (typeof Buffer !== 'undefined' && Buffer && Buffer.from) return Buffer.from(data);
  return data;
}

function _b64dec(s) {
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var lookup = {};
  for (var i = 0; i < chars.length; i++) lookup[chars.charCodeAt(i)] = i;
  var out = [];
  for (var j = 0; j < s.length; j += 4) {
    var a = lookup[s.charCodeAt(j)] || 0;
    var b = lookup[s.charCodeAt(j + 1)] || 0;
    var c = lookup[s.charCodeAt(j + 2)] || 0;
    var d = lookup[s.charCodeAt(j + 3)] || 0;
    var t = (a << 18) | (b << 12) | (c << 6) | d;
    out.push((t >> 16) & 0xFF);
    if (s[j + 2] !== '=') out.push((t >> 8) & 0xFF);
    if (s[j + 3] !== '=') out.push(t & 0xFF);
  }
  return new Uint8Array(out);
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

// ─── Stats 类（与 Node 兼容：方法是函数） ───
function Stats(size, mode, mtimeMs, ctimeMs, isFile, isDir, isSymlink) {
  this.size = size;
  this.mode = mode;
  this.mtimeMs = mtimeMs;
  this.ctimeMs = ctimeMs;
  this.atimeMs = mtimeMs;
  this.birthtimeMs = ctimeMs;
  this._isFile = isFile;
  this._isDir = isDir;
  this._isSymlink = isSymlink;
  this.uid = 0;
  this.gid = 0;
  this.ino = 0;
  this.dev = 0;
  this.nlink = 1;
  this.blksize = 4096;
  this.blocks = 0;
}

Stats.prototype.isFile = function() { return this._isFile; };
Stats.prototype.isDirectory = function() { return this._isDir; };
Stats.prototype.isSymbolicLink = function() { return this._isSymlink; };
Stats.prototype.isBlockDevice = function() { return false; };
Stats.prototype.isCharacterDevice = function() { return false; };
Stats.prototype.isFIFO = function() { return false; };
Stats.prototype.isSocket = function() { return false; };

function _toStats(raw) {
  if (raw instanceof Stats) return raw;
  if (!raw) return new Stats(0, 0, 0, 0, true, false, false);
  var isFile = raw.isFile === undefined ? true : Boolean(raw.isFile);
  var isDir = raw.isDirectory !== undefined ? Boolean(raw.isDirectory) : Boolean(raw.isDir);
  var size = typeof raw.size === 'number' ? raw.size : 0;
  var mtime = typeof raw.mtime === 'number' ? raw.mtime : (raw.mtimeMs || 0);
  var ctime = typeof raw.ctime === 'number' ? raw.ctime : (raw.ctimeMs || 0);
  return new Stats(size, 0o100644, mtime, ctime, isFile, isDir, Boolean(raw.isSymlink));
}

// Dirent 类
function Dirent(name, isDir, isFile) {
  this.name = name;
  this._isDir = isDir;
  this._isFile = isFile;
}
Dirent.prototype.isDirectory = function() { return this._isDir; };
Dirent.prototype.isFile = function() { return this._isFile; };
Dirent.prototype.isSymbolicLink = function() { return false; };
Dirent.prototype.isBlockDevice = function() { return false; };
Dirent.prototype.isCharacterDevice = function() { return false; };
Dirent.prototype.isFIFO = function() { return false; };
Dirent.prototype.isSocket = function() { return false; };

// === Synchronous API ===

function readFileSync(path, options) {
  var opts = getOptions(options, null);
  if (opts.encoding) {
    return io.readText(String(path));
  }
  return _wrapBuffer(io.read(String(path)));
}

function writeFileSync(path, data, options) {
  var opts = getOptions(options, 'utf8');
  if (typeof data === 'string') {
    io.writeText(String(path), data);
  } else {
    io.write(String(path), data);
  }
}

function appendFileSync(path, data, options) {
  var u8 = null;
  if (data instanceof Uint8Array) u8 = data;
  else if (data && data._data instanceof Uint8Array) u8 = data._data;
  if (u8 && typeof globalThis.__koss_fs_append === 'function') {
    globalThis.__koss_fs_append(String(path), _b64c(u8), true);
  } else {
    globalThis.__koss_fs_append(String(path), String(data), false);
  }
}

function existsSync(path) {
  return io.exists(String(path));
}

function statSync(path, options) {
  return _toStats(io.stat(String(path)));
}

function lstatSync(path, options) {
  return _toStats(io.lstat(String(path)));
}

function mkdirSync(path, options) {
  var opts = typeof options === 'object' ? options : { recursive: Boolean(options) };
  io.mkdir(String(path), opts);
}

function rmdirSync(path, options) {
  io.rm(String(path), { recursive: !!(options && options.recursive) });
}

function unlinkSync(path) {
  io.rm(String(path));
}

function readdirSync(path, options) {
  var opts = getOptions(options, null);
  var entries = io.list(String(path));
  if (!opts.withFileTypes) {
    return entries.map(function(e) { return typeof e === 'string' ? e : e[0]; });
  }
  var result = [];
  for (var i = 0; i < entries.length; i++) {
    var name = typeof entries[i] === 'string' ? entries[i] : entries[i][0];
    var raw = null;
    try { raw = io.stat(String(path).replace(/[\\/]+$/, '') + '/' + name); } catch (e) {}
    var isDir = raw ? Boolean(raw.isDirectory !== undefined ? raw.isDirectory : raw.isDir) : false;
    result.push(new Dirent(name, isDir, !isDir));
  }
  return result;
}

function renameSync(oldPath, newPath) {
  io.mv(String(oldPath), String(newPath));
}

function realpathSync(path, options) {
  return io.realpath(String(path));
}

function copyFileSync(src, dest, flags) {
  io.cp(String(src), String(dest));
}

function chmodSync(path, mode) {
  io.chmod(String(path), Number(mode) || 0);
}

function accessSync(path, mode) {
  if (!io.exists(path)) throw Object.assign(new Error('ENOENT'), { code: 'ENOENT', errno: -2, syscall: 'access', path: String(path) });
}

function mkdtempSync(prefix, options) {
  return io.mkdtemp(prefix || 'tmp-');
}

function truncateSync(path, len) {
  io.truncate(String(path), len || 0);
}

// ─── fd 级 API ───

function openSync(path, flags, mode) {
  if (typeof globalThis.__koss_fd_open !== 'function') {
    throw new Error('fs.openSync requires FS_READ/FS_WRITE capability');
  }
  var flagNum = typeof flags === 'number' ? flags : _flagStringToInt(flags);
  return globalThis.__koss_fd_open(String(path), flagNum);
}

function _flagStringToInt(flag) {
  switch (String(flag)) {
    case 'r': return 0;
    case 'r+': return 2;
    case 'w': return 577; // O_WRONLY(1) | O_CREAT(64) | O_TRUNC(512)
    case 'w+': return 578; // O_RDWR(2) | O_CREAT(64) | O_TRUNC(512)
    case 'a': return 1025; // O_WRONLY(1) | O_CREAT(64) | O_APPEND(1024)
    case 'a+': return 1090; // O_RDWR(2) | O_CREAT(64) | O_APPEND(1024)
    case 'ax': return 1153; // O_WRONLY | O_CREAT | O_APPEND | O_EXCL
    case 'ax+': return 1218;
    case 'wx': return 705; // O_WRONLY | O_CREAT | O_EXCL
    case 'wx+': return 706;
    default: return 0;
  }
}

function closeSync(fd) {
  if (typeof globalThis.__koss_fd_close !== 'function') throw new Error('fs.closeSync requires FS capability');
  globalThis.__koss_fd_close(Number(fd));
}

function readSync(fd, buffer, offset, length, position) {
  if (typeof globalThis.__koss_fd_read !== 'function') throw new Error('fs.readSync requires FS_READ capability');
  var len = length !== undefined ? length : (buffer.length - (offset || 0));
  var result = _parseFdResult(globalThis.__koss_fd_read(Number(fd), len));
  if (result && result.code === 2) return 0;
  if (result && result.code === 0 && result.value) {
    var bytes = _b64dec(result.value);
    var target = buffer._data || buffer;
    var off = offset || 0;
    for (var i = 0; i < bytes.length; i++) target[off + i] = bytes[i];
    return bytes.length;
  }
  throw new Error('fs.readSync failed: ' + (result && result.value || ''));
}

function writeSync(fd, buffer, offset, length, position) {
  if (typeof globalThis.__koss_fd_write !== 'function') throw new Error('fs.writeSync requires FS_WRITE capability');
  var data;
  if (typeof buffer === 'string') {
    data = buffer;
    offset = offset !== undefined ? offset : 0;
    length = length !== undefined ? length : undefined;
  } else {
    var bytes = buffer._data || buffer;
    var off = offset || 0;
    var len = length !== undefined ? length : bytes.length - off;
    data = _b64c(bytes.subarray(off, off + len));
    var result = _parseFdResult(globalThis.__koss_fd_write(Number(fd), data, true));
    if (result && result.code === 0) return result.value || len;
    throw new Error('fs.writeSync failed: ' + (result && result.value || ''));
  }
  var result2 = _parseFdResult(globalThis.__koss_fd_write(Number(fd), data, false));
  if (result2 && result2.code === 0) return result2.value || data.length;
  throw new Error('fs.writeSync failed: ' + (result2 && result2.value || ''));
}

function _parseFdResult(raw) {
  if (raw && typeof raw === 'object' && raw.code !== undefined) return raw;
  if (typeof raw === 'string') {
    try { return JSON.parse(raw); } catch (e) { return null; }
  }
  return null;
}

function fsyncSync(fd) {
  if (typeof globalThis.__koss_fd_sync !== 'function') throw new Error('fs.fsyncSync requires FS capability');
  globalThis.__koss_fd_sync(Number(fd));
}

function ftruncateSync(fd, len) {
  if (typeof globalThis.__koss_fd_truncate !== 'function') throw new Error('fs.ftruncateSync requires FS capability');
  globalThis.__koss_fd_truncate(Number(fd), len || 0);
}

function fstatSync(fd) {
  // 通过 fd 读取 stat：使用 __koss_fd_fstat（若存在），否则返回占位
  if (typeof globalThis.__koss_fd_fstat === 'function') {
    var result = _parseFdResult(globalThis.__koss_fd_fstat(Number(fd)));
    if (result && result.code === 0) {
      return _toStats(result);
    }
  }
  throw new Error('fstatSync: invalid fd ' + fd);
}

// ─── 流式 API ───

function createReadStream(path, options) {
  var opts = options || {};
  var fd;
  try { fd = openSync(path, 'r'); } catch (e) {
    var errStream = new (require('koss:stream').Readable)();
    process.nextTick(function() { errStream.emit('error', e); });
    return errStream;
  }
  var pos = 0;
  var self = this;
  var Readable = require('koss:stream').Readable;
  var stream = new Readable({
    read: function() {
      try {
        var n = readSync(fd, Buffer.alloc(65536), 0, 65536);
        if (n === 0) {
          closeSync(fd);
          this.push(null);
        } else {
          var buf = Buffer.alloc(n);
          var chunk = Buffer.alloc(65536);
          var n2 = readSync(fd, chunk, 0, 65536);
          if (n2 === 0) { closeSync(fd); this.push(null); return; }
          buf = Buffer.from(chunk.subarray(0, n2));
          this.push(buf);
        }
      } catch (err) {
        try { closeSync(fd); } catch (e) {}
        this.emit('error', err);
      }
    }
  });
  return stream;
}

function createWriteStream(path, options) {
  var opts = options || {};
  var fd;
  var flags = opts.flags || 'w';
  try { fd = openSync(path, flags, opts.mode); } catch (e) {
    var errStream = new (require('koss:stream').Writable)();
    process.nextTick(function() { errStream.emit('error', e); });
    return errStream;
  }
  var Writable = require('koss:stream').Writable;
  var stream = new Writable({
    write: function(chunk, enc, cb) {
      try {
        writeSync(fd, chunk);
        cb && cb();
      } catch (err) {
        cb && cb(err);
      }
    },
    final: function(cb) {
      try { fsyncSync(fd); closeSync(fd); } catch (e) {}
      cb && cb();
    },
  });
  stream.on('error', function() { try { closeSync(fd); } catch (e) {} });
  return stream;
}

// ─── watch ───

function watch(filename, options, listener) {
  if (typeof options === 'function') { listener = options; options = {}; }
  if (typeof listener !== 'function') listener = function() {};
  return io.watch(String(filename), function(event, path) { listener(event, path); });
}

function watchFile(filename, options, listener) {
  if (typeof options === 'function') { listener = options; options = {}; }
  if (typeof listener !== 'function') listener = function() {};
  return io.watch(String(filename), function(event, path) { listener(null, _toStats(io.stat(String(filename)))); });
}

function unwatchFile(filename, listener) {
  // io.watch 返回的对象支持 close()，此处由用户持有并调用 close
  return null;
}

// === Callback API ===

function callAsync(fn, args) {
  nextTick(function() {
    try { fn.apply(null, args); } catch (err) { /* swallow */ }
  });
}

function readFile(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, readFileSync(path, options)); }
    catch (err) { cb(err); }
  });
}

function writeFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { writeFileSync(path, data, options); cb(null); }
    catch (err) { cb(err); }
  });
}

function appendFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { appendFileSync(path, data, options); cb(null); }
    catch (err) { cb(err); }
  });
}

function exists(path, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() { cb(existsSync(path)); });
}

function stat(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, statSync(path, options)); }
    catch (err) { cb(err); }
  });
}

function lstat(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, lstatSync(path, options)); }
    catch (err) { cb(err); }
  });
}

function mkdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { mkdirSync(path, options); cb(null); }
    catch (err) { cb(err); }
  });
}

function rmdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { rmdirSync(path, options); cb(null); }
    catch (err) { cb(err); }
  });
}

function unlink(path, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { unlinkSync(path); cb(null); }
    catch (err) { cb(err); }
  });
}

function readdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, readdirSync(path, options)); }
    catch (err) { cb(err); }
  });
}

function rename(oldPath, newPath, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { renameSync(oldPath, newPath); cb(null); }
    catch (err) { cb(err); }
  });
}

function realpath(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, realpathSync(path, options)); }
    catch (err) { cb(err); }
  });
}

function copyFile(src, dest, flags, callback) {
  if (typeof flags === 'function') { callback = flags; flags = 0; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { copyFileSync(src, dest, flags); cb(null); }
    catch (err) { cb(err); }
  });
}

function access(path, mode, callback) {
  if (typeof mode === 'function') { callback = mode; mode = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { accessSync(path, mode); cb(null); }
    catch (err) { cb(err); }
  });
}

function chmod(path, mode, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { chmodSync(path, mode); cb(null); }
    catch (err) { cb(err); }
  });
}

function open(path, flags, mode, callback) {
  if (typeof mode === 'function') { callback = mode; mode = undefined; }
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { cb(null, openSync(path, flags, mode)); }
    catch (err) { cb(err); }
  });
}

function close(fd, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  callAsync(function() {
    try { closeSync(fd); cb(null); }
    catch (err) { cb(err); }
  });
}

// === Promises API ===

var promises = {
  readFile: function(path, options) { return new Promise(function(resolve, reject) { readFile(path, options, function(err, data) { err ? reject(err) : resolve(data); }); }); },
  writeFile: function(path, data, options) { return new Promise(function(resolve, reject) { writeFile(path, data, options, function(err) { err ? reject(err) : resolve(); }); }); },
  appendFile: function(path, data, options) { return new Promise(function(resolve, reject) { appendFile(path, data, options, function(err) { err ? reject(err) : resolve(); }); }); },
  stat: function(path, options) { return new Promise(function(resolve, reject) { stat(path, options, function(err, s) { err ? reject(err) : resolve(s); }); }); },
  lstat: function(path, options) { return new Promise(function(resolve, reject) { lstat(path, options, function(err, s) { err ? reject(err) : resolve(s); }); }); },
  mkdir: function(path, options) { return new Promise(function(resolve, reject) { mkdir(path, options, function(err) { err ? reject(err) : resolve(); }); }); },
  rmdir: function(path, options) { return new Promise(function(resolve, reject) { rmdir(path, options, function(err) { err ? reject(err) : resolve(); }); }); },
  unlink: function(path) { return new Promise(function(resolve, reject) { unlink(path, function(err) { err ? reject(err) : resolve(); }); }); },
  readdir: function(path, options) { return new Promise(function(resolve, reject) { readdir(path, options, function(err, files) { err ? reject(err) : resolve(files); }); }); },
  rename: function(oldPath, newPath) { return new Promise(function(resolve, reject) { rename(oldPath, newPath, function(err) { err ? reject(err) : resolve(); }); }); },
  realpath: function(path, options) { return new Promise(function(resolve, reject) { realpath(path, options, function(err, p) { err ? reject(err) : resolve(p); }); }); },
  copyFile: function(src, dest, flags) { return new Promise(function(resolve, reject) { copyFile(src, dest, flags, function(err) { err ? reject(err) : resolve(); }); }); },
  access: function(path, mode) { return new Promise(function(resolve, reject) { access(path, mode, function(err) { err ? reject(err) : resolve(); }); }); },
  chmod: function(path, mode) { return new Promise(function(resolve, reject) { chmod(path, mode, function(err) { err ? reject(err) : resolve(); }); }); },
  mkdtemp: function(prefix, options) { return new Promise(function(resolve, reject) { try { resolve(mkdtempSync(prefix, options)); } catch (err) { reject(err); } }); },
  open: function(path, flags, mode) { return new Promise(function(resolve, reject) { open(path, flags, mode, function(err, fd) { err ? reject(err) : resolve(fd); }); }); },
  close: function(fd) { return new Promise(function(resolve, reject) { close(fd, function(err) { err ? reject(err) : resolve(); }); }); },
};

var constants = {
  F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1,
  O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2, O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512, O_APPEND: 1024,
  S_IFMT: 61440, S_IFREG: 32768, S_IFDIR: 16384, S_IFLNK: 40960,
  S_IRWXU: 448, S_IRUSR: 256, S_IWUSR: 128, S_IXUSR: 64,
  S_IRWXG: 56, S_IRGRP: 32, S_IWGRP: 16, S_IXGRP: 8, S_IRWXO: 7, S_IROTH: 4, S_IWOTH: 2, S_IXOTH: 1,
  COPYFILE_EXCL: 1, COPYFILE_FICLONE: 2, COPYFILE_FICLONE_FORCE: 4,
};

module.exports = { readFileSync, writeFileSync, appendFileSync, existsSync, statSync, lstatSync, mkdirSync, rmdirSync, unlinkSync, readdirSync, renameSync, realpathSync, copyFileSync, chmodSync, accessSync, mkdtempSync, truncateSync, fstatSync, openSync, closeSync, readSync, writeSync, fsyncSync, ftruncateSync, readFile, writeFile, appendFile, exists, stat, lstat, mkdir, rmdir, unlink, readdir, rename, realpath, copyFile, access, chmod, open, close, promises, constants, watch, watchFile, unwatchFile, createReadStream, createWriteStream, Stats, Dirent };
