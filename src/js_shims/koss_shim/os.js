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

function version() {
  return process && process.version ? process.version : 'unknown';
}

function machine() {
  return process && process.arch ? process.arch : 'unknown';
}

var constants = {
  signals: {
    SIGHUP: 1,
    SIGINT: 2,
    SIGQUIT: 3,
    SIGILL: 4,
    SIGTRAP: 5,
    SIGABRT: 6,
    SIGIOT: 6,
    SIGBUS: 7,
    SIGFPE: 8,
    SIGKILL: 9,
    SIGUSR1: 10,
    SIGSEGV: 11,
    SIGUSR2: 12,
    SIGPIPE: 13,
    SIGALRM: 14,
    SIGTERM: 15,
    SIGCHLD: 17,
    SIGCONT: 18,
    SIGSTOP: 19,
    SIGTSTP: 20,
    SIGTTIN: 21,
    SIGTTOU: 22,
    SIGURG: 23,
    SIGXCPU: 24,
    SIGXFSZ: 25,
    SIGVTALRM: 26,
    SIGPROF: 27,
    SIGWINCH: 28,
    SIGIO: 29,
    SIGPWR: 30,
    SIGSYS: 31,
  },
  errno: {
    EPERM: 1,
    ENOENT: 2,
    ESRCH: 3,
    EINTR: 4,
    EIO: 5,
    ENXIO: 6,
    E2BIG: 7,
    ENOEXEC: 8,
    EBADF: 9,
    ECHILD: 10,
    EAGAIN: 11,
    ENOMEM: 12,
    EACCES: 13,
    EFAULT: 14,
    ENOTBLK: 15,
    EBUSY: 16,
    EEXIST: 17,
    EXDEV: 18,
    ENODEV: 19,
    ENOTDIR: 20,
    EISDIR: 21,
    EINVAL: 22,
    ENFILE: 23,
    EMFILE: 24,
    ENOTTY: 25,
    ETXTBSY: 26,
    EFBIG: 27,
    ENOSPC: 28,
    ESPIPE: 29,
    EROFS: 30,
    EMLINK: 31,
    EPIPE: 32,
    EDOM: 33,
    ERANGE: 34,
    ENOSYS: 38,
    ENOTSOCK: 39,
    EDESTADDRREQ: 40,
    EMSGSIZE: 41,
    EPROTOTYPE: 42,
    ENOPROTOOPT: 43,
    EPROTONOSUPPORT: 44,
    ENOTSUP: 45,
    EAFNOSUPPORT: 47,
    EADDRINUSE: 48,
    EADDRNOTAVAIL: 49,
    ENETDOWN: 50,
    ENETUNREACH: 51,
    ECONNABORTED: 53,
    ECONNRESET: 54,
    ENOBUFS: 55,
    EISCONN: 56,
    ENOTCONN: 57,
    ETIMEDOUT: 60,
    ECONNREFUSED: 61,
    EHOSTUNREACH: 65,
    EALREADY: 103,
    EINPROGRESS: 115,
  },
};

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
  version: version,
  machine: machine,
  constants: constants,
  devNull: platform() === 'win32' ? 'nul' : '/dev/null',
};
