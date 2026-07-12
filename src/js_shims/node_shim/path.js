// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/path - Node.js path module (L3)
// Pure JS implementation

const CHAR_DOT = 46;
const CHAR_FORWARD_SLASH = 47;
const CHAR_BACKWARD_SLASH = 92;

function isPathSeparator(code) { return code === CHAR_FORWARD_SLASH || code === CHAR_BACKWARD_SLASH; }
function isPosixPathSeparator(code) { return code === CHAR_FORWARD_SLASH; }

function win32Normalize(path) {
  const len = path.length;
  if (len === 0) return '.';
  let rootEnd = 0;
  let device;
  let isAbsolute = false;
  if (len >= 2 && path.charCodeAt(1) === 58) {
    device = path.slice(0, 2);
    rootEnd = 2;
    if (len >= 3 && isPathSeparator(path.charCodeAt(2))) {
      isAbsolute = true;
      rootEnd = 3;
    }
  } else if (isPathSeparator(path.charCodeAt(0))) {
    isAbsolute = true;
    rootEnd = 1;
    if (isPathSeparator(path.charCodeAt(1))) {
      let j = 2;
      while (j < len && !isPathSeparator(path.charCodeAt(j))) j++;
      rootEnd = j + 1;
    }
  }
  const tail = path.slice(rootEnd);
  const parts = tail.split(/[\\/]+/).filter(Boolean);
  const resolved = [];
  for (var part of parts) {
    if (part === '.') continue;
    if (part === '..') {
      if (resolved.length > 0 && resolved[resolved.length - 1] !== '..') {
        resolved.pop();
      } else if (!isAbsolute) {
        resolved.push('..');
      }
    } else {
      resolved.push(part);
    }
  }
  return (device || '') + (isAbsolute ? '\\' : '') + resolved.join('\\') || '.';
}

function normalize(path, isPosix) {
  const sep = isPosix ? '/' : '\\';
  const isSep = isPosix ? isPosixPathSeparator : isPathSeparator;
  const len = path.length;
  if (len === 0) return '.';
  let rootEnd = 0;
  let device;
  let isAbsolute = false;
  if (!isPosix) {
    if (len >= 2 && path.charCodeAt(1) === 58) {
      device = path.slice(0, 2);
      rootEnd = 2;
      if (len >= 3 && isSep(path.charCodeAt(2))) { isAbsolute = true; rootEnd = 3; }
    } else if (isSep(path.charCodeAt(0))) {
      isAbsolute = true;
      rootEnd = 1;
    }
  } else {
    if (isSep(path.charCodeAt(0))) { isAbsolute = true; rootEnd = 1; }
  }
  const tail = path.slice(rootEnd);
  const parts = tail.split(/[\\/]+/).filter(Boolean);
  const resolved = [];
  for (var part of parts) {
    if (part === '.') continue;
    if (part === '..') {
      if (resolved.length > 0 && resolved[resolved.length - 1] !== '..') resolved.pop();
      else if (!isAbsolute) resolved.push('..');
    } else {
      resolved.push(part);
    }
  }
  return (device || '') + (isAbsolute ? sep : '') + resolved.join(sep) || '.';
}

function relative(from, to, isPosix) {
  const sep = isPosix ? '/' : '\\';
  const resolvedFrom = normalize(from, isPosix).split(sep).filter(Boolean);
  const resolvedTo = normalize(to, isPosix).split(sep).filter(Boolean);
  const minLength = Math.min(resolvedFrom.length, resolvedTo.length);
  let samePartsLength = minLength;
  for (var i = 0; i < minLength; i++) {
    if (resolvedFrom[i] !== resolvedTo[i]) { samePartsLength = i; break; }
  }
  const out = [];
  for (var i = samePartsLength; i < resolvedFrom.length; i++) out.push('..');
  for (var i = samePartsLength; i < resolvedTo.length; i++) out.push(resolvedTo[i]);
  return out.join(sep) || '.';
}

const win32 = {
  sep: '\\',
  delimiter: ';',
  resolve(...paths) {
    let resolvedPath = '';
    let resolvedAbsolute = false;
    for (var i = paths.length - 1; i >= 0 && !resolvedAbsolute; i--) {
      const path = String(paths[i]);
      if (path.length === 0) continue;
      resolvedPath = path + '\\' + resolvedPath;
      resolvedAbsolute = path.charCodeAt(0) === CHAR_FORWARD_SLASH || path.charCodeAt(0) === CHAR_BACKWARD_SLASH ||
        (path.length >= 2 && path.charCodeAt(1) === 58);
    }
    if (!resolvedAbsolute) {
      resolvedPath = process?.cwd() + '\\' + resolvedPath;
    }
    return win32Normalize(resolvedPath);
  },
  normalize(path) { return win32Normalize(path); },
  isAbsolute(path) {
    const len = path.length;
    if (len === 0) return false;
    const code = path.charCodeAt(0);
    if (code === CHAR_FORWARD_SLASH || code === CHAR_BACKWARD_SLASH) return true;
    if (len >= 2 && path.charCodeAt(1) === 58) return true;
    return false;
  },
  join(...paths) {
    if (paths.length === 0) return '.';
    let joined = '';
    for (var i = 0; i < paths.length; i++) {
      const segment = String(paths[i]);
      if (segment.length > 0) {
        if (joined.length === 0) {
          joined = segment;
        } else {
          joined += '\\' + segment;
        }
    }
    }
    return win32Normalize(joined);
  },
  relative(from, to) {
    return relative(from, to, false);
  },
  dirname(path) {
    const len = path.length;
    if (len === 0) return '.';
    let end = -1;
    let matchedSlash = true;
    for (var i = len - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (code === CHAR_FORWARD_SLASH || code === CHAR_BACKWARD_SLASH) {
        if (!matchedSlash) {
          end = i;
          break;
        }
      } else {
        matchedSlash = false;
      }
    }
    if (end === -1) {
      const hasDrive = len >= 2 && path.charCodeAt(1) === 58;
      return hasDrive ? path.slice(0, 2) : '.';
    }
    if (end === 0) return path.slice(0, 1);
    const hasRoot = end === 1 && path.charCodeAt(0) === CHAR_BACKWARD_SLASH ||
                    end === 2 && path.charCodeAt(1) === 58;
    return path.slice(0, hasRoot ? end + 1 : end);
  },
  basename(path, ext) {
    let start = 0;
    let end = -1;
    let matchedSlash = true;
    for (var i = path.length - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (isPathSeparator(code)) {
        if (!matchedSlash) {
          start = i + 1;
          break;
        }
      } else if (end === -1) {
        matchedSlash = false;
        end = i + 1;
      }
    }
    if (end === -1) return '';
    const name = path.slice(start, end);
    if (ext && name.endsWith(ext)) return name.slice(0, -ext.length);
    return name;
  },
  extname(path) {
    let startDot = -1;
    let startPart = 0;
    let end = -1;
    let matchedSlash = true;
    let preDotState = 0;
    for (var i = path.length - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (isPathSeparator(code)) {
        if (!matchedSlash) {
          startPart = i + 1;
          break;
        }
        continue;
      }
      if (end === -1) {
        matchedSlash = false;
        end = i + 1;
      }
      if (code === CHAR_DOT) {
        if (startDot === -1) startDot = i;
        else if (preDotState !== 1) preDotState = 1;
      } else if (startDot !== -1) {
        preDotState = -1;
      }
    }
    if (startDot === -1 || end === -1 || preDotState === 0 || preDotState === 1 && startDot === end - 1 && startDot === startPart + 1) return '';
    return path.slice(startDot, end);
  },
  format(pathObj) {
    const dir = pathObj.dir || pathObj.root;
    const base = pathObj.base || '';
    if (!base) return dir || '.';
    if (!dir) return base;
    return dir + (dir.endsWith('\\') ? '' : '\\') + base;
  },
  parse(path) {
    const allParts = win32._splitPath(path);
    return {
      root: allParts[0],
      dir: allParts[1],
      base: allParts[2],
      ext: allParts[3],
      name: allParts[4],
    };
  },
  _splitPath(path) {
    const len = path.length;
    if (len === 0) return ['', '', '', '', ''];
    let root = '';
    const deviceMatch = len >= 2 && path.charCodeAt(1) === 58 ? path.slice(0, 2) : '';
    const isAbs = isPathSeparator(path.charCodeAt(0));
    if (deviceMatch && len >= 3 && isPathSeparator(path.charCodeAt(2))) {
      root = deviceMatch + '\\';
    } else if (isAbs) {
      root = '\\';
    } else if (deviceMatch) {
      root = deviceMatch;
    }
    const base = win32.basename(path);
    const dir = len > 1 ? path.slice(root.length, len - base.length - (root.length < len && isPathSeparator(path.charCodeAt(len - base.length - 1)) ? 1 : 0)) : '';
    const ext = win32.extname(base);
    const name = base.slice(0, base.length - ext.length);
    return [root || '', dir || '', base || '', ext || '', name || ''];
  },
};

const posix = {
  sep: '/',
  delimiter: ':',
  resolve(...paths) {
    let resolvedPath = '';
    let resolvedAbsolute = false;
    for (var i = paths.length - 1; i >= 0 && !resolvedAbsolute; i--) {
      const path = String(paths[i]);
      if (path.length === 0) continue;
      resolvedPath = path + '/' + resolvedPath;
      resolvedAbsolute = path.charCodeAt(0) === CHAR_FORWARD_SLASH;
    }
    if (!resolvedAbsolute) {
      resolvedPath = (process?.cwd?.() || '.') + '/' + resolvedPath;
    }
    return posix.normalize(resolvedPath);
  },
  normalize(path) { return normalize(path, true); },
  isAbsolute(path) { return path.length > 0 && path.charCodeAt(0) === CHAR_FORWARD_SLASH; },
  join(...paths) {
    if (paths.length === 0) return '.';
    let joined = '';
    for (var i = 0; i < paths.length; i++) {
      const segment = String(paths[i]);
      if (segment.length > 0) {
        if (joined.length === 0) joined = segment;
        else joined += '/' + segment;
      }
    }
    return posix.normalize(joined);
  },
  relative(from, to) { return relative(from, to, true); },
  dirname(path) {
    const len = path.length;
    if (len === 0) return '.';
    let end = -1;
    let matchedSlash = true;
    for (var i = len - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (code === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { end = i; break; }
      } else { matchedSlash = false; }
    }
    if (end === -1) return '.';
    if (end === 0) return '/';
    return path.slice(0, end);
  },
  basename(path, ext) {
    let start = 0;
    let end = -1;
    let matchedSlash = true;
    for (var i = path.length - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (code === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { start = i + 1; break; }
      } else if (end === -1) { matchedSlash = false; end = i + 1; }
    }
    if (end === -1) return '';
    const name = path.slice(start, end);
    if (ext && name.endsWith(ext)) return name.slice(0, -ext.length);
    return name;
  },
  extname(path) {
    let startDot = -1;
    let startPart = 0;
    let end = -1;
    let matchedSlash = true;
    let preDotState = 0;
    for (var i = path.length - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (code === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { startPart = i + 1; break; }
        continue;
      }
      if (end === -1) { matchedSlash = false; end = i + 1; }
      if (code === CHAR_DOT) {
        if (startDot === -1) startDot = i;
        else if (preDotState !== 1) preDotState = 1;
      } else if (startDot !== -1) { preDotState = -1; }
    }
    if (startDot === -1 || end === -1 || preDotState === 0 || preDotState === 1 && startDot === end - 1 && startDot === startPart + 1) return '';
    return path.slice(startDot, end);
  },
  format(pathObj) {
    const dir = pathObj.dir || pathObj.root;
    const base = pathObj.base || '';
    if (!base) return dir || '.';
    if (!dir) return base;
    return dir + (dir.endsWith('/') ? '' : '/') + base;
  },
  parse(path) {
    const root = path.charCodeAt(0) === CHAR_FORWARD_SLASH ? '/' : '';
    const base = posix.basename(path);
    const dir = root ? path.slice(0, path.length - base.length) || '/' : path.slice(0, path.length - base.length) || '.';
    const ext = posix.extname(base);
    const name = base.slice(0, base.length - ext.length);
    return { root, dir, base, ext, name };
  },
};

const sep = '/';
const delimiter = ':';

function resolve(...paths) { return posix.resolve(...paths); }
function normalizePath(path) { return posix.normalize(path); }
function isAbsolute(path) { return posix.isAbsolute(path); }
function join(...paths) { return posix.join(...paths); }
function relativePath(from, to) { return posix.relative(from, to); }
function dirname(path) { return posix.dirname(path); }
function basename(path, ext) { return posix.basename(path, ext); }
function extname(path) { return posix.extname(path); }
function format(pathObj) { return posix.format(pathObj); }
function parse(path) { return posix.parse(path); }

module.exports = { sep, delimiter, resolve, normalize: normalizePath, isAbsolute, join, relative: relativePath, dirname, basename, extname, format, parse, win32, posix };
