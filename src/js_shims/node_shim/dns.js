// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/dns - Node.js dns module (L3)
// Maps to koss:io standard library

var io = require('koss:io');

function isIP(input) {
  if (typeof input !== 'string') return 0;
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(input)) {
    var parts = input.split('.').map(Number);
    if (parts.every(function(p) { return p >= 0 && p <= 255; })) return 4;
  }
  if (/^[0-9a-fA-F:]+$/.test(input)) {
    var count = input.split(':').length;
    if (count >= 3 && count <= 8) return 6;
  }
  return 0;
}

function isIPv4(input) { return isIP(input) === 4; }
function isIPv6(input) { return isIP(input) === 6; }

function lookup(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (typeof options === 'number') options = { family: options };
  var opts = options || {};
  var cb = typeof callback === 'function' ? callback : function() {};

  try {
    var ips = io.dns(String(hostname));
    if (!Array.isArray(ips) || ips.length === 0) {
      cb(new Error('ENOTFOUND ' + hostname), null, null);
      return;
    }
    if (opts.all) {
      var results = ips.map(function(ip) { return { address: ip, family: isIP(ip) }; });
      cb(null, results);
    } else {
      cb(null, ips[0], isIP(ips[0]));
    }
  } catch (err) {
    cb(err, null, null);
  }
}

function resolve(hostname, rrtype, callback) {
  if (typeof rrtype === 'function') { callback = rrtype; rrtype = 'A'; }
  try {
    var ips = io.dns(String(hostname));
    if (callback) callback(null, ips);
  } catch (err) {
    if (callback) callback(err);
  }
}

function resolve4(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  resolve(hostname, 'A', callback);
}

function resolve6(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  try {
    var ips = io.dns(String(hostname));
    var v6 = ips.filter(function(ip) { return isIP(ip) === 6; });
    if (callback) callback(null, v6.length > 0 ? v6 : ips);
  } catch (err) {
    if (callback) callback(err);
  }
}

function lookupService(address, port, callback) {
  if (callback) {
    var nextTick = (typeof process !== 'undefined' && process.nextTick) ? process.nextTick : setTimeout;
    nextTick(function() { callback(new Error('lookupService not implemented'), null, null); });
  }
}

var promises = {
  lookup: function(hostname, options) {
    return new Promise(function(res, rej) {
      lookup(hostname, options, function(err, addr, family) {
        if (err) return rej(err);
        if (options && options.all) return res(addr);
        res({ address: addr, family: family });
      });
    });
  },
  resolve: function(hostname, rrtype) {
    return new Promise(function(res, rej) {
      resolve(hostname, rrtype, function(err, addrs) {
        if (err) return rej(err);
        res(addrs);
      });
    });
  },
  resolve4: function(hostname, options) {
    return new Promise(function(res, rej) {
      resolve4(hostname, options, function(err, addrs) {
        if (err) return rej(err);
        res(addrs);
      });
    });
  },
  resolve6: function(hostname, options) {
    return new Promise(function(res, rej) {
      resolve6(hostname, options, function(err, addrs) {
        if (err) return rej(err);
        res(addrs);
      });
    });
  },
};

module.exports = { lookup: lookup, resolve: resolve, resolve4: resolve4, resolve6: resolve6, lookupService: lookupService, isIP: isIP, isIPv4: isIPv4, isIPv6: isIPv6, promises: promises };
