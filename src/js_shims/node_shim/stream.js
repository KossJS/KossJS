// koss:node/stream - Node.js stream module (L3)

const _events = require('koss:node/events');
const EventEmitter = _events.EventEmitter;
const { Buffer } = globalThis;

const kHighWaterMark = 16384;

function isDuplex(s) { return s && s._writableState && s._readableState; }

class Readable extends EventEmitter {
  constructor(options = {}) {
    super();
    this._readableState = { highWaterMark: options.highWaterMark || kHighWaterMark, encoding: options.encoding || null, objectMode: options.objectMode || false, flowing: null, ended: false, destroyed: false };
    this._read = options.read || null;
    this._buffer = [];
  }

  _read(size) {
    if (this._read) this._read(size);
    else this.push(null);
  }

  push(chunk, encoding) {
    if (chunk === null) {
      this._readableState.ended = true;
      this.emit('end');
      return false;
    }
    if (!this._readableState.objectMode && typeof chunk === 'string') {
      chunk = Buffer.from(chunk, encoding || 'utf8');
    }
    this._buffer.push(chunk);
    if (this._readableState.flowing) this.emit('data', chunk);
    return true;
  }

  pipe(dest, options) {
    this.on('data', (chunk) => {
      const ok = dest.write(chunk);
      if (!ok) this.pause();
    });
    dest.on('drain', () => this.resume());
    this.on('end', () => dest.end());
    this.on('error', (err) => dest.destroy(err));
    return dest;
  }

  pause() { this._readableState.flowing = false; return this; }
  resume() { this._readableState.flowing = true; return this; }

  read(size) {
    if (this._buffer.length === 0) {
      this._read(size);
      return null;
    }
    return this._buffer.shift();
  }

  destroy(err) {
    if (this._readableState.destroyed) return this;
    this._readableState.destroyed = true;
    if (err) this.emit('error', err);
    this.emit('close');
    return this;
  }

  setEncoding(enc) { this._readableState.encoding = enc; return this; }

  get readable() { return !this._readableState.ended && !this._readableState.destroyed; }
  get readableFlowing() { return this._readableState.flowing !== null ? this._readableState.flowing : null; }
  get readableEnded() { return this._readableState.ended; }
  get destroyed() { return this._readableState.destroyed; }

  [Symbol.asyncIterator]() {
    let done = false;
    const buffer = this._buffer;
    return {
      next() {
        if (buffer.length > 0) return Promise.resolve({ value: buffer.shift(), done: false });
        if (done) return Promise.resolve({ done: true });
        return new Promise((resolve) => {
          const onData = (chunk) => { resolve({ value: chunk, done: false }); };
          const onEnd = () => { done = true; resolve({ done: true }); };
          this.once('data', onData);
          this.once('end', onEnd);
        });
      },
    };
  }
}

class Writable extends EventEmitter {
  constructor(options = {}) {
    super();
    this._writableState = { highWaterMark: options.highWaterMark || kHighWaterMark, encoding: options.encoding || null, objectMode: options.objectMode || false, ended: false, destroyed: false, writable: true };
    this._write = options.write || null;
  }

  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    const cb = callback || (() => {});
    if (this._writableState.ended) { cb(new Error('write after end')); return false; }
    try {
      if (this._write) this._write(chunk, encoding, cb);
      else cb();
      return true;
    } catch (err) { cb(err); return false; }
  }

  end(chunk, encoding, callback) {
    if (typeof chunk === 'function') { callback = chunk; chunk = undefined; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (chunk) this.write(chunk, encoding);
    this._writableState.ended = true;
    this.emit('finish');
    if (callback) callback();
    return this;
  }

  destroy(err) {
    if (this._writableState.destroyed) return this;
    this._writableState.destroyed = true;
    if (err) this.emit('error', err);
    this.emit('close');
    return this;
  }

  get writable() { return this._writableState.writable && !this._writableState.ended; }
  get writableEnded() { return this._writableState.ended; }
  get destroyed() { return this._writableState.destroyed; }
}

class Duplex extends EventEmitter {
  constructor(options = {}) { super(); this._readable = new Readable(options); this._writable = new Writable(options); }
  _read(size) { this._readable._read(size); }
  push(chunk, encoding) { return this._readable.push(chunk, encoding); }
  write(chunk, encoding, callback) { return this._writable.write(chunk, encoding, callback); }
  end(chunk, encoding, callback) { return this._writable.end(chunk, encoding, callback); }
  destroy(err) { this._readable.destroy(err); this._writable.destroy(err); return this; }
  get readable() { return this._readable.readable; }
  get writable() { return this._writable.writable; }
  pipe(dest) { return this._readable.pipe(dest); }
}

class Transform extends Duplex {
  constructor(options = {}) {
    super(options);
    this._transform = options.transform || null;
    this._flush = options.flush || null;
  }
  _transform(chunk, encoding, callback) {
    this.push(chunk);
    callback();
  }
  write(chunk, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    try {
      if (this._transform) this._transform(chunk, encoding, callback || (() => {}));
      else { this._transform(chunk, encoding, callback || (() => {})); }
    } catch (err) { if (callback) callback(err); }
    return true;
  }
}

class PassThrough extends Transform {
  _transform(chunk, encoding, callback) { this.push(chunk); callback(); }
}

function pipeline(...streams) {
  const lastCallback = typeof streams[streams.length - 1] === 'function' ? streams.pop() : null;
  let src = streams[0];
  for (let i = 1; i < streams.length; i++) {
    const dest = streams[i];
    if (src && dest) {
      if (typeof src.pipe === 'function') {
        src.pipe(dest);
      } else if (typeof src.on === 'function') {
        src.on('data', (chunk) => dest.write(chunk));
        src.on('end', () => dest.end());
      }
    }
    src = dest;
  }
  if (lastCallback) process.nextTick(lastCallback);
  return src;
}

function finished(stream, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  const cb = callback || (() => {});
  const done = () => { stream.removeListener('end', done); stream.removeListener('finish', done); stream.removeListener('close', done); cb(null); };
  const errHandler = (err) => { stream.removeListener('error', errHandler); cb(err); };
  stream.on('end', done);
  stream.on('finish', done);
  stream.on('close', done);
  stream.on('error', errHandler);
  return () => { stream.removeListener('end', done); stream.removeListener('finish', done); stream.removeListener('close', done); stream.removeListener('error', errHandler); };
}

function compose(...streams) { throw new Error('stream.compose not implemented'); }
function addAbortSignal(signal, stream) { return stream; }

module.exports = { Readable, Writable, Duplex, Transform, PassThrough, pipeline, finished, compose, addAbortSignal };