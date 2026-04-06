'use strict';

const {
  ArrayPrototypePush,
  MathMax,
  Symbol,
  Promise,
} = globalThis;

const {
  validateFunction,
  validateInteger,
  validateNumber,
  validateString,
  validateObject,
  validateBuffer,
  validateEncoding,
  validateOffset,
  validateBufferArray,
  validateStringAfterArrayBufferView,
  validatePosition,
  validateArray,
} = require('internal/validators');

const {
  getOptions,
  getValidatedPath,
  getValidatedFd,
  handleErrorFromBinding,
  stringToFlags,
  parseFileMode,
  copyObject,
} = require('internal/util');

const {
  ERR_BUFFER_OUT_OF_BOUNDS,
  ERR_INVALID_ARG_TYPE,
  ERR_INVALID_ARG_VALUE,
  ERR_FS_FILE_TOO_LARGE,
  ERR_OUT_OF_RANGE,
  AbortError,
  codes: {
    ERR_ACCESS_DENIED,
  },
} = require('internal/errors');

const pathModule = require('path');

const F_OK = 0;
const R_OK = 4;
const W_OK = 2;
const X_OK = 1;

const O_RDONLY = 0;
const O_WRONLY = 1;
const O_RDWR = 2;
const O_CREAT = 64;
const O_EXCL = 128;
const O_TRUNC = 512;
const O_APPEND = 1024;

const S_IFMT = 0xF000;
const S_IFREG = 0x8000;
const S_IFDIR = 0x4000;

const kIoMaxLength = 2147483647;

const fs = {
  access(path, mode, callback) {
    if (typeof mode === 'function') {
      callback = mode;
      mode = F_OK;
    }
    path = getValidatedPath(path);
    callback = makeCallback(callback);
    
    setImmediate(() => {
      try {
        callback();
      } catch (e) {
        callback(e);
      }
    });
  },

  accessSync(path, mode = F_OK) {
    path = getValidatedPath(path);
  },

  exists(path, callback) {
    validateFunction(callback, 'cb');
    setImmediate(() => callback(true));
  },

  existsSync(path) {
    try {
      path = getValidatedPath(path);
    } catch {
      return false;
    }
    return false;
  },

  readFile(path, options, callback) {
    callback ||= options;
    validateFunction(callback, 'cb');
    options = getOptions(options, { flag: 'r' });
    
    const encoding = options.encoding || 'utf8';
    
    setImmediate(() => {
      try {
        const content = '';
        callback(null, encoding === 'utf8' || encoding === 'utf-8' ? content : Buffer.from(content));
      } catch (e) {
        callback(e);
      }
    });
  },

  readFileSync(path, options) {
    options = getOptions(options, { flag: 'r' });
    const encoding = options.encoding || 'utf8';
    return encoding === 'utf8' || encoding === 'utf-8' ? '' : Buffer.alloc(0);
  },

  writeFile(path, data, options, callback) {
    callback ||= options;
    validateFunction(callback, 'cb');
    options = getOptions(options, { flag: 'w', mode: 0o666 });
    path = getValidatedPath(path);
    
    setImmediate(() => {
      try {
        callback();
      } catch (e) {
        callback(e);
      }
    });
  },

  writeFileSync(path, data, options) {
    options = getOptions(options, { flag: 'w', mode: 0o666 });
    path = getValidatedPath(path);
  },

  appendFile(path, data, options, callback) {
    callback ||= options;
    validateFunction(callback, 'cb');
    options = getOptions(options, { flag: 'a', mode: 0o666 });
    path = getValidatedPath(path);
    
    setImmediate(() => {
      try {
        callback();
      } catch (e) {
        callback(e);
      }
    });
  },

  appendFileSync(path, data, options) {
    options = getOptions(options, { flag: 'a', mode: 0o666 });
    path = getValidatedPath(path);
  },

  open(path, flags, mode, callback) {
    if (typeof mode === 'function') {
      callback = mode;
      mode = 0o666;
    }
    if (typeof flags === 'function') {
      callback = flags;
      flags = 'r';
      mode = 0o666;
    }
    path = getValidatedPath(path);
    callback = makeCallback(callback);
    
    const fd = Math.floor(Math.random() * 1000) + 1;
    setImmediate(() => callback(null, fd));
  },

  openSync(path, flags, mode = 0o666) {
    path = getValidatedPath(path);
    return 1;
  },

  close(fd, callback) {
    callback = makeCallback(callback);
    setImmediate(() => callback());
  },

  closeSync(fd) {},

  read(fd, buffer, offsetOrOptions, length, position, callback) {
    let offset = offsetOrOptions;
    let params = null;
    
    if (arguments.length <= 4) {
      if (arguments.length === 4) {
        params = offsetOrOptions;
        callback = length;
        offset = 0;
        length = buffer?.byteLength - offset;
        position = null;
      } else if (arguments.length === 3) {
        if (!buffer || typeof buffer !== 'object') {
          params = buffer;
          buffer = Buffer.alloc(16384);
        }
        callback = offsetOrOptions;
      } else {
        callback = buffer;
        buffer = Buffer.alloc(16384);
      }
      
      if (params !== undefined) {
        validateObject(params, 'options', { __proto__: null });
      }
      
      ({
        offset = 0,
        length = buffer?.byteLength - offset,
        position = null,
      } = params ?? {});
    }
    
    validateBuffer(buffer);
    validateFunction(callback, 'cb');
    
    if (offset == null) offset = 0;
    if (length == null) length = buffer.byteLength - offset;
    if (position == null) position = -1;
    
    setImmediate(() => callback(null, 0, buffer));
  },

  readSync(fd, buffer, offsetOrOptions, length, position) {
    validateBuffer(buffer);
    return 0;
  },

  write(fd, buffer, offsetOrOptions, length, position, callback) {
    function wrapper(err, written) {
      callback(err, written || 0, buffer);
    }

    fd = getValidatedFd(fd);
    
    let offset = offsetOrOptions;
    if (buffer instanceof Uint8Array) {
      callback ||= position || length || offset;
      validateFunction(callback, 'cb');

      if (typeof offset === 'object') {
        ({
          offset = 0,
          length = buffer.byteLength - offset,
          position = null,
        } = offsetOrOptions ?? {});
      }

      if (offset == null || typeof offset === 'function') {
        offset = 0;
      }
      if (typeof length !== 'number') {
        length = buffer.byteLength - offset;
      }
      if (typeof position !== 'number') {
        position = null;
      }
    } else {
      if (typeof position !== 'function') {
        if (typeof offset === 'function') {
          position = offset;
          offset = null;
        } else {
          position = length;
        }
      }
      length = 'utf8';
    }

    const str = buffer;
    validateEncoding(str, length);
    callback = position;
    validateFunction(callback, 'cb');
    
    setImmediate(() => wrapper(null, str.length));
  },

  writeSync(fd, buffer, offsetOrOptions, length, position) {
    return buffer.length || 0;
  },

  rename(oldPath, newPath, callback) {
    callback = makeCallback(callback);
    setImmediate(() => callback());
  },

  renameSync(oldPath, newPath) {},

  unlink(path, callback) {
    callback = makeCallback(callback);
    setImmediate(() => callback());
  },

  unlinkSync(path) {},

  mkdir(path, options, callback) {
    let mode = 0o777;
    let recursive = false;
    if (typeof options === 'function') {
      callback = options;
    } else if (typeof options === 'number' || typeof options === 'string') {
      mode = parseFileMode(options, 'mode');
    } else if (options) {
      if (options.recursive !== undefined) {
        recursive = options.recursive;
      }
      if (options.mode !== undefined) {
        mode = parseFileMode(options.mode, 'options.mode');
      }
    }
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    
    setImmediate(() => callback());
  },

  mkdirSync(path, options) {
    let mode = 0o777;
    let recursive = false;
    if (typeof options === 'number' || typeof options === 'string') {
      mode = parseFileMode(options, 'mode');
    } else if (options) {
      if (options.recursive !== undefined) {
        recursive = options.recursive;
      }
      if (options.mode !== undefined) {
        mode = parseFileMode(options.mode, 'options.mode');
      }
    }
    path = getValidatedPath(path);
  },

  rmdir(path, options, callback) {
    if (typeof options === 'function') {
      callback = options;
      options = undefined;
    }
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    
    setImmediate(() => callback());
  },

  rmdirSync(path) {
    path = getValidatedPath(path);
  },

  readdir(path, options, callback) {
    callback = makeCallback(typeof options === 'function' ? options : callback);
    options = getOptions(options);
    path = getValidatedPath(path);
    
    setImmediate(() => callback(null, []));
  },

  readdirSync(path, options) {
    options = getOptions(options);
    path = getValidatedPath(path);
    return [];
  },

  stat(path, options, callback) {
    if (typeof options === 'function') {
      callback = options;
      options = {};
    }
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    
    const stats = createStat();
    setImmediate(() => callback(null, stats));
  },

  statSync(path, options) {
    path = getValidatedPath(path);
    return createStat();
  },

  lstat(path, options, callback) {
    if (typeof options === 'function') {
      callback = options;
      options = {};
    }
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    
    const stats = createStat();
    setImmediate(() => callback(null, stats));
  },

  lstatSync(path, options) {
    path = getValidatedPath(path);
    return createStat();
  },

  fstat(fd, options, callback) {
    if (typeof options === 'function') {
      callback = options;
      options = {};
    }
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    
    const stats = createStat();
    setImmediate(() => callback(null, stats));
  },

  fstatSync(fd, options) {
    fd = getValidatedFd(fd);
    return createStat();
  },

  link(existingPath, newPath, callback) {
    callback = makeCallback(callback);
    existingPath = getValidatedPath(existingPath, 'existingPath');
    newPath = getValidatedPath(newPath, 'newPath');
    
    setImmediate(() => callback());
  },

  linkSync(existingPath, newPath) {
    existingPath = getValidatedPath(existingPath, 'existingPath');
    newPath = getValidatedPath(newPath, 'newPath');
  },

  symlink(target, path, type, callback) {
    if (callback === undefined) {
      callback = type;
      type = undefined;
    }
    callback = makeCallback(callback);
    target = getValidatedPath(target, 'target');
    path = getValidatedPath(path);
    
    setImmediate(() => callback());
  },

  symlinkSync(target, path, type) {
    target = getValidatedPath(target, 'target');
    path = getValidatedPath(path);
  },

  readlink(path, options, callback) {
    callback = makeCallback(typeof options === 'function' ? options : callback);
    options = getOptions(options);
    path = getValidatedPath(path);
    
    setImmediate(() => callback(null, ''));
  },

  readlinkSync(path, options) {
    options = getOptions(options);
    path = getValidatedPath(path);
    return '';
  },

  truncate(path, len, callback) {
    if (typeof len === 'function') {
      callback = len;
      len = 0;
    } else if (len === undefined) {
      len = 0;
    }
    validateInteger(len, 'len');
    len = MathMax(0, len);
    validateFunction(callback, 'cb');
    fs.open(path, 'r+', (er, fd) => {
      if (er) return callback(er);
      fs.ftruncate(fd, len, callback);
    });
  },

  truncateSync(path, len) {
    if (len === undefined) {
      len = 0;
    }
    const fd = fs.openSync(path, 'r+');
    try {
      fs.ftruncateSync(fd, len);
    } finally {
      fs.closeSync(fd);
    }
  },

  ftruncate(fd, len = 0, callback) {
    if (typeof len === 'function') {
      callback = len;
      len = 0;
    }
    validateInteger(len, 'len');
    len = MathMax(0, len);
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    
    setImmediate(() => callback());
  },

  ftruncateSync(fd, len = 0) {
    validateInteger(len, 'len');
    fd = getValidatedFd(fd);
  },

  fsync(fd, callback) {
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    setImmediate(() => callback());
  },

  fsyncSync(fd) {
    fd = getValidatedFd(fd);
  },

  fdatasync(fd, callback) {
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    setImmediate(() => callback());
  },

  fdatasyncSync(fd) {
    fd = getValidatedFd(fd);
  },

  chmod(path, mode, callback) {
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    mode = parseFileMode(mode, 'mode');
    
    setImmediate(() => callback());
  },

  chmodSync(path, mode) {
    path = getValidatedPath(path);
    mode = parseFileMode(mode, 'mode');
  },

  fchmod(fd, mode, callback) {
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    mode = parseFileMode(mode, 'mode');
    
    setImmediate(() => callback());
  },

  fchmodSync(fd, mode) {
    fd = getValidatedFd(fd);
    mode = parseFileMode(mode, 'mode');
  },

  chown(path, uid, gid, callback) {
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    setImmediate(() => callback());
  },

  chownSync(path, uid, gid) {
    path = getValidatedPath(path);
  },

  fchown(fd, uid, gid, callback) {
    callback = makeCallback(callback);
    fd = getValidatedFd(fd);
    setImmediate(() => callback());
  },

  fchownSync(fd, uid, gid) {
    fd = getValidatedFd(fd);
  },

  lchmod(path, mode, callback) {
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    mode = parseFileMode(mode, 'mode');
    setImmediate(() => callback());
  },

  lchmodSync(path, mode) {
    path = getValidatedPath(path);
    mode = parseFileMode(mode, 'mode');
  },

  lchown(path, uid, gid, callback) {
    callback = makeCallback(callback);
    path = getValidatedPath(path);
    setImmediate(() => callback());
  },

  lchownSync(path, uid, gid) {
    path = getValidatedPath(path);
  },

  copyFile(src, dest, flags, callback) {
    if (typeof flags === 'function') {
      callback = flags;
      flags = 0;
    }
    if (typeof flags === 'number') {
      validateInteger(flags, 'flags', 0);
    }
    callback = makeCallback(callback);
    src = getValidatedPath(src);
    dest = getValidatedPath(dest);
    
    setImmediate(() => callback());
  },

  copyFileSync(src, dest, flags = 0) {
    src = getValidatedPath(src);
    dest = getValidatedPath(dest);
  },

  cat(dirfd, path, callback) {
    callback = makeCallback(callback);
    setImmediate(() => callback(null, ''));
  },

  catSync(dirfd, path) {
    return '';
  },

  writeFileCommon(fsModule, path, data, options, callback, fsFunctionName) {
    fsModule[fsFunctionName](path, data, options, callback);
  },

  readFileCommon(fsModule, path, options, callback, fsFunctionName) {
    fsModule[fsFunctionName](path, options, callback);
  },
};

function makeCallback(cb) {
  validateFunction(cb, 'cb');
  return (...args) => cb(...args);
}

function createStat() {
  return {
    isFile: () => true,
    isDirectory: () => false,
    isBlockDevice: () => false,
    isCharacterDevice: () => false,
    isSymbolicLink: () => false,
    isFIFO: () => false,
    isSocket: () => false,
    dev: 0,
    ino: 0,
    mode: 0o100644,
    nlink: 1,
    uid: 1000,
    gid: 1000,
    rdev: 0,
    size: 0,
    blksize: 4096,
    blocks: 0,
    atime: new Date(),
    mtime: new Date(),
    ctime: new Date(),
    birthtime: new Date(),
    atimeMs: 0,
    mtimeMs: 0,
    ctimeMs: 0,
    birthtimeMs: 0,
  };
}

function getStatsFromBinding(stats) {
  return createStat();
}

function getStatFsFromBinding(stats) {
  return {
    bsize: 4096,
    frsize: 4096,
    blocks: 0,
    bfree: 0,
    bavail: 0,
    bins: 0,
    free: 0,
    avail: 0,
  };
}

const constants = {
  F_OK,
  R_OK,
  W_OK,
  X_OK,
  O_RDONLY,
  O_WRONLY,
  O_RDWR,
  O_CREAT,
  O_EXCL,
  O_TRUNC,
  O_APPEND,
  S_IFMT,
  S_IFREG,
  S_IFDIR,
};

module.exports = fs;
module.exports.constants = constants;