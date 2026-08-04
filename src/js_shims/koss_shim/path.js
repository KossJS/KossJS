// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:path — Koss 原生路径标准库
// POSIX 与 Win32 路径操作（纯 JS 实现）

// ── 工具函数 ──

var isWindows = typeof process !== 'undefined' && process.platform === 'win32';

function toStr(path) {
  return path == null ? '' : String(path);
}

function splitPathPosix(p) {
  var s = toStr(p).replace(/\/+$/, '');
  if (s === '') return ['.', ''];
  var isAbsolute = s.charCodeAt(0) === 0x2f; // '/'
  var prefix = isAbsolute ? '/' : '';
  var parts = s.split('/');
  var filtered = [];
  for (var i = 0; i < parts.length; i++) {
    if (parts[i] !== '') filtered.push(parts[i]);
  }
  if (filtered.length === 0) return [prefix, ''];
  return [prefix, filtered.join('/')];
}

function splitPathWin32(p) {
  var s = toStr(p).replace(/[\\\/]+$/, '');
  if (s === '') return ['.', ''];
  var isAbsolute = s.charCodeAt(0) === 0x5c || s.charCodeAt(0) === 0x2f;
  if (/^[A-Za-z]:[\\\/]/.test(s)) {
    var drive = s.substring(0, 2);
    var rest = s.substring(3).replace(/[\\\/]+/g, '\\');
    var parts = rest ? rest.split('\\') : [];
    var filtered = [];
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] !== '') filtered.push(parts[i]);
    }
    return [drive + '\\', filtered.join('\\')];
  }
  if (isAbsolute) {
    var rest2 = s.replace(/^[\\\/]+/, '').replace(/[\\\/]+/g, '\\');
    var parts2 = rest2 ? rest2.split('\\') : [];
    var filtered2 = [];
    for (var j = 0; j < parts2.length; j++) {
      if (parts2[j] !== '') filtered2.push(parts2[j]);
    }
    return ['\\', filtered2.join('\\')];
  }
  var parts3 = s.replace(/[\\\/]+/g, '\\').split('\\');
  var filtered3 = [];
  for (var k = 0; k < parts3.length; k++) {
    if (parts3[k] !== '') filtered3.push(parts3[k]);
  }
  return ['', filtered3.join('\\')];
}

function isPosixRoot(p) {
  return p === '/';
}

function isWin32Root(p) {
  return p === '\\' || /^[A-Za-z]:\\$/.test(p) || p === '\\\\';
}

// ── POSIX ──

var posixResolve = (function () {
  var sep = '/';

  function resolve() {
    var resolvedPath = '';
    var resolvedAbsolute = false;
    for (var i = arguments.length - 1; i >= 0 && !resolvedAbsolute; i--) {
      var path = toStr(arguments[i]);
      if (path === '') continue;
      var parts = path.split('/');
      var lastEmptyIndex = -1;
      for (var j = 0; j < parts.length; j++) {
        if (parts[j] === '') { lastEmptyIndex = j; break; }
      }
      resolvedPath = parts.join('/') + (resolvedPath !== '' ? '/' + resolvedPath : '');
      if (resolvedPath.charAt(0) === '/') resolvedAbsolute = true;
    }
    if (!resolvedAbsolute) resolvedPath = '/' + resolvedPath;
    var result = '';
    var segs = resolvedPath.split('/');
    var stack = [];
    for (var k = 0; k < segs.length; k++) {
      var seg = segs[k];
      if (seg === '' && k !== 0) continue;
      if (seg === '.') continue;
      if (seg === '..') { if (stack.length > 0) stack.pop(); }
      else stack.push(seg);
    }
    result = '/' + stack.join('/');
    return result;
  }

  function normalize(path) {
    var p = toStr(path);
    if (p === '') return '.';
    var isAbs = p.charAt(0) === '/';
    var trailingSlash = p.charAt(p.length - 1) === '/';
    var segs = p.split('/');
    var filtered = [];
    for (var i = 0; i < segs.length; i++) {
      if (segs[i] !== '' && segs[i] !== '.') filtered.push(segs[i]);
    }
    var result = [];
    for (var j = 0; j < filtered.length; j++) {
      if (filtered[j] === '..') {
        if (result.length > 0 && result[result.length - 1] !== '..') result.pop();
        else if (!isAbs) result.push('..');
      } else {
        result.push(filtered[j]);
      }
    }
    var joined = result.join('/');
    if (isAbs) joined = '/' + joined;
    if (joined === '' && !isAbs) joined = '.';
    if (trailingSlash && joined !== '' && joined !== '/' && !joined.endsWith('/')) joined += '/';
    if (!isAbs && !trailingSlash && joined !== '.' && result.length > 0) {
      // no trailing slash unless specified
    }
    return joined;
  }

  function isAbsolute(path) {
    return toStr(path).charAt(0) === '/';
  }

  function join() {
    var joined = '';
    for (var i = 0; i < arguments.length; i++) {
      var arg = toStr(arguments[i]);
      if (arg === '') continue;
      if (joined === '') {
        joined = arg;
      } else {
        joined = joined.replace(/\/+$/, '') + '/' + arg.replace(/^\/+/, '');
      }
    }
    return joined === '' ? '.' : normalize(joined);
  }

  function relative(from, to) {
    var fromPath = resolve(from);
    var toPath = resolve(to);
    if (fromPath === toPath) return '';
    var fromSegs = fromPath.split('/').slice(1);
    var toSegs = toPath.split('/').slice(1);
    var i = 0;
    while (i < fromSegs.length && i < toSegs.length && fromSegs[i] === toSegs[i]) i++;
    var result = [];
    for (var j = i; j < fromSegs.length; j++) result.push('..');
    for (var k = i; k < toSegs.length; k++) result.push(toSegs[k]);
    return result.length === 0 ? '.' : result.join('/');
  }

  function dirname(path) {
    var p = toStr(path);
    if (p === '' || p === '.') return '.';
    if (p === '/') return '/';
    var parts = p.split('/');
    var last = parts.pop();
    if (parts.length === 0) return last === '' ? '/' : '.';
    var result = parts.join('/');
    if (result === '') result = '/';
    return result;
  }

  function basename(path, ext) {
    var p = toStr(path);
    if (p === '') return '';
    p = p.replace(/\/+$/, '');
    var lastSlash = p.lastIndexOf('/');
    if (lastSlash !== -1) p = p.substring(lastSlash + 1);
    if (ext !== undefined && ext !== '') {
      if (p.endsWith(ext)) p = p.substring(0, p.length - ext.length);
    }
    return p === '' ? '/' : p;
  }

  function extname(path) {
    var base = basename(path);
    var lastDot = base.lastIndexOf('.');
    if (lastDot <= 0) return '';
    return base.substring(lastDot);
  }

  function format(obj) {
    if (typeof obj === 'object' && obj !== null) {
      var root = obj.root || '';
      var dir = obj.dir || '';
      var base = obj.base || '';
      var ext = obj.ext || '';
      var name = obj.name || '';
      if (base !== '') return dir + '/' + base;
      if (name !== '' || ext !== '') return (dir || '.') + '/' + name + ext;
      return dir || root || '.';
    }
    return toStr(obj);
  }

  function parse(path) {
    var root = '';
    var dir = '';
    var base = '';
    var ext = '';
    var name = '';
    var p = toStr(path);
    if (p === '') return { root: '', dir: '', base: '', ext: '', name: '' };
    if (p === '/') return { root: '/', dir: '/', base: '/', ext: '', name: '' };
    var isAbs = p.charAt(0) === '/';
    if (isAbs) root = '/';
    var parts = p.split('/');
    base = parts.pop() || '';
    if (parts.length > 0) {
      dir = (isAbs ? '/' : '') + parts.join('/');
    } else {
      dir = isAbs ? '/' : '.';
    }
    var dotIdx = base.lastIndexOf('.');
    if (dotIdx > 0) {
      ext = base.substring(dotIdx);
      name = base.substring(0, dotIdx);
    } else {
      name = base;
    }
    return { root: root, dir: dir, base: base, ext: ext, name: name };
  }

  var posix = {
    resolve: resolve,
    normalize: normalize,
    isAbsolute: isAbsolute,
    join: join,
    relative: relative,
    dirname: dirname,
    basename: basename,
    extname: extname,
    format: format,
    parse: parse,
    sep: '/',
    delimiter: ':',
  };

  return posix;
})();

// ── Win32 ──

var win32Resolve = (function () {
  var sep = '\\';

  function isAbsoluteWin(path) {
    var p = toStr(path);
    if (p === '') return false;
    if (p.charAt(0) === '/' || p.charAt(0) === '\\') return true;
    if (/^[A-Za-z]:/.test(p)) return true;
    return false;
  }

  function resolve() {
    var resolvedPath = '';
    var resolvedAbsolute = false;
    for (var i = arguments.length - 1; i >= 0 && !resolvedAbsolute; i--) {
      var path = toStr(arguments[i]);
      if (path === '') continue;
      var parts = path.replace(/[\\\/]+/g, '\\').split('\\');
      var filtered = [];
      for (var j = 0; j < parts.length; j++) {
        if (parts[j] !== '') filtered.push(parts[j]);
      }
      if (filtered.length > 0) {
        if (/^[A-Za-z]:$/.test(filtered[0])) {
          resolvedPath = filtered[0] + '\\' + filtered.slice(1).join('\\') + (resolvedPath !== '' ? '\\' + resolvedPath : '');
          resolvedAbsolute = true;
        } else if (filtered[0] === '\\') {
          resolvedPath = '\\' + filtered.slice(1).join('\\') + (resolvedPath !== '' ? '\\' + resolvedPath : '');
          resolvedAbsolute = true;
        } else {
          resolvedPath = filtered.join('\\') + (resolvedPath !== '' ? '\\' + resolvedPath : '');
        }
      }
    }
    if (!resolvedAbsolute) resolvedPath = process.cwd() + '\\' + resolvedPath;
    var stack = [];
    var segs = resolvedPath.replace(/[\\\/]+/g, '\\').split('\\');
    for (var k = 0; k < segs.length; k++) {
      var seg = segs[k];
      if (seg === '' && k !== 0 && k !== segs.length - 1) continue;
      if (seg === '.') continue;
      if (seg === '..') { if (stack.length > 0) stack.pop(); }
      else stack.push(seg);
    }
    if (stack.length > 0 && /^[A-Za-z]:$/.test(stack[0])) {
      return stack[0] + '\\' + stack.slice(1).join('\\');
    }
    return '\\' + stack.join('\\');
  }

  function normalize(path) {
    var p = toStr(path);
    if (p === '') return '.';
    var isAbs = isAbsoluteWin(p);
    var trailingSlash = p.charAt(p.length - 1) === '\\' || p.charAt(p.length - 1) === '/';
    var parts = p.replace(/[\\\/]+/g, '\\').split('\\');
    var filtered = [];
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] !== '' && parts[i] !== '.') filtered.push(parts[i]);
    }
    var result = [];
    var driveRoot = '';
    if (filtered.length > 0 && /^[A-Za-z]:$/.test(filtered[0])) {
      driveRoot = filtered[0] + '\\';
      filtered = filtered.slice(1);
    }
    for (var j = 0; j < filtered.length; j++) {
      if (filtered[j] === '..') {
        if (result.length > 0 && result[result.length - 1] !== '..') result.pop();
        else if (!isAbs && driveRoot === '') result.push('..');
      } else {
        result.push(filtered[j]);
      }
    }
    var joined = driveRoot + result.join('\\');
    if (isAbs && driveRoot === '') joined = '\\' + result.join('\\');
    if (joined === '' && !isAbs) joined = '.';
    if (trailingSlash && joined !== '' && !joined.endsWith('\\')) joined += '\\';
    return joined;
  }

  function isAbsolute(path) {
    return isAbsoluteWin(path);
  }

  function join() {
    var joined = '';
    for (var i = 0; i < arguments.length; i++) {
      var arg = toStr(arguments[i]);
      if (arg === '') continue;
      if (joined === '') {
        joined = arg;
      } else {
        joined = joined.replace(/[\\\/]+$/, '') + '\\' + arg.replace(/^[\\\/]+/, '');
      }
    }
    return joined === '' ? '.' : normalize(joined);
  }

  function relative(from, to) {
    var fromPath = normalize(resolve(from));
    var toPath = normalize(resolve(to));
    if (fromPath === toPath) return '';
    var fromSegs = fromPath.replace(/^[A-Za-z]:\\?/, '').split('\\');
    var toSegs = toPath.replace(/^[A-Za-z]:\\?/, '').split('\\');
    var i = 0;
    while (i < fromSegs.length && i < toSegs.length && fromSegs[i] === toSegs[i]) i++;
    var result = [];
    for (var j = i; j < fromSegs.length; j++) result.push('..');
    for (var k = i; k < toSegs.length; k++) result.push(toSegs[k]);
    return result.length === 0 ? '.' : result.join('\\');
  }

  function dirname(path) {
    var p = toStr(path).replace(/[\\\/]+$/, '');
    if (p === '') return '.';
    if (p === '\\') return '\\';
    if (p === '/') return '/';
    if (/^[A-Za-z]:$/.test(p)) return p + '\\';
    var parts = p.replace(/[\\\/]+/g, '\\').split('\\');
    var last = parts.pop();
    if (parts.length === 0) return last === '' ? '\\' : '.';
    var result = parts.join('\\');
    if (result === '') result = '\\';
    if (/^[A-Za-z]:$/.test(result)) result += '\\';
    return result;
  }

  function basename(path, ext) {
    var p = toStr(path).replace(/[\\\/]+$/, '');
    if (p === '') return '';
    var lastSlash = p.lastIndexOf('\\');
    var lastSlash2 = p.lastIndexOf('/');
    var lastSep = Math.max(lastSlash, lastSlash2);
    if (lastSep !== -1) p = p.substring(lastSep + 1);
    if (ext !== undefined && ext !== '') {
      if (p.endsWith(ext)) p = p.substring(0, p.length - ext.length);
    }
    return p === '' ? '\\' : p;
  }

  function extname(path) {
    var base = basename(path);
    var lastDot = base.lastIndexOf('.');
    if (lastDot <= 0) return '';
    return base.substring(lastDot);
  }

  function format(obj) {
    if (typeof obj === 'object' && obj !== null) {
      var root = obj.root || '';
      var dir = obj.dir || '';
      var base = obj.base || '';
      var ext = obj.ext || '';
      var name = obj.name || '';
      if (base !== '') return dir + '\\' + base;
      if (name !== '' || ext !== '') return (dir || '.') + '\\' + name + ext;
      return dir || root || '.';
    }
    return toStr(obj);
  }

  function parse(path) {
    var root = '';
    var dir = '';
    var base = '';
    var ext = '';
    var name = '';
    var p = toStr(path);
    if (p === '') return { root: '', dir: '', base: '', ext: '', name: '' };
    var isAbs = isAbsoluteWin(p);
    var parts = p.replace(/[\\\/]+/g, '\\').split('\\');
    base = parts.pop() || '';
    if (isAbs) {
      if (/^[A-Za-z]:\\$/.test(p)) {
        root = p.substring(0, 2) + '\\';
        dir = root;
      } else if (p.charAt(0) === '\\' || p.charAt(0) === '/') {
        root = '\\';
        dir = parts.length > 0 ? '\\' + parts.join('\\') : '\\';
      }
    } else {
      dir = parts.length > 0 ? parts.join('\\') : '.';
    }
    var dotIdx = base.lastIndexOf('.');
    if (dotIdx > 0) {
      ext = base.substring(dotIdx);
      name = base.substring(0, dotIdx);
    } else {
      name = base;
    }
    return { root: root, dir: dir, base: base, ext: ext, name: name };
  }

  var win32 = {
    resolve: resolve,
    normalize: normalize,
    isAbsolute: isAbsolute,
    join: join,
    relative: relative,
    dirname: dirname,
    basename: basename,
    extname: extname,
    format: format,
    parse: parse,
    sep: '\\',
    delimiter: ';',
  };

  return win32;
})();

// ── 默认导出 ──

var posix_ = posixResolve;
var win32_ = win32Resolve;
var defaultImpl = isWindows ? win32_ : posix_;

var path = {
  resolve: defaultImpl.resolve,
  normalize: defaultImpl.normalize,
  isAbsolute: defaultImpl.isAbsolute,
  join: defaultImpl.join,
  relative: defaultImpl.relative,
  dirname: defaultImpl.dirname,
  basename: defaultImpl.basename,
  extname: defaultImpl.extname,
  format: defaultImpl.format,
  parse: defaultImpl.parse,
  sep: defaultImpl.sep,
  delimiter: defaultImpl.delimiter,
  posix: posix_,
  win32: win32_,
};

module.exports = path;
