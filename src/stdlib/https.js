'use strict';

var http = require('http');
var net = require('net');

function createServer(opts, requestListener) {
  if (typeof opts === 'function') { requestListener = opts; opts = {}; }
  return http.createServer(function(req, res) {
    if (requestListener) requestListener(req, res);
  });
}

function request(opts, cb) {
  var port = opts.port || 443;
  var host = opts.hostname || opts.host || '127.0.0.1';
  var path = opts.path || '/';
  var method = opts.method || 'GET';
  var headers = opts.headers || {};

  var socket = net.connect(port, host, function() {
    var reqLine = method + ' ' + path + ' HTTP/1.1\r\n';
    reqLine += 'Host: ' + host + '\r\n';
    for (var k in headers) reqLine += k + ': ' + headers[k] + '\r\n';
    reqLine += '\r\n';
    socket.write(reqLine);
  });

  var res = new http.IncomingMessage(socket);
  var buf = '';
  socket.on('data', function(data) {
    buf += data;
    if (!res._parsed && res._parse(buf)) {
      buf = res._body;
      res.emit('response');
      if (cb) cb(res);
    }
  });

  return socket;
}

function get(opts, cb) {
  if (typeof opts === 'string') {
    var url = opts;
    opts = {};
    var parts = url.replace('https://', '').split('/');
    opts.hostname = parts[0].split(':')[0];
    opts.port = parseInt(parts[0].split(':')[1]) || 443;
    opts.path = '/' + parts.slice(1).join('/');
  }
  return request(opts, cb);
}

module.exports = {
  ...http,
  createServer: createServer,
  request: request,
  get: get,
};
