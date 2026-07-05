// koss:internal/stream - Internal stream layer (L2)
// Not directly accessible to user code. Used by L3 compatibility layers.

var Buffer = globalThis.Buffer;

function ReadStream(readFn, options) {
  this._readFn = readFn;
  this._position = 0;
  this._chunkSize = (options && options.chunkSize) || 65536;
  this._encoding = (options && options.encoding) || null;
  this._readable = true;
  this._buffer = [];
  this._listeners = {};
}

ReadStream.prototype.on = function(event, callback) {
  if (!this._listeners[event]) this._listeners[event] = [];
  this._listeners[event].push(callback);
  return this;
};

ReadStream.prototype.emit = function(event) {
  var handlers = this._listeners[event] || [];
  var args = [];
  for (var i = 1; i < arguments.length; i++) args.push(arguments[i]);
  for (var j = 0; j < handlers.length; j++) handlers[j].apply(null, args);
};

ReadStream.prototype.read = function(size) {
  try {
    var data = this._readFn();
    if (data === null || data === undefined) {
      this._readable = false;
      this.emit('end');
      return null;
    }
    var buf = typeof data === 'string' ? Buffer.from(data) : data;
    if (this._encoding === 'utf8' || this._encoding === 'utf-8') {
      this.emit('data', buf.toString());
    } else {
      this.emit('data', buf);
    }
    return buf;
  } catch (err) {
    this.emit('error', err);
    return null;
  }
};

ReadStream.prototype.pause = function() { this._paused = true; };
ReadStream.prototype.resume = function() { this._paused = false; };

Object.defineProperty(ReadStream.prototype, 'readable', {
  get: function() { return this._readable; }
});

ReadStream.prototype.pipe = function(dest) {
  this.on('data', function(chunk) { dest.write(chunk); });
  return dest;
};

function WriteStream(writeFn) {
  this._writeFn = writeFn;
  this._writable = true;
  this._listeners = {};
}

WriteStream.prototype.on = function(event, callback) {
  if (!this._listeners[event]) this._listeners[event] = [];
  this._listeners[event].push(callback);
  return this;
};

WriteStream.prototype.emit = function(event) {
  var handlers = this._listeners[event] || [];
  var args = [];
  for (var i = 1; i < arguments.length; i++) args.push(arguments[i]);
  for (var j = 0; j < handlers.length; j++) handlers[j].apply(null, args);
};

WriteStream.prototype.write = function(data) {
  try {
    this._writeFn(data);
    return true;
  } catch (err) {
    this.emit('error', err);
    return false;
  }
};

WriteStream.prototype.end = function(data) {
  if (data) this.write(data);
  this._writable = false;
  this.emit('finish');
};

Object.defineProperty(WriteStream.prototype, 'writable', {
  get: function() { return this._writable; }
});

function createReadStream(readFn, options) {
  return new ReadStream(readFn, options);
}

function createWriteStream(writeFn) {
  return new WriteStream(writeFn);
}

function pipeline() {
  var streams = [];
  for (var i = 0; i < arguments.length; i++) streams.push(arguments[i]);
  for (var j = 0; j < streams.length - 1; j++) {
    var src = streams[j];
    var dest = streams[j + 1];
    if (src && dest && typeof dest.write === 'function') {
      src.on('data', function(chunk) { dest.write(chunk); });
    }
  }
  return streams[streams.length - 1];
}

module.exports = {
  ReadStream: ReadStream,
  WriteStream: WriteStream,
  createReadStream: createReadStream,
  createWriteStream: createWriteStream,
  pipeline: pipeline,
};
