// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:stream — Koss 标准库 Stream 模块
// Node.js Stream API 兼容实现，纯 JS，无外部依赖

var Buffer = globalThis.Buffer || require('koss:buffer').Buffer;
var EventEmitter = require('koss:events');

var kDefaultHighWaterMark = 16384;
var kMaxListeners = 16;

// ═══════════════════════════════════════════
// Stream 基类
// ═══════════════════════════════════════════

function Stream(options) {
  EventEmitter.call(this);
  var opts = options || {};
  try {
    this.readable = opts.readable !== false;
    this.writable = opts.writable !== false;
  } catch (e) {
    // 子类原型可能定义了只读 getter（Readable/Writable/Duplex），此时忽略赋值
  }
}

Stream.prototype = Object.create(EventEmitter.prototype);
Stream.prototype.constructor = Stream;

Stream.prototype.pipe = function(dest, options) {
  return dest;
};

Stream.prototype.unpipe = function(dest) {
  return this;
};

Stream.prototype.setEncoding = function(enc) {
  return this;
};

// ═══════════════════════════════════════════
// Readable 流
// ═══════════════════════════════════════════

function Readable(options) {
  Stream.call(this, options);
  var opts = options || {};
  this._readableState = {
    highWaterMark: opts.highWaterMark || kDefaultHighWaterMark,
    encoding: opts.encoding || null,
    objectMode: opts.objectMode || false,
    flowing: null,
    ended: false,
    destroyed: false,
    length: 0,
  };
  this._readFn = opts._read || opts.read || null;
  this._buffer = [];
}

Readable.prototype = Object.create(Stream.prototype);
Readable.prototype.constructor = Readable;

Readable.prototype._read = function(size) {
  if (this._readFn) {
    this._readFn.call(this, size);
  } else {
    this.push(null);
  }
};

Readable.prototype.push = function(chunk, encoding) {
  var state = this._readableState;

  if (chunk === null) {
    state.ended = true;
    this.emit('end');
    return false;
  }

  if (!state.objectMode && typeof chunk === 'string') {
    chunk = Buffer.from(chunk, encoding || 'utf8');
  }

  this._buffer.push(chunk);
  state.length += chunk.length;

  if (state.flowing !== null && state.flowing) {
    this.emit('data', chunk);
  }

  return true;
};

Readable.prototype.unshift = function(chunk) {
  this._buffer.unshift(chunk);
  if (chunk) {
    this._readableState.length += chunk.length;
  }
};

Readable.prototype.read = function(size) {
  var state = this._readableState;

  if (this._buffer.length === 0) {
    this._read(size);
    return null;
  }

  var chunk = this._buffer.shift();
  if (chunk) {
    state.length -= chunk.length;
  }

  if (state.encoding) {
    return chunk ? chunk.toString(state.encoding) : chunk;
  }

  return chunk;
};

Readable.prototype.pipe = function(dest, options) {
  var source = this;

  function onData(chunk) {
    var ret = dest.write(chunk);
    if (!ret) {
      source.pause();
    }
  }

  function onDrain() {
    source.resume();
  }

  function onEnd() {
    dest.end();
  }

  function onError(err) {
    dest.destroy(err);
  }

  source.on('data', onData);
  dest.on('drain', onDrain);
  source.on('end', onEnd);
  source.on('error', onError);

  return dest;
};

Readable.prototype.unpipe = function(dest) {
  if (!dest) {
    this.removeAllListeners('data');
    return this;
  }
  this.removeListener('data', dest._onData);
  this.removeListener('end', dest._onEnd);
  return this;
};

Readable.prototype.pause = function() {
  this._readableState.flowing = false;
  this.emit('pause');
  return this;
};

Readable.prototype.resume = function() {
  this._readableState.flowing = true;
  this.emit('resume');

  var self = this;
  function tick() {
    if (self._readableState.flowing && self._buffer.length > 0) {
      var chunk = self.read();
      if (chunk !== null) {
        self.emit('data', chunk);
      }
      tick();
    }
  }
  tick();

  return this;
};

Readable.prototype.setEncoding = function(enc) {
  this._readableState.encoding = enc;
  return this;
};

Readable.prototype.destroy = function(err) {
  var state = this._readableState;
  if (state.destroyed) return this;
  state.destroyed = true;
  if (err) this.emit('error', err);
  this.emit('close');
  return this;
};

Object.defineProperty(Readable.prototype, 'readable', {
  get: function() {
    return !this._readableState.ended && !this._readableState.destroyed;
  }
});

Object.defineProperty(Readable.prototype, 'readableFlowing', {
  get: function() {
    return this._readableState.flowing;
  }
});

Object.defineProperty(Readable.prototype, 'readableEnded', {
  get: function() {
    return this._readableState.ended;
  }
});

Object.defineProperty(Readable.prototype, 'destroyed', {
  get: function() {
    return this._readableState.destroyed;
  }
});

Object.defineProperty(Readable.prototype, 'readableObjectMode', {
  get: function() {
    return this._readableState.objectMode;
  }
});

Object.defineProperty(Readable.prototype, 'readableLength', {
  get: function() {
    return this._readableState.length;
  }
});

Readable.prototype[Symbol.asyncIterator] = function() {
  var self = this;
  var buffer = this._buffer;
  var done = false;

  return {
    next: function() {
      if (buffer.length > 0) {
        var chunk = buffer.shift();
        return Promise.resolve({ value: chunk, done: false });
      }
      if (done) {
        return Promise.resolve({ done: true });
      }
      return new Promise(function(resolve, reject) {
        function onData(chunk) {
          cleanup();
          resolve({ value: chunk, done: false });
        }
        function onEnd() {
          done = true;
          cleanup();
          resolve({ done: true });
        }
        function onError(err) {
          cleanup();
          reject(err);
        }
        function cleanup() {
          self.removeListener('data', onData);
          self.removeListener('end', onEnd);
          self.removeListener('error', onError);
        }
        self.on('data', onData);
        self.on('end', onEnd);
        self.on('error', onError);
        self.resume();
      });
    },
    return: function() {
      done = true;
      return Promise.resolve({ done: true });
    },
    [Symbol.asyncIterator]: function() { return this; }
  };
};

// ═══════════════════════════════════════════
// Writable 流
// ═══════════════════════════════════════════

function Writable(options) {
  Stream.call(this, options);
  var opts = options || {};
  this._writableState = {
    highWaterMark: opts.highWaterMark || kDefaultHighWaterMark,
    encoding: opts.encoding || null,
    objectMode: opts.objectMode || false,
    writable: true,
    ended: false,
    destroyed: false,
    writecb: null,
    writecount: 0,
    length: 0,
  };
  this._writeFn = opts._write || opts.write || null;
}

Writable.prototype = Object.create(Stream.prototype);
Writable.prototype.constructor = Writable;

Writable.prototype._write = function(chunk, encoding, callback) {
  if (this._writeFn) {
    this._writeFn.call(this, chunk, encoding, callback);
  } else {
    callback();
  }
};

Writable.prototype.write = function(chunk, encoding, callback) {
  var state = this._writableState;

  if (typeof encoding === 'function') {
    callback = encoding;
    encoding = 'utf8';
  }

  var cb = callback || function() {};

  if (state.ended) {
    cb(new Error('write after end'));
    return false;
  }

  if (state.destroyed) {
    cb(new Error('write after destroy'));
    return false;
  }

  if (!state.objectMode && typeof chunk === 'string') {
    chunk = Buffer.from(chunk, encoding || 'utf8');
  }

  state.writecount++;

  try {
    this._write(chunk, encoding || 'utf8', function(err) {
      if (err) {
        cb(err);
        return;
      }
      if (chunk && typeof chunk.length === 'number') {
        state.length = Math.max(0, state.length - chunk.length);
      }
      cb();
    });
    if (chunk && typeof chunk.length === 'number') {
      state.length += chunk.length;
    }
    return true;
  } catch (err) {
    cb(err);
    return false;
  }
};

Writable.prototype.end = function(chunk, encoding, callback) {
  var state = this._writableState;

  if (typeof chunk === 'function') {
    callback = chunk;
    chunk = undefined;
  } else if (typeof encoding === 'function') {
    callback = encoding;
    encoding = undefined;
  }

  if (chunk && typeof chunk !== 'function') {
    this.write(chunk, encoding);
  }

  state.ended = true;

  if (typeof callback === 'function') {
    this.once('finish', callback);
  }

  this.emit('finish');
  return this;
};

Writable.prototype.destroy = function(err) {
  var state = this._writableState;
  if (state.destroyed) return this;
  state.destroyed = true;
  if (err) this.emit('error', err);
  this.emit('close');
  return this;
};

Object.defineProperty(Writable.prototype, 'writable', {
  get: function() {
    return this._writableState && this._writableState.writable !== false && !this._writableState.ended && !this._writableState.destroyed;
  }
});

Object.defineProperty(Writable.prototype, 'writableEnded', {
  get: function() {
    return this._writableState.ended;
  }
});

Object.defineProperty(Writable.prototype, 'writableFinished', {
  get: function() {
    return this._writableState.ended;
  }
});

Object.defineProperty(Writable.prototype, 'writableObjectMode', {
  get: function() {
    return this._writableState.objectMode;
  }
});

Object.defineProperty(Writable.prototype, 'writableLength', {
  get: function() {
    return this._writableState ? this._writableState.length || 0 : 0;
  }
});

// ═══════════════════════════════════════════
// Duplex 流
// ═══════════════════════════════════════════

function Duplex(options) {
  Stream.call(this, options);
  var opts = options || {};
  this._readableState = {
    highWaterMark: opts.highWaterMark || kDefaultHighWaterMark,
    encoding: opts.encoding || null,
    objectMode: opts.objectMode || false,
    flowing: null,
    ended: false,
    destroyed: false,
    length: 0,
  };
  this._writableState = {
    highWaterMark: opts.highWaterMark || kDefaultHighWaterMark,
    encoding: opts.encoding || null,
    objectMode: opts.objectMode || false,
    writable: true,
    ended: false,
    destroyed: false,
    writecb: null,
    writecount: 0,
    length: 0,
  };
  this._readFn = opts._read || opts.read || null;
  this._writeFn = opts._write || opts.write || null;
  this._buffer = [];
}

Duplex.prototype = Object.create(Stream.prototype);
Duplex.prototype.constructor = Duplex;

Duplex.prototype._read = function(size) {
  if (this._readFn) {
    this._readFn.call(this, size);
  } else {
    this.push(null);
  }
};

Duplex.prototype._write = function(chunk, encoding, callback) {
  if (this._writeFn) {
    this._writeFn.call(this, chunk, encoding, callback);
  } else {
    callback();
  }
};

Duplex.prototype.push = function(chunk, encoding) {
  var state = this._readableState;

  if (chunk === null) {
    state.ended = true;
    this.emit('end');
    return false;
  }

  if (!state.objectMode && typeof chunk === 'string') {
    chunk = Buffer.from(chunk, encoding || 'utf8');
  }

  this._buffer.push(chunk);
  state.length += chunk.length;

  if (state.flowing !== null && state.flowing) {
    this.emit('data', chunk);
  }

  return true;
};

Duplex.prototype.read = function(size) {
  var state = this._readableState;

  if (this._buffer.length === 0) {
    this._read(size);
    return null;
  }

  var chunk = this._buffer.shift();
  if (chunk) {
    state.length -= chunk.length;
  }

  if (state.encoding) {
    return chunk ? chunk.toString(state.encoding) : chunk;
  }

  return chunk;
};

Duplex.prototype.write = function(chunk, encoding, callback) {
  var state = this._writableState;

  if (typeof encoding === 'function') {
    callback = encoding;
    encoding = 'utf8';
  }

  var cb = callback || function() {};

  if (state.ended) {
    cb(new Error('write after end'));
    return false;
  }

  if (state.destroyed) {
    cb(new Error('write after destroy'));
    return false;
  }

  if (!state.objectMode && typeof chunk === 'string') {
    chunk = Buffer.from(chunk, encoding || 'utf8');
  }

  state.writecount++;

  try {
    this._write(chunk, encoding || 'utf8', function(err) {
      if (err) {
        cb(err);
        return;
      }
      cb();
    });
    return true;
  } catch (err) {
    cb(err);
    return false;
  }
};

Duplex.prototype.end = function(chunk, encoding, callback) {
  var state = this._writableState;

  if (typeof chunk === 'function') {
    callback = chunk;
    chunk = undefined;
  } else if (typeof encoding === 'function') {
    callback = encoding;
    encoding = undefined;
  }

  if (chunk && typeof chunk !== 'function') {
    this.write(chunk, encoding);
  }

  state.ended = true;

  if (typeof callback === 'function') {
    this.once('finish', callback);
  }

  this.emit('finish');
  return this;
};

Duplex.prototype.pipe = function(dest, options) {
  var source = this;

  function onData(chunk) {
    var ret = dest.write(chunk);
    if (!ret) {
      source.pause();
    }
  }

  function onDrain() {
    source.resume();
  }

  function onEnd() {
    dest.end();
  }

  function onError(err) {
    dest.destroy(err);
  }

  source.on('data', onData);
  dest.on('drain', onDrain);
  source.on('end', onEnd);
  source.on('error', onError);

  return dest;
};

Duplex.prototype.pause = function() {
  this._readableState.flowing = false;
  this.emit('pause');
  return this;
};

Duplex.prototype.resume = function() {
  this._readableState.flowing = true;
  this.emit('resume');

  var self = this;
  function tick() {
    if (self._readableState.flowing && self._buffer.length > 0) {
      var chunk = self.read();
      if (chunk !== null) {
        self.emit('data', chunk);
      }
      tick();
    }
  }
  tick();

  return this;
};

Duplex.prototype.setEncoding = function(enc) {
  this._readableState.encoding = enc;
  return this;
};

Duplex.prototype.destroy = function(err) {
  var readState = this._readableState;
  var writeState = this._writableState;
  if (readState.destroyed && writeState.destroyed) return this;
  readState.destroyed = true;
  writeState.destroyed = true;
  if (err) this.emit('error', err);
  this.emit('close');
  return this;
};

Object.defineProperty(Duplex.prototype, 'readable', {
  get: function() {
    return !this._readableState.ended && !this._readableState.destroyed;
  }
});

Object.defineProperty(Duplex.prototype, 'writable', {
  get: function() {
    return !this._writableState.ended && !this._writableState.destroyed;
  }
});

Object.defineProperty(Duplex.prototype, 'readableFlowing', {
  get: function() {
    return this._readableState.flowing;
  }
});

Object.defineProperty(Duplex.prototype, 'readableEnded', {
  get: function() {
    return this._readableState.ended;
  }
});

Object.defineProperty(Duplex.prototype, 'writableEnded', {
  get: function() {
    return this._writableState.ended;
  }
});

Object.defineProperty(Duplex.prototype, 'writableFinished', {
  get: function() {
    return this._writableState.ended;
  }
});

Object.defineProperty(Duplex.prototype, 'destroyed', {
  get: function() {
    return this._readableState.destroyed || this._writableState.destroyed;
  }
});

Object.defineProperty(Duplex.prototype, 'readableObjectMode', {
  get: function() {
    return this._readableState.objectMode;
  }
});

Object.defineProperty(Duplex.prototype, 'writableObjectMode', {
  get: function() {
    return this._writableState.objectMode;
  }
});

Object.defineProperty(Duplex.prototype, 'readableLength', {
  get: function() {
    return this._readableState.length;
  }
});

Object.defineProperty(Duplex.prototype, 'writableLength', {
  get: function() {
    return 0;
  }
});

// ═══════════════════════════════════════════
// Transform 流
// ═══════════════════════════════════════════

function Transform(options) {
  Duplex.call(this, options);
  var opts = options || {};
  this._transformFn = opts.transform || null;
  this._flushFn = opts.flush || null;
  this._transformState = {
    afterTransform: function(err, data) {
      this.needTransform = false;
    }.bind(this),
    writing: false,
    writecb: null,
    buffer: [],
  };
}

Transform.prototype = Object.create(Duplex.prototype);
Transform.prototype.constructor = Transform;

Transform.prototype._transform = function(chunk, encoding, callback) {
  if (this._transformFn) {
    this._transformFn.call(this, chunk, encoding, callback);
  } else {
    this.push(chunk);
    callback();
  }
};

Transform.prototype._flush = function(callback) {
  if (this._flushFn) {
    this._flushFn.call(this, callback);
  } else {
    callback();
  }
};

Transform.prototype._write = function(chunk, encoding, callback) {
  var state = this._transformState;
  state.writing = true;

  var self = this;
  this._transform(chunk, encoding, function(err, data) {
    state.writing = false;
    if (err) {
      callback(err);
      return;
    }
    if (data !== undefined) {
      self._transformState.buffer.push(data);
      self._emitReadable();
    }
    callback();
  });
};

Transform.prototype._emitReadable = function() {
  if (this._readableState && this._readableState.flowing) {
    while (this._transformState.buffer.length > 0) {
      this.push(this._transformState.buffer.shift());
    }
  }
};

Transform.prototype._read = function(size) {
  var state = this._transformState;
  if (state.buffer.length > 0) {
    var chunk = state.buffer.shift();
    this.push(chunk);
  }
};

Transform.prototype.end = function(chunk, encoding, callback) {
  var self = this;
  Duplex.prototype.end.call(this, chunk, encoding, function() {
    self._flush(function(err) {
      if (err) {
        self.emit('error', err);
      }
    });
  });
  return this;
};

// ═══════════════════════════════════════════
// PassThrough 流
// ═══════════════════════════════════════════

function PassThrough(options) {
  Transform.call(this, options);
}

PassThrough.prototype = Object.create(Transform.prototype);
PassThrough.prototype.constructor = PassThrough;

PassThrough.prototype._transform = function(chunk, encoding, callback) {
  this.push(chunk);
  callback();
};

// ═══════════════════════════════════════════
// pipeline 工具
// ═══════════════════════════════════════════

function pipeline() {
  var streams = [];
  for (var i = 0; i < arguments.length; i++) {
    streams.push(arguments[i]);
  }

  var lastCallback = null;
  if (typeof streams[streams.length - 1] === 'function') {
    lastCallback = streams.pop();
  }

  var src = streams[0];
  var error = null;

  function onError(err) {
    error = err;
    if (lastCallback) {
      lastCallback(err);
    }
  }

  for (var j = 0; j < streams.length - 1; j++) {
    var dest = streams[j + 1];
    if (src && dest) {
      if (typeof src.pipe === 'function') {
        src.pipe(dest);
      } else if (typeof src.on === 'function') {
        src.on('data', function(chunk) {
          dest.write(chunk);
        });
        src.on('end', function() {
          dest.end();
        });
      }
      src.on('error', onError);
    }
    src = dest;
  }

  if (lastCallback && !error) {
    var lastStream = streams[streams.length - 1];
    if (lastStream && typeof lastStream.on === 'function') {
      lastStream.on('finish', function() {
        lastCallback(null);
      });
      lastStream.on('error', lastCallback);
    } else {
      setTimeout(function() {
        lastCallback(null);
      }, 0);
    }
  }

  return streams[streams.length - 1];
}

// ═══════════════════════════════════════════
// compose 工具
// ═══════════════════════════════════════════

function compose() {
  var streams = [];
  for (var i = 0; i < arguments.length; i++) {
    streams.push(arguments[i]);
  }

  var lastStream = null;

  for (var j = 0; j < streams.length; j++) {
    var stream = streams[j];
    if (typeof stream === 'function') {
      var fn = stream;
      stream = new Transform({
        transform: fn
      });
    }
    if (lastStream) {
      lastStream.pipe(stream);
    }
    lastStream = stream;
  }

  return lastStream;
}

// ═══════════════════════════════════════════
// addAbortSignal 工具
// ═══════════════════════════════════════════

function addAbortSignal(signal, stream) {
  if (signal && typeof signal.addEventListener === 'function') {
    signal.addEventListener('abort', function() {
      stream.destroy(new Error('Aborted'));
    }, { once: true });
  }
  return stream;
}

// ═══════════════════════════════════════════
// finished 工具
// ═══════════════════════════════════════════

function finished(stream, options, callback) {
  if (typeof options === 'function') {
    callback = options;
    options = {};
  }

  var cb = callback || function() {};

  function onEnd() {
    cleanup();
    cb(null);
  }

  function onFinish() {
    cleanup();
    cb(null);
  }

  function onError(err) {
    cleanup();
    cb(err);
  }

  function cleanup() {
    stream.removeListener('end', onEnd);
    stream.removeListener('finish', onFinish);
    stream.removeListener('error', onError);
  }

  stream.on('end', onEnd);
  stream.on('finish', onFinish);
  stream.on('error', onError);

  return cleanup;
}

// ═══════════════════════════════════════════
// isDisturbed / isDestroyed 工具
// ═══════════════════════════════════════════

function isDisturbed(stream) {
  if (stream && stream._readableState) {
    return stream._readableState.flowing !== null || stream._readableState.ended;
  }
  return false;
}

function isDestroyed(stream) {
  if (stream && stream.destroyed !== undefined) return stream.destroyed;
  return false;
}

// ═══════════════════════════════════════════
// Module Exports
// ═══════════════════════════════════════════

module.exports = {
  Stream: Stream,
  Readable: Readable,
  Writable: Writable,
  Duplex: Duplex,
  Transform: Transform,
  PassThrough: PassThrough,
  pipeline: pipeline,
  compose: compose,
  addAbortSignal: addAbortSignal,
  finished: finished,
  isDisturbed: isDisturbed,
  isDestroyed: isDestroyed,
};

module.exports.default = Stream;
module.exports.Stream = Stream;
module.exports.Readable = Readable;
module.exports.Writable = Writable;
module.exports.Duplex = Duplex;
module.exports.Transform = Transform;
module.exports.PassThrough = PassThrough;
module.exports.pipeline = pipeline;
module.exports.compose = compose;
module.exports.addAbortSignal = addAbortSignal;
module.exports.finished = finished;
module.exports.isDisturbed = isDisturbed;
module.exports.isDestroyed = isDestroyed;
