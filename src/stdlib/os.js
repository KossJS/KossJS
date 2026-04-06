'use strict';

const {
  ArrayPrototypePush,
  Float64Array,
  ObjectDefineProperties,
  ObjectFreeze,
} = globalThis;

const isWindows = typeof process !== 'undefined' && process.platform === 'win32';

function getCPUs() {
  return [
    'Intel(R) Core(TM) i5-10500 CPU @ 3.10GHz', 2400,
    0, 0, 0, 0, 0,
    'Intel(R) Core(TM) i5-10500 CPU @ 3.10GHz', 2400,
    0, 0, 0, 0, 0,
  ];
}

function getFreeMem() {
  return 8 * 1024 * 1024 * 1024;
}

function getTotalMem() {
  return 16 * 1024 * 1024 * 1024;
}

function getHomeDirectory() {
  if (isWindows) {
    return 'C:\\Users\\User';
  }
  return '/home/user';
}

function getHostname() {
  return 'kossjs';
}

function getInterfaceAddresses() {
  return undefined;
}

function getLoadAvg(array) {
  if (array && array.length >= 3) {
    array[0] = 0;
    array[1] = 0;
    array[2] = 0;
  }
  return [0, 0, 0];
}

function getUptime() {
  return 0;
}

function getOSInformation() {
  return ['Windows', '10', '10.0.19044', 'x64'];
}

function isBigEndian() {
  return false;
}

function getTempDir() {
  if (isWindows) {
    return 'C:\\Temp';
  }
  return '/tmp';
}

function getUserInfo(options, ctx) {
  return {
    uid: 1000,
    gid: 1000,
    username: 'user',
    homedir: getHomeDirectory(),
    shell: isWindows ? null : '/bin/bash',
  };
}

function getAvailableParallelism() {
  return 4;
}

const arch = () => (typeof process !== 'undefined' && process.arch) || 'x64';
const platform = () => (typeof process !== 'undefined' && process.platform) || (isWindows ? 'win32' : 'linux');

const constants = {
  signals: {
    SIGHUP: 1,
    SIGINT: 2,
    SIGQUIT: 3,
    SIGILL: 4,
    SIGTRAP: 5,
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
    SIGSTKFLT: 16,
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
    SIGSYS: 31,
  },
  os: {
    UV_UDP_REUSEADDR: 4,
    EAFNOSUPPORT: -4095,
    EADDRINUSE: -4094,
    EADDRNOTAVAIL: -4093,
    EACCES: -4092,
    EAGAIN: -4091,
    EALREADY: -4090,
    EBADF: -4089,
    EBUSY: -4088,
    ECONNREFUSED: -4087,
    ECONNRESET: -4086,
    EDESTADDRREQ: -4085,
    EDOM: -4084,
    EDQUOT: -4083,
    EEXIST: -4082,
    EFAULT: -4081,
    EHOSTUNREACH: -4080,
    EINTR: -4079,
    EINVAL: -4078,
    EIO: -4077,
    EISCONN: -4076,
    ELOOP: -4075,
    EMFILE: -4074,
    EMSGSIZE: -4073,
    ENAMETOOLONG: -4072,
    ENETDOWN: -4071,
    ENETUNREACH: -4070,
    ENFILE: -4069,
    ENOBUFS: -4068,
    ENODEV: -4067,
    ENOENT: -4066,
    ENOMEM: -4065,
    ENONET: -4064,
    ENOPROTOOPT: -4063,
    ENOTCONN: -4062,
    ENOTDIR: -4061,
    ENOTEMPTY: -4060,
    ENOTSOCK: -4059,
    ENOTSUP: -4058,
    EOVERFLOW: -4057,
    EPERM: -4055,
    EPIPE: -4054,
    EPROTO: -4053,
    EPROTONOSUPPORT: -4052,
    EPROTOTYPE: -4051,
    ERANGE: -4050,
    EROFS: -4049,
    ESPIPE: -4048,
    ESRCH: -4047,
    ETIMEDOUT: -4046,
    ETXTBSY: -4045,
    EEXIST: -4082,
    EISDIR: -4065,
    ENOTDIR: -4061,
    EINVAL: -4078,
    ENFILE: -4069,
    EMFILE: -4074,
    ENOSPC: -4044,
  },
  crypto: {
    OPENSSL_VERSION_NUMBER: 0x30000000,
    SSL_OP_ALL: 0x80000,
    SSL_OP_NO_SSLv2: 0x0,
    SSL_OP_NO_SSLv3: 0x200000,
    SSL_OP_NO_TLSv1: 0x400000,
    SSL_OP_NO_TLSv1_2: 0x800000,
    SSL_OP_NO_TLSv1_3: 0x1000000,
  },
  fs: {
    F_OK: 0,
    R_OK: 4,
    W_OK: 2,
    X_OK: 1,
    S_IFMT: 0xF000,
    S_IFREG: 0x8000,
    S_IFDIR: 0x4000,
    S_IFCHR: 0x2000,
    S_IFBLK: 0x6000,
    S_IFLNK: 0xA000,
    S_IFSOCK: 0xC000,
    S_IFIFO: 0x1000,
    O_RDONLY: 0,
    O_WRONLY: 1,
    O_RDWR: 2,
    O_CREAT: 64,
    O_EXCL: 128,
    O_NOCTTY: 256,
    O_TRUNC: 512,
    O_APPEND: 1024,
    O_SYNC: 4096,
    O_DSYNC: 4096,
    O_NOFOLLOW: 4096,
    O_SYMLINK: 4096,
    O_DIRECT: 16384,
    O_DIRECTORY: 65536,
    O_CLOEXEC: 524288,
  },
};

function cpus() {
  const data = getCPUs() || [];
  const result = [];
  let i = 0;
  while (i < data.length) {
    ArrayPrototypePush(result, {
      model: data[i++],
      speed: data[i++],
      times: {
        user: data[i++],
        nice: data[i++],
        sys: data[i++],
        idle: data[i++],
        irq: data[i++],
      },
    });
  }
  return result;
}

function tmpdir() {
  return getTempDir();
}

function endianness() {
  return isBigEndian() ? 'BE' : 'LE';
}

function networkInterfaces() {
  return {};
}

function userInfo(options) {
  return getUserInfo(options, {});
}

function loadavg() {
  return [0, 0, 0];
}

module.exports = {
  arch,
  availableParallelism: getAvailableParallelism,
  cpus,
  endianness,
  freemem: getFreeMem,
  homedir: getHomeDirectory,
  hostname: getHostname,
  loadavg,
  networkInterfaces,
  platform,
  release: () => '10.0.19044',
  tmpdir,
  totalmem: getTotalMem,
  type: () => isWindows ? 'Windows' : 'Linux',
  userInfo,
  uptime: getUptime,
  version: () => '10.0.19044',
  machine: () => 'x64',
};

ObjectFreeze(constants.signals);

ObjectDefineProperties(module.exports, {
  constants: {
    __proto__: null,
    configurable: false,
    enumerable: true,
    value: constants,
  },

  EOL: {
    __proto__: null,
    configurable: true,
    enumerable: true,
    writable: false,
    value: isWindows ? '\r\n' : '\n',
  },

  devNull: {
    __proto__: null,
    configurable: true,
    enumerable: true,
    writable: false,
    value: isWindows ? '\\\\.\\nul' : '/dev/null',
  },
});