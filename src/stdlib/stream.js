'use strict';

var EventEmitter = require('events');
var timers = require('timers');

function on(obj, ev, fn) {
  obj.on(ev, fn);
  return function() { obj.removeListener(ev, fn); };
}

function isDuplex(obj) {
  return obj && typeof obj._read === 'function' && typeof obj._write === 'function';
}

function isReadable(obj) {
  return obj && typeof obj._read === 'function';
}

function isWritable(obj) {
  return obj && typeof obj._write === 'function';
}

function highWaterMark(options) {
  return (options && options.highWaterMark) || 16384;
}

function Readable(options) {
  if (!(this instanceof Readable)) return new Readable(options);
  EventEmitter.call(this);
  this._readableState = {
    highWaterMark: highWaterMark(options),
    buffer: [],
    flowing: false,
    ended: false,
    endEmitted: false,
    reading: false,
    destroyed: false,
  };
  this._read = (options && options.read) || function() {};
}
Readable.prototype = Object.create(EventEmitter.prototype, { constructor: { value: Readable } });

Readable.prototype.push = function(chunk) {
  var state = this._readableState;
  if (chunk === null) {
    state.ended = true;
    if (state.flowing) this.emit('end');
    return false;
  }
  if (state.flowing) {
    this.emit('data', chunk);
    return true;
  }
  state.buffer.push(chunk);
  return state.buffer.length < state.highWaterMark;
};

Readable.prototype.pipe = function(dest, options) {
  var self = this;
  var end = !options || options.end !== false;

  self.on('data', function(data) {
    var ok = dest.write(data);
    if (!ok && self.pause) self.pause();
  });

  dest.on('drain', function() {
    if (self.resume) self.resume();
  });

  if (end) {
    self.on('end', function() {
      dest.end();
    });
  }

  self.on('error', function(err) { dest.emit('error', err); });
  dest.on('error', function(err) { self.emit('error', err); });

  return dest;
};

Readable.prototype.pause = function() {
  this._readableState.flowing = false;
  return this;
};

Readable.prototype.resume = function() {
  this._readableState.flowing = true;
  if (this._readableState.buffer.length > 0) {
    var buf = this._readableState.buffer;
    this._readableState.buffer = [];
    for (var i = 0; i < buf.length; i++) {
      this.emit('data', buf[i]);
    }
  }
  if (this._readableState.ended) {
    this.emit('end');
  }
  return this;
};

Readable.prototype.isPaused = function() {
  return !this._readableState.flowing;
};

Readable.prototype.read = function(n) {
  if (n === undefined) {
    if (this._readableState.buffer.length > 0) {
      return this._readableState.buffer.shift();
    }
    return null;
  }
  var chunks = [];
  var len = 0;
  while (this._readableState.buffer.length > 0 && len < n) {
    var c = this._readableState.buffer.shift();
    chunks.push(c);
    len += c.length;
  }
  if (chunks.length === 0) return null;
  if (chunks.length === 1) return chunks[0];
  return Buffer.concat(chunks, len);
};

Readable.prototype.setEncoding = function(enc) {
  this._readableState.encoding = enc;
  return this;
};

Readable.prototype.destroy = function(err) {
  this._readableState.destroyed = true;
  if (err) this.emit('error', err);
  this.emit('close');
};

Readable.prototype.wrap = function(oldStream) {
  var self = this;
  oldStream.on('data', function(chunk) { self.push(chunk); });
  oldStream.on('end', function() { self.push(null); });
  return self;
};

function Writable(options) {
  if (!(this instanceof Writable)) return new Writable(options);
  EventEmitter.call(this);
  this._writableState = {
    highWaterMark: highWaterMark(options),
    writing: false,
    ended: false,
    destroyed: false,
    needDrain: false,
    corked: 0,
    buffer: [],
  };
  this._write = (options && options.write) || function(chunk, enc, cb) { cb(null); };
  this._writev = options && options.writev;
  this._final = (options && options.final) || function(cb) { cb(null); };
}
Writable.prototype = Object.create(EventEmitter.prototype, { constructor: { value: Writable } });

Writable.prototype.write = function(chunk, encoding, callback) {
  if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  var state = this._writableState;
  if (state.ended) {
    if (callback) process.nextTick(callback);
    return false;
  }
  if (state.corked > 0) {
    state.buffer.push({ chunk: chunk, encoding: encoding, callback: callback });
    return false;
  }
  var cb = callback || function() {};
  if (state.writing) {
    if (state.buffer.length < state.highWaterMark) {
      state.buffer.push({ chunk: chunk, encoding: encoding, callback: cb });
      return true;
    }
    state.needDrain = true;
    state.buffer.push({ chunk: chunk, encoding: encoding, callback: cb });
    return false;
  }
  state.writing = true;
  var self = this;
  this._write(chunk, encoding || 'utf8', function(err) {
    state.writing = false;
    if (err) {
      self.emit('error', err);
      return;
    }
    if (state.buffer.length > 0) {
      var next = state.buffer.shift();
      self.write(next.chunk, next.encoding, next.callback);
    } else if (state.needDrain) {
      state.needDrain = false;
      self.emit('drain');
    }
    if (cb) cb(err);
  });
  return state.buffer.length < state.highWaterMark;
};

Writable.prototype.cork = function() { this._writableState.corked++; };
Writable.prototype.uncork = function() {
  if (--this._writableState.corked <= 0) {
    this._writableState.corked = 0;
    this._flushBuffer();
  }
};

Writable.prototype._flushBuffer = function() {
  var state = this._writableState;
  while (state.buffer.length > 0 && !state.writing) {
    var next = state.buffer.shift();
    this.write(next.chunk, next.encoding, next.callback);
  }
};

Writable.prototype.setDefaultEncoding = function(enc) { return this; };

Writable.prototype.end = function(chunk, encoding, callback) {
  if (typeof chunk === 'function') { callback = chunk; chunk = undefined; }
  if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
  if (chunk !== undefined) this.write(chunk, encoding);
  this._writableState.ended = true;
  if (callback) this.once('finish', callback);
  var self = this;
  process.nextTick(function() {
    self._final(function(err) {
      if (err) self.emit('error', err);
      self.emit('finish');
    });
  });
};

Writable.prototype.destroy = function(err) {
  this._writableState.destroyed = true;
  if (err) this.emit('error', err);
  this.emit('close');
};

function Duplex(options) {
  if (!(this instanceof Duplex)) return new Duplex(options);
  Readable.call(this, options);
  Writable.call(this, options);
}
Duplex.prototype = Object.create(Readable.prototype, { constructor: { value: Duplex } });
Object.keys(Writable.prototype).forEach(function(k) {
  if (!Duplex.prototype[k]) Duplex.prototype[k] = Writable.prototype[k];
});

function Transform(options) {
  if (!(this instanceof Transform)) return new Transform(options);
  Duplex.call(this, options);
  this._transform = (options && options.transform) || function(chunk, enc, cb) { cb(null, chunk); };
  this._flush = (options && options.flush) || function(cb) { cb(null); };
}
Transform.prototype = Object.create(Duplex.prototype, { constructor: { value: Transform } });

Transform.prototype._write = function(chunk, encoding, callback) {
  var self = this;
  this._transform(chunk, encoding, function(err, data) {
    if (err) { callback(err); return; }
    if (data !== undefined) self.push(data);
    callback(null);
  });
};

Transform.prototype._read = function(n) {
  this.resume();
};

function PassThrough(options) {
  if (!(this instanceof PassThrough)) return new PassThrough(options);
  Transform.call(this, options);
}
PassThrough.prototype = Object.create(Transform.prototype, { constructor: { value: PassThrough } });
PassThrough.prototype._transform = function(chunk, encoding, callback) {
  callback(null, chunk);
};

function finished(stream, opts, cb) {
  if (typeof opts === 'function') { cb = opts; opts = {}; }
  if (!cb) return finished(stream, opts, undefined);
  stream.on('end', cb);
  stream.on('finish', cb);
  stream.on('error', cb);
  stream.on('close', cb);
  return function() {
    stream.removeListener('end', cb);
    stream.removeListener('finish', cb);
    stream.removeListener('error', cb);
    stream.removeListener('close', cb);
  };
}

function pipeline() {
  var streams = Array.prototype.slice.call(arguments);
  var cb = typeof streams[streams.length - 1] === 'function' ? streams.pop() : null;
  var src = streams[0];
  var dest = streams[streams.length - 1];

  var i = 0;
  function next(err) {
    if (err) {
      if (cb) cb(err);
      return;
    }
    if (i >= streams.length - 1) {
      if (cb) cb(null);
      return;
    }
    streams[i].pipe(streams[i + 1]);
    i++;
    next(null);
  }

  src.on('error', function(err) {
    dest.destroy(err);
    if (cb) cb(err);
  });
  dest.on('error', function(err) {
    src.destroy(err);
    if (cb) cb(err);
  });
  dest.on('finish', function() {
    if (cb) cb(null);
  });

  src.pipe(dest);
  return dest;
}

function compose() { throw new Error('stream.compose not implemented'); }
function addAbortSignal() { throw new Error('stream.addAbortSignal not implemented'); }

function isReadableStream(stream) { return stream instanceof Readable; }
function isWritableStream(stream) { return stream instanceof Writable; }
function isTransformStream(stream) { return stream instanceof Transform; }
function isStream(stream) { return stream instanceof Readable || stream instanceof Writable; }

module.exports = {
  Readable: Readable,
  Writable: Writable,
  Duplex: Duplex,
  Transform: Transform,
  PassThrough: PassThrough,
  finished: finished,
  pipeline: pipeline,
  compose: compose,
  addAbortSignal: addAbortSignal,
  isReadable: isReadableStream,
  isWritable: isWritableStream,
  isTransform: isTransformStream,
  isStream: isStream,
  promises: {
    pipeline: function() { return Promise.reject(new Error('not implemented')); },
    finished: function() { return Promise.reject(new Error('not implemented')); },
  },
  Stream: Readable,
};
