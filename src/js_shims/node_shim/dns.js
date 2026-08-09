// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/dns - Node.js dns module (L3)
// Maps to koss:io standard library

var io = require('koss:io');

function isIP(input) {
  if (typeof input !== 'string') return 0;
  // IPv4
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(input)) {
    var parts = input.split('.').map(Number);
    if (parts.every(function(p) { return p >= 0 && p <= 255; })) return 4;
  }
  // IPv6：标准校验
  if (/^[0-9a-fA-F:]+$/.test(input) && input.indexOf(':') !== -1) {
    var count = input.split(':').length;
    if (input.indexOf('::') !== -1) {
      // 含 :: 压缩形式，最多 8 段（含空）
      if (count <= 8) return 6;
    } else {
      if (count === 8) return 6;
    }
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
    // family 过滤
    var filtered = ips;
    if (opts.family === 4) filtered = ips.filter(function(ip) { return isIP(ip) === 4; });
    else if (opts.family === 6) filtered = ips.filter(function(ip) { return isIP(ip) === 6; });
    if (filtered.length === 0) {
      cb(new Error('ENOTFOUND ' + hostname), null, null);
      return;
    }
    if (opts.all) {
      var results = filtered.map(function(ip) { return { address: ip, family: isIP(ip) }; });
      cb(null, results);
    } else {
      cb(null, filtered[0], isIP(filtered[0]));
    }
  } catch (err) {
    cb(err, null, null);
  }
}

function resolve(hostname, rrtype, callback) {
  if (typeof rrtype === 'function') { callback = rrtype; rrtype = 'A'; }
  var cb = typeof callback === 'function' ? callback : function() {};
  var type = String(rrtype || 'A').toUpperCase();
  try {
    var ips = io.dns(String(hostname));
    if (type === 'AAAA') {
      cb(null, ips.filter(function(ip) { return isIP(ip) === 6; }));
    } else if (type === 'A') {
      cb(null, ips.filter(function(ip) { return isIP(ip) === 4; }));
    } else {
      // MX/TXT/NS/CNAME 等暂不支持，返回原记录（保持兼容）
      cb(null, ips);
    }
  } catch (err) {
    cb(err);
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
    if (callback) callback(null, v6);
  } catch (err) {
    if (callback) callback(err);
  }
}

function lookupService(address, port, callback) {
  var cb = typeof callback === 'function' ? callback : function() {};
  var nextTick = (typeof process !== 'undefined' && process.nextTick) ? process.nextTick : setTimeout;
  if (typeof globalThis.__koss_dns_lookup_service === 'function') {
    nextTick(function() {
      try {
        var host = globalThis.__koss_dns_lookup_service(String(address));
        if (host) cb(null, host, port || 0);
        else cb(new Error('ENOTFOUND ' + address), null, null);
      } catch (err) {
        cb(err, null, null);
      }
    });
  } else {
    nextTick(function() { cb(new Error('lookupService not available (NET_DNS not granted)'), null, null); });
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
  lookupService: function(address, port) {
    return new Promise(function(res, rej) {
      lookupService(address, port, function(err, hostname, service) {
        if (err) return rej(err);
        res({ hostname: hostname, service: service });
      });
    });
  },
};

module.exports = { lookup: lookup, resolve: resolve, resolve4: resolve4, resolve6: resolve6, lookupService: lookupService, isIP: isIP, isIPv4: isIPv4, isIPv6: isIPv6, promises: promises };
