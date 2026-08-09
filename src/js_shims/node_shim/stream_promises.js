// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/stream/promises - Node.js stream/promises module (L3)

var kossStream = require('koss:stream');

function pipeline() {
  var args = Array.prototype.slice.call(arguments);
  var callback = typeof args[args.length - 1] === 'function' ? args.pop() : null;
  return new Promise(function(resolve, reject) {
    try {
      var result = kossStream.pipeline.apply(null, args);
      if (callback) callback(null);
      resolve(result);
    } catch (err) {
      if (callback) callback(err);
      reject(err);
    }
  });
}

function finished(stream, options) {
  return new Promise(function(resolve, reject) {
    if (typeof kossStream.finished === 'function') {
      try {
        kossStream.finished(stream, options || {}, function(err) {
          if (err) reject(err); else resolve();
        });
      } catch (err) {
        reject(err);
      }
    } else {
      // 降级：检查流已结束
      var state = stream && (stream._readableState || stream._writableState);
      if (state && state.ended) resolve();
      else reject(new Error('stream not finished'));
    }
  });
}

function rejectWithError(promise, err) {
  return promise.catch(function() { throw err; });
}

module.exports = {
  pipeline: pipeline,
  finished: finished,
  rejectWithError: rejectWithError,
};
