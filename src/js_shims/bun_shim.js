// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:bun - Bun 运行时兼容层 (L3)
// Bun v1.1.x API alignment
// Maps to koss:io and koss:crypto standard libraries

var io = require('koss:io');
var kossCrypto = require('koss:crypto');
var kossSystem = require('koss:system');

var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;

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
      var Readable;
      try { Readable = require('koss:stream').Readable; } catch (e) { Readable = null; }
      if (!Readable) {
        throw new Error('ReadableStream is not supported in KossJS (Boa 0.21.x)');
      }
      var path = this.path;
      var self = this;
      var stream = new Readable({
        read: function() {
          try {
            var data = io.read(path);
            if (data === null || data === undefined) { this.push(null); return; }
            this.push(data);
            this.push(null);
          } catch (err) {
            this.emit('error', err);
          }
        },
      });
      return stream;
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
  if (!cmd) return null;
  var pathVar = (kossSystem.env && kossSystem.env().PATH) || '';
  var sep = (kossSystem.platform && kossSystem.platform() === 'win32') ? ';' : ':';
  var pathSep = (kossSystem.platform && kossSystem.platform() === 'win32') ? '\\' : '/';
  var isWin = sep === ';';
  var candidates = pathVar.split(sep);
  for (var i = 0; i < candidates.length; i++) {
    if (!candidates[i]) continue;
    var base = candidates[i];
    if (base.charAt(base.length - 1) !== pathSep && base.charAt(base.length - 1) !== '/') {
      base += pathSep;
    }
    var full = base + cmd;
    if (!isWin && io.exists(full)) return full;
    if (isWin) {
      if (io.exists(full)) return full;
      var exts = ['.exe', '.cmd', '.bat', '.com'];
      for (var j = 0; j < exts.length; j++) {
        if (io.exists(full + exts[j])) return full + exts[j];
      }
    }
  }
  // 若 cmd 本身含路径分隔符，直接检查
  if (cmd.indexOf('/') !== -1 || (isWin && cmd.indexOf('\\') !== -1)) {
    if (io.exists(cmd)) return cmd;
  }
  return null;
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

// ── 压缩 ──
var kossZlib = null;
try { kossZlib = require('koss:zlib'); } catch (e) { kossZlib = null; }

function gzipSync(data, options) { return kossZlib ? kossZlib.gzipSync(data, options) : _unsupported('gzipSync'); }
function gunzipSync(data, options) { return kossZlib ? kossZlib.gunzipSync(data, options) : _unsupported('gunzipSync'); }
function deflateSync(data, options) { return kossZlib ? kossZlib.deflateSync(data, options) : _unsupported('deflateSync'); }
function inflateSync(data, options) { return kossZlib ? kossZlib.inflateSync(data, options) : _unsupported('inflateSync'); }

// ── 高精度时间 ──
function nanoseconds() {
  if (typeof globalThis.__koss_performance_now === 'function') {
    return Math.round(globalThis.__koss_performance_now() * 1e6);
  }
  return Date.now() * 1e6;
}

// ── 深比较 ──
function deepEquals(a, b) {
  if (a === b) return true;
  if (a === null || b === null || a === undefined || b === undefined) return a === b;
  if (typeof a !== 'object' || typeof b !== 'object') return false;
  if (a instanceof Uint8Array && b instanceof Uint8Array) {
    if (a.length !== b.length) return false;
    for (var i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
  }
  var ka = Object.keys(a), kb = Object.keys(b);
  if (ka.length !== kb.length) return false;
  for (var j = 0; j < ka.length; j++) {
    var k = ka[j];
    if (!Object.prototype.hasOwnProperty.call(b, k)) return false;
    if (!deepEquals(a[k], b[k])) return false;
  }
  return true;
}

function deepMatch(a, b) {
  if (b === a) return true;
  if (b && typeof b === 'object' && a && typeof a === 'object') {
    for (var k in b) {
      if (!deepMatch(a[k], b[k])) return false;
    }
    return true;
  }
  return b === a;
}

// ── 文本工具 ──
function escapeHTML(input) {
  return String(input)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function stringWidth(str) {
  var s = String(str);
  var width = 0;
  for (var i = 0; i < s.length; i++) {
    var c = s.codePointAt(i);
    // 东亚宽字符（CJK）计 2 宽度
    if (c > 0x1100 && (c <= 0x115f || c === 0x2329 || c === 0x232a ||
        (c >= 0x2e80 && c <= 0xa4cf && c !== 0x303f) ||
        (c >= 0xac00 && c <= 0xd7a3) ||
        (c >= 0xf900 && c <= 0xfaff) ||
        (c >= 0xfe10 && c <= 0xfe19) ||
        (c >= 0xfe30 && c <= 0xfe6f) ||
        (c >= 0xff00 && c <= 0xff60) ||
        (c >= 0xffe0 && c <= 0xffe6))) {
      width += 2;
    } else {
      width += 1;
    }
  }
  return width;
}

// ── URL/路径工具 ──
function fileURLToPath(url) {
  var u = typeof url === 'string' ? new URL(url) : url;
  if (u.protocol !== 'file:') throw new TypeError('Must be a file URL');
  var path = decodeURIComponent(u.pathname);
  if (process && process.platform === 'win32') {
    path = path.replace(/^\//, '').replace(/\//g, '\\');
  }
  return path;
}

function pathToFileURL(path) {
  var p = String(path).replace(/\\/g, '/');
  if (p.charAt(0) !== '/') p = '/' + p;
  return new URL('file://' + encodeURI(p).replace(/%2F/gi, '/'));
}

// ── 内存工具 ──
function concatArrayBuffers(buffers) {
  var total = 0;
  var i;
  for (i = 0; i < buffers.length; i++) total += buffers[i].byteLength;
  var out = new Uint8Array(total);
  var offset = 0;
  for (i = 0; i < buffers.length; i++) {
    out.set(new Uint8Array(buffers[i]), offset);
    offset += buffers[i].byteLength;
  }
  return out.buffer;
}

function allocUnsafe(size) {
  return new ArrayBuffer(Number(size) || 0);
}

// ── CryptoHasher ──
function CryptoHasher(algorithm) {
  this._algorithm = String(algorithm).toLowerCase().replace('-', '');
  this._chunks = [];
}
CryptoHasher.prototype.update = function(data) {
  var u8 = data instanceof Uint8Array ? data : new Uint8Array(data);
  this._chunks.push(u8);
  return this;
};
CryptoHasher.prototype.digest = function(encoding) {
  var total = 0;
  for (var i = 0; i < this._chunks.length; i++) total += this._chunks[i].length;
  var input = new Uint8Array(total);
  var offset = 0;
  for (var j = 0; j < this._chunks.length; j++) { input.set(this._chunks[j], offset); offset += this._chunks[j].length; }
  var hex = kossCrypto.hashHex(this._algorithm, input);
  if (encoding === 'hex' || !encoding) return hex;
  if (encoding === 'base64') {
    var bytes = new Uint8Array(hex.length / 2);
    for (var k = 0; k < bytes.length; k++) bytes[k] = parseInt(hex.substr(k * 2, 2), 16);
    var b64 = '';
    for (var m = 0; m < bytes.length; m += 3) {
      var a = bytes[m], b = bytes[m+1], c = bytes[m+2];
      b64 += _b64chars[(a >> 2) & 63] + _b64chars[((a << 4) | (b >> 4)) & 63] +
        (m+1 < bytes.length ? _b64chars[((b << 2) | (c >> 6)) & 63] : '=') +
        (m+2 < bytes.length ? _b64chars[c & 63] : '=');
    }
    return b64;
  }
  return hex;
};
var _b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';

// ── Glob ──
function Glob(pattern) {
  this._pattern = String(pattern);
  this._regex = new RegExp('^' + _globToRegex(this._pattern) + '$');
}
Glob.prototype.match = function(path) {
  var p = String(path).replace(/\\/g, '/');
  return this._regex.test(p);
};
Glob.prototype.scan = function(options) {
  var root = (options && options.cwd) || '.';
  var self = this;
  var results = [];
  function walk(dir) {
    var entries;
    try { entries = io.list(dir); } catch (e) { return; }
    for (var i = 0; i < entries.length; i++) {
      var name = typeof entries[i] === 'string' ? entries[i] : entries[i][0];
      var full = dir === '.' ? name : dir + '/' + name;
      try {
        var st = io.stat(full);
        if (st && (st.isDir || st.isDirectory)) {
          walk(full);
        } else {
          if (self.match(full)) results.push(full);
        }
      } catch (e) {
        if (self.match(full)) results.push(full);
      }
    }
  }
  walk(root);
  return results;
};
function _globToRegex(pattern) {
  var p = pattern;
  var out = '';
  var i = 0;
  while (i < p.length) {
    var ch = p[i];
    if (ch === '*') {
      if (p[i+1] === '*') {
        // ** 匹配任意层级
        out += '.*';
        i += 2;
      } else {
        out += '[^/]*';
        i++;
      }
    } else if (ch === '?') {
      out += '[^/]';
      i++;
    } else if (ch === '[') {
      var j = i + 1;
      var cls = '[';
      if (p[j] === '!' || p[j] === '^') { cls += '^'; j++; }
      while (j < p.length && p[j] !== ']') { cls += p[j]; j++; }
      cls += ']';
      out += cls;
      i = j + 1;
    } else if ('\\.+?^$|(){}'.indexOf(ch) !== -1) {
      out += '\\' + ch;
      i++;
    } else {
      out += ch;
      i++;
    }
  }
  return out;
}

// ── Cookie / CookieMap ──
function Cookie(name, value, options) {
  this.name = String(name);
  this.value = String(value);
  var opts = options || {};
  this.path = opts.path || '/';
  this.domain = opts.domain || '';
  this.expires = opts.expires || undefined;
  this.httpOnly = !!(opts.httpOnly);
  this.secure = !!(opts.secure);
  this.sameSite = opts.sameSite || '';
}
Object.defineProperty(Cookie.prototype, 'toString', {
  value: function() {
    var parts = [this.name + '=' + this.value];
    if (this.path) parts.push('Path=' + this.path);
    if (this.domain) parts.push('Domain=' + this.domain);
    if (this.expires) parts.push('Expires=' + this.expires);
    if (this.httpOnly) parts.push('HttpOnly');
    if (this.secure) parts.push('Secure');
    if (this.sameSite) parts.push('SameSite=' + this.sameSite);
    return parts.join('; ');
  },
  writable: true,
  configurable: true,
});

function CookieMap(initial) {
  this._map = {};
  if (initial) {
    if (typeof initial === 'string') this._parseHeader(initial);
    else if (initial instanceof CookieMap) this._map = Object.assign({}, initial._map);
    else if (typeof initial === 'object') this.set(initial);
  }
}
CookieMap.prototype._parseHeader = function(header) {
  var pairs = header.split(';');
  for (var i = 0; i < pairs.length; i++) {
    var pair = pairs[i].trim();
    if (!pair) continue;
    var eq = pair.indexOf('=');
    if (eq === -1) continue;
    this._map[pair.substring(0, eq).trim()] = pair.substring(eq + 1).trim();
  }
};
CookieMap.prototype.get = function(name) {
  return this._map[name] !== undefined ? this._map[name] : undefined;
};
CookieMap.prototype.set = function(name, value) {
  if (typeof name === 'object' && name !== null) {
    var keys = Object.keys(name);
    for (var i = 0; i < keys.length; i++) this._map[keys[i]] = name[keys[i]];
  } else {
    this._map[String(name)] = String(value);
  }
  return this;
};
CookieMap.prototype.delete = function(name) {
  delete this._map[String(name)];
};
CookieMap.prototype.has = function(name) {
  return Object.prototype.hasOwnProperty.call(this._map, String(name));
};
CookieMap.prototype.entries = function() {
  var arr = [];
  for (var k in this._map) arr.push([k, this._map[k]]);
  return arr[Symbol.iterator]();
};
Object.defineProperty(CookieMap.prototype, 'toString', {
  value: function() {
    var parts = [];
    for (var k in this._map) parts.push(k + '=' + this._map[k]);
    return parts.join('; ');
  },
  writable: true,
  configurable: true,
});
CookieMap.prototype[Symbol.iterator] = function() { return this.entries(); };
Object.defineProperty(CookieMap.prototype, 'size', {
  get: function() { return Object.keys(this._map).length; }
});

function _unsupported(name) {
  return new Error('Bun.' + name + ' is not available in KossJS');
}

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
  // 新增
  gzipSync: gzipSync,
  gunzipSync: gunzipSync,
  deflateSync: deflateSync,
  inflateSync: inflateSync,
  nanoseconds: nanoseconds,
  deepEquals: deepEquals,
  deepMatch: deepMatch,
  escapeHTML: escapeHTML,
  stringWidth: stringWidth,
  fileURLToPath: fileURLToPath,
  pathToFileURL: pathToFileURL,
  concatArrayBuffers: concatArrayBuffers,
  allocUnsafe: allocUnsafe,
  CryptoHasher: CryptoHasher,
  Glob: Glob,
  Cookie: Cookie,
  CookieMap: CookieMap,
};
