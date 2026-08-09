// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:internal/net - Internal network layer (L2)
// Not directly accessible to user code. Used by L3 compatibility layers.

var __koss_fetch = globalThis.__koss_fetch;
var __koss_tcp_connect = globalThis.__koss_tcp_connect;
var __koss_tcp_listen = globalThis.__koss_tcp_listen;
var __koss_tcp_write = globalThis.__koss_tcp_write;
var __koss_tcp_read = globalThis.__koss_tcp_read;
var __koss_tcp_close = globalThis.__koss_tcp_close;
var __koss_tcp_accept = globalThis.__koss_tcp_accept;
var __koss_dns_lookup = globalThis.__koss_dns_lookup;

function _toBinaryString(data) {
  if (typeof data === 'string') return data;
  if (data instanceof Uint8Array) {
    var s = '';
    for (var i = 0; i < data.length; i++) s += String.fromCharCode(data[i]);
    return s;
  }
  if (data && data._data instanceof Uint8Array) {
    var s2 = '';
    for (var j = 0; j < data._data.length; j++) s2 += String.fromCharCode(data._data[j]);
    return s2;
  }
  if (Array.isArray(data)) {
    var s3 = '';
    for (var k = 0; k < data.length; k++) s3 += String.fromCharCode(data[k] & 0xff);
    return s3;
  }
  return String(data);
}

function createSocket(fd) {
  return {
    _fd: fd,
    write: function(data) {
      if (typeof __koss_tcp_write === 'function') {
        return __koss_tcp_write(fd, _toBinaryString(data));
      }
      throw new Error('TCP write not available');
    },
    read: function() {
      if (typeof __koss_tcp_read === 'function') {
        return __koss_tcp_read(fd);
      }
      throw new Error('TCP read not available');
    },
    close: function() {
      if (typeof __koss_tcp_close === 'function') {
        return __koss_tcp_close(fd);
      }
    },
  };
}

function tcpConnect(host, port) {
  if (typeof __koss_tcp_connect === 'function') {
    var fd = __koss_tcp_connect(String(host), Number(port));
    if (fd === undefined || fd === null) {
      throw new Error('Failed to connect to ' + host + ':' + port);
    }
    return createSocket(Number(fd));
  }
  throw new Error('TCP connect not available');
}

function tcpListen(host, port, handler) {
  if (typeof __koss_tcp_listen === 'function') {
    var serverFd = __koss_tcp_listen(String(host), Number(port));
    if (serverFd === undefined || serverFd === null) {
      throw new Error('Failed to listen on ' + host + ':' + port);
    }
    var fd = Number(serverFd);
    return {
      _fd: fd,
      port: Number(port),
      hostname: String(host),
      accept: function() {
        if (typeof __koss_tcp_accept === 'function') {
          var clientFd = __koss_tcp_accept(fd);
          if (clientFd !== undefined && clientFd !== null) {
            return createSocket(Number(clientFd));
          }
        }
        return null;
      },
      close: function() {
        if (typeof __koss_tcp_close === 'function') {
          __koss_tcp_close(fd);
        }
      },
    };
  }
  throw new Error('TCP listen not available');
}

function dnsLookup(hostname) {
  if (typeof __koss_dns_lookup === 'function') {
    var result = __koss_dns_lookup(String(hostname));
    if (result && typeof result === 'string') {
      try { return JSON.parse(result); } catch (e) { return [result]; }
    }
    return [];
  }
  throw new Error('DNS lookup not available');
}

function httpFetch(url, options) {
  var opts = options || {};
  if (typeof __koss_fetch !== 'function') {
    if (typeof globalThis.fetch === 'function') {
      return globalThis.fetch(url, opts);
    }
    throw new Error('fetch not available');
  }

  var method = opts.method || 'GET';
  var headers = opts.headers || {};
  var body = opts.body;

  var bodyStr = undefined;
  if (body !== undefined && body !== null) {
    if (typeof body === 'string') {
      bodyStr = body;
    } else if (body instanceof Uint8Array) {
      bodyStr = _toBinaryString(body);
    } else {
      bodyStr = String(body);
    }
  }

  var requestJson = JSON.stringify({ method: method, headers: headers, body: bodyStr });
  var result = __koss_fetch(url, requestJson);
  if (result && typeof result === 'string') {
    try {
      var parsed = JSON.parse(result);
      return new globalThis.Response(parsed.body || '', {
        status: parsed.status,
        statusText: parsed.status_text || '',
        headers: parsed.headers || {},
        url: url,
      });
    } catch (e) {
      return new globalThis.Response(result, {
        status: 200,
        headers: { 'content-type': 'text/plain' },
        url: url,
      });
    }
  }
  throw new Error('HTTP fetch failed');
}

module.exports = {
  tcpConnect: tcpConnect,
  tcpListen: tcpListen,
  dnsLookup: dnsLookup,
  httpFetch: httpFetch,
};
