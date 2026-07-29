// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:os — Koss 操作系统信息模块
// 主机名、平台、架构、内存、CPU、网络接口等

var kossSystem = require('koss:system');

var process = globalThis.process;

function homedir() {
  return kossSystem.homedir();
}

function hostname() {
  return kossSystem.hostname();
}

function platform() {
  return kossSystem.platform();
}

function arch() {
  return kossSystem.arch();
}

function type() {
  return kossSystem.type();
}

function release() {
  return kossSystem.release();
}

function totalmem() {
  var mem = kossSystem.memory();
  return mem.total || 0;
}

function freemem() {
  var mem = kossSystem.memory();
  return mem.free || 0;
}

function cpus() {
  return kossSystem.cpus();
}

function uptime() {
  return kossSystem.uptime();
}

function loadavg() {
  return kossSystem.loadavg();
}

function networkInterfaces() {
  var result = {};
  if (process && typeof process.getNetworkInterfaces === 'function') {
    var interfaces = process.getNetworkInterfaces();
    for (var name in interfaces) {
      result[name] = interfaces[name];
    }
    return result;
  }
  if (process && process.binding) {
    try {
      var binding = process.binding('net');
      if (binding && typeof binding.getNetworkInterfaces === 'function') {
        var ifaces = binding.getNetworkInterfaces();
        for (var key in ifaces) {
          result[key] = ifaces[key];
        }
        return result;
      }
    } catch (e) { /* ignore */ }
  }
  return result;
}

function userInfo(options) {
  return kossSystem.userInfo(options);
}

function tmpdir() {
  return kossSystem.tmpdir();
}

function endianness() {
  var buffer = new ArrayBuffer(2);
  var view = new Uint8Array(buffer);
  var shortView = new Uint16Array(buffer);
  shortView[0] = 0x0102;
  if (view[0] === 0x02) return 'LE';
  if (view[0] === 0x01) return 'BE';
  return 'LE';
}

var EOL = kossSystem.EOL;

function availableParallelism() {
  return kossSystem.availableParallelism();
}

module.exports = {
  homedir: homedir,
  hostname: hostname,
  platform: platform,
  arch: arch,
  type: type,
  release: release,
  totalmem: totalmem,
  freemem: freemem,
  cpus: cpus,
  uptime: uptime,
  loadavg: loadavg,
  networkInterfaces: networkInterfaces,
  userInfo: userInfo,
  tmpdir: tmpdir,
  endianness: endianness,
  EOL: EOL,
  availableParallelism: availableParallelism,
};
