// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:constants — Koss 标准库常量模块
// Node.js constants 兼容实现，纯 JS，无外部依赖

// ═══════════════════════════════════════════
// 文件打开标志 (O_*)
// ═══════════════════════════════════════════

var O_RDONLY    = 0;
var O_WRONLY    = 1;
var O_RDWR      = 2;
var O_CREAT     = 64;
var O_EXCL      = 128;
var O_NOCTTY    = 256;
var O_TRUNC     = 512;
var O_APPEND    = 1024;
var O_DIRECTORY = 65536;
var O_NOATIME   = 262144;
var O_NOFOLLOW  = 131072;
var O_SYNC      = 1052672;
var O_DSYNC     = 1052672;
var O_SYMLINK   = 1048576;
var O_DIRECT    = 16384;
var O_NONBLOCK  = 2048;

// ═══════════════════════════════════════════
// 文件权限 (S_I*)
// ═══════════════════════════════════════════

var S_IRWXU = 448;
var S_IRUSR = 256;
var S_IWUSR = 128;
var S_IXUSR = 64;
var S_IRWXG = 56;
var S_IRGRP = 32;
var S_IWGRP = 16;
var S_IXGRP = 8;
var S_IRWXO = 7;
var S_IROTH = 4;
var S_IWOTH = 2;
var S_IXOTH = 1;

// ═══════════════════════════════════════════
// 进程优先级 (PRIORITY_*)
// ═══════════════════════════════════════════

var PRIORITY_LOW      = 19;
var PRIORITY_NORMAL   = 0;
var PRIORITY_HIGH     = -14;
var PRIORITY_HIGHEST  = -20;

// ═══════════════════════════════════════════
// 文件类型掩码 (S_IF*)
// ═══════════════════════════════════════════

var S_IFMT   = 61440;
var S_IFREG  = 32768;
var S_IFDIR  = 16384;
var S_IFCHR  = 8192;
var S_IFBLK  = 24576;
var S_IFIFO  = 4096;
var S_IFLNK  = 40960;
var S_IFSOCK = 49152;

module.exports = {
  O_RDONLY: O_RDONLY,
  O_WRONLY: O_WRONLY,
  O_RDWR: O_RDWR,
  O_CREAT: O_CREAT,
  O_EXCL: O_EXCL,
  O_NOCTTY: O_NOCTTY,
  O_TRUNC: O_TRUNC,
  O_APPEND: O_APPEND,
  O_DIRECTORY: O_DIRECTORY,
  O_NOATIME: O_NOATIME,
  O_NOFOLLOW: O_NOFOLLOW,
  O_SYNC: O_SYNC,
  O_DSYNC: O_DSYNC,
  O_SYMLINK: O_SYMLINK,
  O_DIRECT: O_DIRECT,
  O_NONBLOCK: O_NONBLOCK,
  S_IRWXU: S_IRWXU,
  S_IRUSR: S_IRUSR,
  S_IWUSR: S_IWUSR,
  S_IXUSR: S_IXUSR,
  S_IRWXG: S_IRWXG,
  S_IRGRP: S_IRGRP,
  S_IWGRP: S_IWGRP,
  S_IXGRP: S_IXGRP,
  S_IRWXO: S_IRWXO,
  S_IROTH: S_IROTH,
  S_IWOTH: S_IWOTH,
  S_IXOTH: S_IXOTH,
  PRIORITY_LOW: PRIORITY_LOW,
  PRIORITY_NORMAL: PRIORITY_NORMAL,
  PRIORITY_HIGH: PRIORITY_HIGH,
  PRIORITY_HIGHEST: PRIORITY_HIGHEST,
  S_IFMT: S_IFMT,
  S_IFREG: S_IFREG,
  S_IFDIR: S_IFDIR,
  S_IFCHR: S_IFCHR,
  S_IFBLK: S_IFBLK,
  S_IFIFO: S_IFIFO,
  S_IFLNK: S_IFLNK,
  S_IFSOCK: S_IFSOCK,
  fs: {
    O_RDONLY: O_RDONLY,
    O_WRONLY: O_WRONLY,
    O_RDWR: O_RDWR,
    O_CREAT: O_CREAT,
    O_EXCL: O_EXCL,
    O_NOCTTY: O_NOCTTY,
    O_TRUNC: O_TRUNC,
    O_APPEND: O_APPEND,
    O_DIRECTORY: O_DIRECTORY,
    O_NOATIME: O_NOATIME,
    O_NOFOLLOW: O_NOFOLLOW,
    O_SYNC: O_SYNC,
    O_DSYNC: O_DSYNC,
    O_SYMLINK: O_SYMLINK,
    O_DIRECT: O_DIRECT,
    O_NONBLOCK: O_NONBLOCK,
    S_IRWXU: S_IRWXU,
    S_IRUSR: S_IRUSR,
    S_IWUSR: S_IWUSR,
    S_IXUSR: S_IXUSR,
    S_IRWXG: S_IRWXG,
    S_IRGRP: S_IRGRP,
    S_IWGRP: S_IWGRP,
    S_IXGRP: S_IXGRP,
    S_IRWXO: S_IRWXO,
    S_IROTH: S_IROTH,
    S_IWOTH: S_IWOTH,
    S_IXOTH: S_IXOTH,
    S_IFMT: S_IFMT,
    S_IFREG: S_IFREG,
    S_IFDIR: S_IFDIR,
    S_IFCHR: S_IFCHR,
    S_IFBLK: S_IFBLK,
    S_IFIFO: S_IFIFO,
    S_IFLNK: S_IFLNK,
    S_IFSOCK: S_IFSOCK,
  },
  os: {
    EOL: process && process.platform === 'win32' ? '\r\n' : '\n',
  },
  crypto: {
    BADFLAGS: -1,
    BADHASH: -2,
    UNKNOWN: -3,
    FAILED: -4,
  },
};
