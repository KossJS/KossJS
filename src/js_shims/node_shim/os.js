// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/os - Node.js os module (L3)

const { __koss_bindings, process } = globalThis;

function getOSInfo() {
  try {
    const binding = typeof __koss_bindings === 'function' ? __koss_bindings('os') : '{}';
    const info = typeof binding === 'string' ? JSON.parse(binding) : binding;
    return info;
  } catch { return {}; }
}

function hostname() {
  try {
    const info = getOSInfo();
    if (info.getHostname) return 'kossjs';
  } catch {}
  return 'kossjs';
}

function platform() {
  return process?.platform || 'unknown';
}

function type() {
  const p = platform();
  if (p === 'win32') return 'Windows_NT';
  if (p === 'darwin') return 'Darwin';
  if (p === 'linux') return 'Linux';
  return 'Unknown';
}

function release() {
  return '1.0.0';
}

function arch() {
  return process?.arch || 'x64';
}

function tmpdir() {
  if (platform() === 'win32') return process?.env?.TEMP || 'C:\\Temp';
  return '/tmp';
}

function homedir() {
  return process?.env?.HOME || process?.env?.USERPROFILE || '/home/user';
}

function endianness() {
  return 'LE';
}

function freemem() {
  return 8 * 1024 * 1024 * 1024;
}

function totalmem() {
  return 16 * 1024 * 1024 * 1024;
}

function uptime() {
  return 0;
}

function loadavg() {
  return [0, 0, 0];
}

function cpus() {
  return [{ model: 'Generic CPU', speed: 2400, times: { user: 0, nice: 0, sys: 0, idle: 0, irq: 0 } }];
}

function networkInterfaces() {
  return {};
}

function userInfo(options) {
  const encoding = options?.encoding || 'utf8';
  return {
    uid: 1000,
    gid: 1000,
    username: 'user',
    homedir: homedir(),
    shell: platform() === 'win32' ? null : '/bin/bash',
  };
}

function EOL() {
  return platform() === 'win32' ? '\r\n' : '\n';
}

function version() {
  return process?.version || '0.0.0';
}

function machine() {
  return arch();
}

function availableParallelism() {
  return 1;
}

const signals = {
  SIGHUP: 1, SIGINT: 2, SIGQUIT: 3, SIGILL: 4, SIGTRAP: 5, SIGABRT: 6,
  SIGBUS: 7, SIGFPE: 8, SIGKILL: 9, SIGUSR1: 10, SIGSEGV: 11, SIGUSR2: 12,
  SIGPIPE: 13, SIGALRM: 14, SIGTERM: 15, SIGCHLD: 17, SIGCONT: 19, SIGSTOP: 17,
  SIGTSTP: 20, SIGTTIN: 21, SIGTTOU: 22, SIGURG: 23, SIGXCPU: 24, SIGXFSZ: 25,
  SIGVTALRM: 26, SIGPROF: 27, SIGWINCH: 28, SIGIO: 29, SIGSYS: 31,
};

const errno = {
  E2BIG: 7, EACCES: 13, EADDRINUSE: 98, EADDRNOTAVAIL: 99, EAFNOSUPPORT: 97,
  EAGAIN: 11, EALREADY: 114, EBADF: 9, EBADMSG: 74, EBUSY: 16, ECANCELED: 125,
  ECHILD: 10, ECONNABORTED: 103, ECONNREFUSED: 111, ECONNRESET: 104,
  EDEADLK: 35, EDESTADDRREQ: 89, EDOM: 33, EDQUOT: 122, EEXIST: 17,
  EFAULT: 14, EFBIG: 27, EHOSTUNREACH: 113, EIDRM: 43, EILSEQ: 84,
  EINPROGRESS: 115, EINTR: 4, EINVAL: 22, EIO: 5, EISCONN: 106,
  EISDIR: 21, ELOOP: 40, EMFILE: 24, EMLINK: 31, EMSGSIZE: 90,
  EMULTIHOP: 72, ENAMETOOLONG: 36, ENETDOWN: 100, ENETRESET: 102,
  ENETUNREACH: 101, ENFILE: 23, ENOBUFS: 105, ENODATA: 61, ENODEV: 19,
  ENOENT: 2, ENOEXEC: 8, ENOLCK: 37, ENOLINK: 67, ENOMEM: 12,
  ENOMSG: 42, ENOPROTOOPT: 92, ENOSPC: 28, ENOSR: 63, ENOSTR: 60,
  ENOSYS: 38, ENOTCONN: 107, ENOTDIR: 20, ENOTEMPTY: 39, ENOTRECOVERABLE: 131,
  ENOTSOCK: 88, ENOTSUP: 95, ENOTTY: 25, ENXIO: 6, EOPNOTSUPP: 95,
  EOVERFLOW: 75, EOWNERDEAD: 130, EPERM: 1, EPIPE: 32, EPROTO: 71,
  EPROTONOSUPPORT: 93, EPROTOTYPE: 91, ERANGE: 34, EROFS: 30, ESPIPE: 29,
  ESRCH: 3, ESTALE: 116, ETIME: 62, ETIMEDOUT: 110, ETXTBSY: 26,
  EWOULDBLOCK: 11, EXDEV: 18,
};

const constants = {
  UV_UDP_REUSEADDR: 4,
  signals,
  errno,
  dllPrefix: platform() === 'win32' ? '' : 'lib',
  dllSuffix: platform() === 'win32' ? '.dll' : platform() === 'darwin' ? '.dylib' : '.so',
  priority: { PRIORITY_LOW: 19, PRIORITY_BELOW_NORMAL: 10, PRIORITY_NORMAL: 0, PRIORITY_ABOVE_NORMAL: -7, PRIORITY_HIGH: -14, PRIORITY_HIGHEST: -20 },
};

const devNull = platform() === 'win32' ? '\\\\.\\nul' : '/dev/null';

module.exports = { hostname, platform, type, release, arch, tmpdir, homedir, endianness, freemem, totalmem, uptime, loadavg, cpus, networkInterfaces, userInfo, EOL: EOL(), version, machine, constants, availableParallelism, devNull };
