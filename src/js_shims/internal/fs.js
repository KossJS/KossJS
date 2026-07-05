// koss:internal/fs - Internal filesystem layer (L2)
// Not directly accessible to user code. Used by L3 compatibility layers.

var __koss_fs_read = globalThis.__koss_fs_read;
var __koss_fs_write = globalThis.__koss_fs_write;
var __koss_fs_stat = globalThis.__koss_fs_stat;
var __koss_fs_mkdir = globalThis.__koss_fs_mkdir;
var __koss_fs_readdir = globalThis.__koss_fs_readdir;
var __koss_fs_unlink = globalThis.__koss_fs_unlink;
var __koss_fs_rename = globalThis.__koss_fs_rename;
var __koss_fs_copy = globalThis.__koss_fs_copy;
var __koss_fs_chmod = globalThis.__koss_fs_chmod;
var __koss_fs_realpath = globalThis.__koss_fs_realpath;
var __koss_fs_exists = globalThis.__koss_fs_exists;

function parseResult(result) {
  if (result && typeof result === 'object' && result.code !== undefined) return result;
  if (typeof result === 'string') {
    try { return JSON.parse(result); } catch(e) { return null; }
  }
  return null;
}

function throwIfError(result, message) {
  var r = parseResult(result);
  if (r && r.code !== undefined && r.code !== 0) {
    throw new Error(message ? message + ': ' + (r.value || r.code) : 'FS error: ' + (r.value || r.code));
  }
}

var _b64c = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function _b64i(c) { var x = _b64c.indexOf(c); return x >= 0 ? x : 0; }
function _b64dec(s) {
  var out = [];
  for (var i = 0; i < s.length; i += 4) {
    var a = _b64i(s[i]), b = _b64i(s[i+1]), c = _b64i(s[i+2]), d = _b64i(s[i+3]);
    var t = (a << 18) | (b << 12) | (c << 6) | d;
    out.push((t >> 16) & 0xFF);
    if (s[i+2] !== '=') out.push((t >> 8) & 0xFF);
    if (s[i+3] !== '=') out.push(t & 0xFF);
  }
  return out;
}

function _b64decToString(b64) {
  var bytes = _b64dec(b64);
  var chars = [];
  for (var i = 0; i < bytes.length; i++) chars.push(String.fromCharCode(bytes[i]));
  return chars.join('');
}

function readFileSyncUtf8(path) {
  if (typeof __koss_fs_read === 'function') {
    var raw = __koss_fs_read(path);
    var result = parseResult(raw);
    if (result && result.code === 0 && result.value) {
      return _b64decToString(result.value);
    }
    throw new Error('Failed to read file: ' + path);
  }
  throw new Error('FS read capability not available');
}

function readFileSync(path) {
  if (typeof __koss_fs_read === 'function') {
    var raw = __koss_fs_read(path);
    var result = parseResult(raw);
    if (result && result.code === 0 && result.value) {
      return new Uint8Array(_b64dec(result.value));
    }
    throw new Error('Failed to read file: ' + path);
  }
  throw new Error('FS read capability not available');
}

function writeFileSync(path, data) {
  if (typeof __koss_fs_write === 'function') {
    var dataStr;
    if (typeof data === 'string') {
      dataStr = data;
    } else if (data instanceof Uint8Array) {
      var chars = [];
      for (var i = 0; i < data.length; i++) chars.push(String.fromCharCode(data[i]));
      dataStr = chars.join('');
    } else if (data && data.toString) {
      dataStr = data.toString();
    } else {
      dataStr = String(data);
    }
    var raw = __koss_fs_write(path, dataStr);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to write file: ' + path);
    return result && result.value;
  }
  throw new Error('FS write capability not available');
}

function existsSync(path) {
  if (typeof __koss_fs_exists === 'function') {
    return Boolean(__koss_fs_exists(path));
  }
  try {
    statSync(path);
    return true;
  } catch (e) {
    return false;
  }
}

function statSync(path) {
  if (typeof __koss_fs_stat === 'function') {
    var raw = __koss_fs_stat(path);
    var result = parseResult(raw);
    if (result && result.value) {
      if (typeof result.value === 'string') {
        return JSON.parse(result.value);
      }
      return result.value;
    }
    throw new Error('Failed to stat: ' + path);
  }
  throw new Error('FS stat capability not available');
}

function mkdirSync(path, options) {
  if (typeof __koss_fs_mkdir === 'function') {
    var recursive = options && options.recursive ? true : false;
    var raw = __koss_fs_mkdir(path, recursive ? 1 : 0);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to create directory: ' + path);
    return result && result.value;
  }
  throw new Error('FS mkdir capability not available');
}

function rmdirSync(path) {
  if (typeof __koss_fs_unlink === 'function') {
    var raw = __koss_fs_unlink(path);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to remove: ' + path);
    return result && result.value;
  }
  throw new Error('FS remove capability not available');
}

function unlinkSync(path) {
  return rmdirSync(path);
}

function readdirSync(path) {
  if (typeof __koss_fs_readdir === 'function') {
    var raw = __koss_fs_readdir(path);
    var result = parseResult(raw);
    if (result && result.value) {
      return typeof result.value === 'string' ? JSON.parse(result.value) : result.value;
    }
    throw new Error('Failed to read directory: ' + path);
  }
  throw new Error('FS readdir capability not available');
}

function renameSync(oldPath, newPath) {
  if (typeof __koss_fs_rename === 'function') {
    var raw = __koss_fs_rename(oldPath, newPath);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to rename: ' + oldPath + ' -> ' + newPath);
    return result && result.value;
  }
  throw new Error('FS rename capability not available');
}

function realpathSync(path) {
  if (typeof __koss_fs_realpath === 'function') {
    var result = __koss_fs_realpath(path);
    if (typeof result === 'string') return result;
    if (result && result.code === 0 && result.value) {
      return result.value;
    }
    return path;
  }
  return path;
}

function copyFileSync(src, dest) {
  if (typeof __koss_fs_copy === 'function') {
    var raw = __koss_fs_copy(src, dest);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to copy: ' + src + ' -> ' + dest);
    return result && result.value;
  }
  throw new Error('FS copy capability not available');
}

function chmodSync(path, mode) {
  if (typeof __koss_fs_chmod === 'function') {
    var raw = __koss_fs_chmod(path, mode);
    var result = parseResult(raw);
    throwIfError(result, 'Failed to chmod: ' + path);
    return result && result.value;
  }
  throw new Error('FS chmod capability not available');
}

module.exports = {
  readFileSync: readFileSync,
  readFileSyncUtf8: readFileSyncUtf8,
  writeFileSync: writeFileSync,
  existsSync: existsSync,
  statSync: statSync,
  mkdirSync: mkdirSync,
  rmdirSync: rmdirSync,
  unlinkSync: unlinkSync,
  readdirSync: readdirSync,
  renameSync: renameSync,
  realpathSync: realpathSync,
  copyFileSync: copyFileSync,
  chmodSync: chmodSync,
};
