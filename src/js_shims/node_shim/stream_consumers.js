// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/stream/consumers - Node.js stream/consumers module (L3)

function _collect(stream) {
  return new Promise(function(resolve, reject) {
    var chunks = [];
    if (stream && typeof stream.on === 'function') {
      stream.on('data', function(chunk) { chunks.push(chunk); });
      stream.on('end', function() { resolve(chunks); });
      stream.on('error', reject);
      if (typeof stream.resume === 'function') stream.resume();
    } else {
      resolve(chunks);
    }
  });
}

function json(stream) {
  return _collect(stream).then(function(chunks) {
    var text = _chunksToString(chunks);
    return JSON.parse(text);
  });
}

function text(stream) {
  return _collect(stream).then(function(chunks) {
    return _chunksToString(chunks);
  });
}

function buffer(stream) {
  return _collect(stream).then(function(chunks) {
    var total = 0;
    for (var i = 0; i < chunks.length; i++) total += chunks[i].length;
    var buf = Buffer.alloc(total);
    var offset = 0;
    for (var j = 0; j < chunks.length; j++) {
      var c = chunks[j];
      var bytes = c._data || c;
      for (var k = 0; k < bytes.length; k++) buf._data[offset + k] = bytes[k];
      offset += bytes.length;
    }
    buf._length = total;
    return buf;
  });
}

function arrayBuffer(stream) {
  return buffer(stream).then(function(buf) {
    return buf._data.buffer;
  });
}

function blob(stream, mimeType) {
  return buffer(stream).then(function(buf) {
    return new Blob([buf._data], { type: mimeType || '' });
  });
}

function _chunksToString(chunks) {
  var parts = [];
  for (var i = 0; i < chunks.length; i++) {
    var c = chunks[i];
    if (typeof c === 'string') {
      parts.push(c);
    } else {
      var bytes = c._data || c;
      if (bytes && typeof Buffer !== 'undefined' && Buffer.prototype && typeof Buffer.prototype.toString === 'function') {
        var b = Buffer.from(bytes);
        parts.push(b.toString('utf8'));
      } else {
        parts.push(String(c));
      }
    }
  }
  return parts.join('');
}

module.exports = {
  json: json,
  text: text,
  buffer: buffer,
  arrayBuffer: arrayBuffer,
  blob: blob,
};
