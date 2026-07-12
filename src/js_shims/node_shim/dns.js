// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/dns - Node.js dns module (L3)

const internalNet = require('koss:internal/net');

function isIP(input) {
  if (typeof input !== 'string') return 0;
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(input)) {
    const parts = input.split('.').map(Number);
    if (parts.every(p => p >= 0 && p <= 255)) return 4;
  }
  if (/^[0-9a-fA-F:]+$/.test(input)) {
    const count = input.split(':').length;
    if (count >= 3 && count <= 8) return 6;
  }
  return 0;
}

function isIPv4(input) { return isIP(input) === 4; }
function isIPv6(input) { return isIP(input) === 6; }

function lookup(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (typeof options === 'number') options = { family: options };
  const opts = options || {};
  const cb = typeof callback === 'function' ? callback : () => {};

  try {
    const ips = internalNet.dnsLookup(String(hostname));
    if (!Array.isArray(ips) || ips.length === 0) {
      cb(new Error(`ENOTFOUND ${hostname}`), null, null);
      return;
    }
    if (opts.all) {
      const results = ips.map(ip => ({ address: ip, family: isIP(ip) }));
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
    const ips = internalNet.dnsLookup(String(hostname));
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
    const ips = internalNet.dnsLookup(String(hostname));
    const v6 = ips.filter(ip => isIP(ip) === 6);
    if (callback) callback(null, v6.length > 0 ? v6 : ips);
  } catch (err) {
    if (callback) callback(err);
  }
}

function lookupService(address, port, callback) {
  if (callback) process.nextTick(() => callback(new Error('lookupService not implemented'), null, null));
}

const promises = {
  lookup: (hostname, options) => new Promise((resolve, reject) => lookup(hostname, options, (err, addr, family) => err ? reject(err) : resolve(options?.all ? addr : { address: addr, family }))),
  resolve: (hostname, rrtype) => new Promise((resolve, reject) => resolve(hostname, rrtype, (err, addrs) => err ? reject(err) : resolve(addrs))),
  resolve4: (hostname, options) => promises.resolve(hostname, 'A'),
  resolve6: (hostname, options) => promises.resolve(hostname, 'AAAA'),
};

module.exports = { lookup, resolve, resolve4, resolve6, lookupService, isIP, isIPv4, isIPv6, promises };