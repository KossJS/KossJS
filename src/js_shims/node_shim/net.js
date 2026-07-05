// koss:node/net - Node.js net module (L3)

const internalNet = require('koss:internal/net');
const _events = require('koss:node/events');
const EventEmitter = _events.EventEmitter;
const _dns = require('koss:node/dns');
const isIP = _dns.isIP;
const isIPv4 = _dns.isIPv4;
const isIPv6 = _dns.isIPv6;

class Socket extends EventEmitter {
  constructor(options) { super(); this._fd = null; this._connected = false; this._destroyed = false; this._buffer = []; this.timeout = 0; }

  connect(port, host, connectListener) {
    if (typeof host === 'function') { connectListener = host; host = 'localhost'; }
    if (typeof port === 'object') { var opts = port; port = opts.port; host = opts.host || 'localhost'; }
    if (connectListener) this.once('connect', connectListener);
    try {
      const sock = internalNet.tcpConnect(host || 'localhost', Number(port));
      this._fd = sock._fd;
      this._connected = true;
      this.emit('connect');
    } catch (err) { this.emit('error', err); }
    return this;
  }

  write(data, encoding, callback) {
    if (typeof encoding === 'function') { callback = encoding; encoding = 'utf8'; }
    try {
      if (this._fd !== null && typeof globalThis.__koss_tcp_write === 'function') {
        globalThis.__koss_tcp_write(this._fd, typeof data === 'string' ? data : String(data));
      } else {
        this._buffer.push(data);
      }
      if (callback) callback();
    } catch (err) { if (callback) callback(err); }
    return true;
  }

  end(data, encoding, callback) {
    if (typeof data === 'function') { callback = data; data = undefined; }
    else if (typeof encoding === 'function') { callback = encoding; encoding = undefined; }
    if (data) this.write(data, encoding);
    this.destroy();
    if (callback) callback();
    return this;
  }

  destroy(err) {
    if (this._destroyed) return this;
    this._destroyed = true;
    if (this._fd !== null && typeof globalThis.__koss_tcp_close === 'function') {
      globalThis.__koss_tcp_close(this._fd);
    }
    if (err) this.emit('error', err);
    this.emit('close');
    this._connected = false;
    return this;
  }

  setEncoding() { return this; }
  setKeepAlive() { return this; }
  setNoDelay() { return this; }
  setTimeout(ms, cb) { this.timeout = ms; if (cb) this.on('timeout', cb); return this; }
  address() { return { address: '127.0.0.1', port: 0, family: 'IPv4' }; }
  ref() { return this; }
  unref() { return this; }

  get readable() { return true; }
  get writable() { return !this._destroyed; }
  get connecting() { return false; }
  get destroyed() { return this._destroyed; }
  get remoteAddress() { return '127.0.0.1'; }
  get remotePort() { return 0; }
}

class Server extends EventEmitter {
  constructor(options, connectionListener) {
    super();
    if (typeof options === 'function') { connectionListener = options; options = {}; }
    if (connectionListener) this.on('connection', connectionListener);
    this._server = null;
    this._listening = false;
  }

  listen(port, host, backlog, callback) {
    if (typeof port === 'object') { var opts = port; port = opts.port; host = opts.host; }
    if (typeof host === 'function') { callback = host; host = '0.0.0.0'; }
    if (typeof backlog === 'function') { callback = backlog; backlog = undefined; }
    if (callback) this.on('listening', callback);
    try {
      this._server = internalNet.tcpListen(host || '0.0.0.0', Number(port) || 0);
      this._listening = true;
      this.emit('listening');
      // Poll for connections
      this._pollInterval = setInterval(() => {
        if (this._server) {
          const client = this._server.accept();
          if (client) {
            const socket = new Socket();
            socket._fd = client._fd;
            socket._connected = true;
            this.emit('connection', socket);
          }
        }
      }, 50);
    } catch (err) { this.emit('error', err); }
    return this;
  }

  close(callback) {
    if (callback) this.on('close', callback);
    if (this._pollInterval) clearInterval(this._pollInterval);
    if (this._server) this._server.close();
    this._listening = false;
    this.emit('close');
    return this;
  }

  address() { return this._server ? { address: '127.0.0.1', port: this._server.port || 0, family: 'IPv4' } : null; }
  get listening() { return this._listening; }
  maxConnections = 0;
  connections = 0;
  ref() { return this; }
  unref() { return this; }
}

function createServer(options, connectionListener) { return new Server(options, connectionListener); }
function connect(...args) { var s = new Socket(); s.connect(...args); return s; }
function createConnection(...args) { return connect(...args); }

module.exports = { createServer, connect, createConnection, Socket, Server, isIP, isIPv4, isIPv6 };