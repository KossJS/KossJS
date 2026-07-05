// koss:ffi — Koss 原生外部函数接口模块
// 基于 _senri_ffi 全局对象

var ffi = globalThis._senri_ffi;

function open(path) {
  if (!ffi) throw new Error('FFI not available: _senri_ffi not initialized');
  var lib = ffi.open(String(path));
  return {
    fn: function(name, options) {
      if (!lib) throw new Error('Library not loaded');
      var opts = options || {};
      var args = opts.args || [];
      var returns = opts.returns || 'void';
      return ffi.func(lib, String(name), args, returns);
    },
    struct: function(name, options) {
      if (!lib) throw new Error('Library not loaded');
      var opts = options || {};
      var fields = opts.fields || {};
      return ffi.struct(String(name), fields);
    },
    close: function() {
      if (lib && typeof lib.close === 'function') lib.close();
    },
  };
}

function dlopen(path) {
  return open(path);
}

function malloc(size) {
  if (!ffi) throw new Error('FFI not available');
  return ffi.alloc(Number(size));
}

function free(pointer) {
  if (!ffi) throw new Error('FFI not available');
  ffi.free(pointer);
}

function addressOf(pointer) {
  if (!ffi) throw new Error('FFI not available');
  return ffi.addressOf(pointer);
}

function createCallback(fn, signature) {
  if (!ffi) throw new Error('FFI not available');
  return ffi.createCallback(fn, signature);
}

function strerror(errno) {
  if (!ffi) return 'FFI not available';
  return ffi.strerror(Number(errno));
}

module.exports = {
  open: open, dlopen: dlopen,
  malloc: malloc, free: free, addressOf: addressOf,
  createCallback: createCallback, strerror: strerror,
};
