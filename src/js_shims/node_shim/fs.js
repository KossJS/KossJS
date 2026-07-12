// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/fs - Node.js fs module (L3)
// Wraps koss:internal/fs with Node.js-compatible API

const internalFs = require('koss:internal/fs');

const { Buffer } = globalThis;
const kMaxLength = 4294967296;

function getOptions(options, defaultEncoding) {
  if (options === null || options === undefined) return { encoding: defaultEncoding };
  if (typeof options === 'string') return { encoding: options };
  if (typeof options === 'object') return { encoding: options.encoding || defaultEncoding, ...options };
  return { encoding: defaultEncoding };
}

function handleError(err) { if (err) throw err; }

// === Synchronous API ===

function readFileSync(path, options) {
  const opts = getOptions(options, null);
  const data = internalFs.readFileSync(String(path));
  if (opts.encoding && Buffer.isBuffer(data)) {
    return data.toString(opts.encoding);
  }
  return data;
}

function writeFileSync(path, data, options) {
  const opts = getOptions(options, 'utf8');
  internalFs.writeFileSync(String(path), data);
}

function appendFileSync(path, data, options) {
  writeFileSync(path, data, options);
}

function existsSync(path) {
  try { return internalFs.existsSync(String(path)); }
  catch { return false; }
}

function statSync(path, options) {
  return internalFs.statSync(String(path));
}

function lstatSync(path, options) {
  return internalFs.statSync(String(path));
}

function mkdirSync(path, options) {
  const opts = typeof options === 'object' ? options : { recursive: Boolean(options) };
  internalFs.mkdirSync(String(path), opts);
}

function rmdirSync(path, options) {
  internalFs.rmdirSync(String(path));
}

function unlinkSync(path) {
  internalFs.unlinkSync(String(path));
}

function readdirSync(path, options) {
  const opts = getOptions(options, null);
  const entries = internalFs.readdirSync(String(path));
  return opts.withFileTypes ? entries : entries.map(e => typeof e === 'string' ? e : e[0]);
}

function renameSync(oldPath, newPath) {
  internalFs.renameSync(String(oldPath), String(newPath));
}

function realpathSync(path, options) {
  return internalFs.realpathSync(String(path));
}

function copyFileSync(src, dest, flags) {
  internalFs.copyFileSync(String(src), String(dest));
}

function chmodSync(path, mode) {
  internalFs.chmodSync(String(path), Number(mode) || 0);
}

function accessSync(path, mode) {
  if (!existsSync(path)) throw Object.assign(new Error('ENOENT'), { code: 'ENOENT', errno: -2, syscall: 'access', path: String(path) });
}

function mkdtempSync(prefix, options) {
  const dir = internalFs.realpathSync('.') + '/' + prefix + Date.now();
  internalFs.mkdirSync(dir);
  return dir;
}

function truncateSync(path, len) {
  if (len === 0) writeFileSync(path, '');
}

function fstatSync(fd) {
  return { dev: 0, mode: 33206, nlink: 1, uid: 0, gid: 0, rdev: 0, blksize: 4096, ino: 0, size: 0, blocks: 0, atimeMs: 0, mtimeMs: 0, ctimeMs: 0 };
}

// === Callback API ===

function readFile(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { var data = readFileSync(path, options); callback(null, data); }
  catch (err) { callback(err); }
}

function writeFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { writeFileSync(path, data, options); callback(null); }
  catch (err) { callback(err); }
}

function appendFile(path, data, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  writeFile(path, data, options, callback);
}

function exists(path, callback) {
  try { callback(existsSync(path)); }
  catch { callback(false); }
}

function stat(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { callback(null, statSync(path, options)); }
  catch (err) { callback(err); }
}

function lstat(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { callback(null, lstatSync(path, options)); }
  catch (err) { callback(err); }
}

function mkdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { mkdirSync(path, options); callback(null); }
  catch (err) { callback(err); }
}

function rmdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { rmdirSync(path); callback(null); }
  catch (err) { callback(err); }
}

function unlink(path, callback) {
  try { unlinkSync(path); callback(null); }
  catch (err) { callback(err); }
}

function readdir(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { callback(null, readdirSync(path, options)); }
  catch (err) { callback(err); }
}

function rename(oldPath, newPath, callback) {
  try { renameSync(oldPath, newPath); callback(null); }
  catch (err) { callback(err); }
}

function realpath(path, options, callback) {
  if (typeof options === 'function') { callback = options; options = undefined; }
  try { callback(null, realpathSync(path, options)); }
  catch (err) { callback(err); }
}

function copyFile(src, dest, flags, callback) {
  if (typeof flags === 'function') { callback = flags; flags = 0; }
  try { copyFileSync(src, dest, flags); callback(null); }
  catch (err) { callback(err); }
}

function access(path, mode, callback) {
  if (typeof mode === 'function') { callback = mode; mode = undefined; }
  try { accessSync(path, mode); callback(null); }
  catch (err) { callback(err); }
}

function chmod(path, mode, callback) {
  try { chmodSync(path, mode); callback(null); }
  catch (err) { callback(err); }
}

// === Promises API ===

const promises = {
  readFile: (path, options) => new Promise((resolve, reject) => readFile(path, options, (err, data) => err ? reject(err) : resolve(data))),
  writeFile: (path, data, options) => new Promise((resolve, reject) => writeFile(path, data, options, (err) => err ? reject(err) : resolve())),
  appendFile: (path, data, options) => new Promise((resolve, reject) => appendFile(path, data, options, (err) => err ? reject(err) : resolve())),
  stat: (path, options) => new Promise((resolve, reject) => stat(path, options, (err, s) => err ? reject(err) : resolve(s))),
  lstat: (path, options) => new Promise((resolve, reject) => lstat(path, options, (err, s) => err ? reject(err) : resolve(s))),
  mkdir: (path, options) => new Promise((resolve, reject) => mkdir(path, options, (err) => err ? reject(err) : resolve())),
  rmdir: (path, options) => new Promise((resolve, reject) => rmdir(path, options, (err) => err ? reject(err) : resolve())),
  unlink: (path) => new Promise((resolve, reject) => unlink(path, (err) => err ? reject(err) : resolve())),
  readdir: (path, options) => new Promise((resolve, reject) => readdir(path, options, (err, files) => err ? reject(err) : resolve(files))),
  rename: (oldPath, newPath) => new Promise((resolve, reject) => rename(oldPath, newPath, (err) => err ? reject(err) : resolve())),
  realpath: (path, options) => new Promise((resolve, reject) => realpath(path, options, (err, p) => err ? reject(err) : resolve(p))),
  copyFile: (src, dest, flags) => new Promise((resolve, reject) => copyFile(src, dest, flags, (err) => err ? reject(err) : resolve())),
  access: (path, mode) => new Promise((resolve, reject) => access(path, mode, (err) => err ? reject(err) : resolve())),
  chmod: (path, mode) => new Promise((resolve, reject) => chmod(path, mode, (err) => err ? reject(err) : resolve())),
  mkdtemp: (prefix, options) => new Promise((resolve, reject) => { try { resolve(mkdtempSync(prefix, options)); } catch (err) { reject(err); } }),
};

const constants = {
  F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1,
  O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2, O_CREAT: 64, O_EXCL: 128, O_TRUNC: 512, O_APPEND: 1024,
  S_IFMT: 61440, S_IFREG: 32768, S_IFDIR: 16384, S_IRWXU: 448, S_IRUSR: 256, S_IWUSR: 128, S_IXUSR: 64,
  S_IRWXG: 56, S_IRGRP: 32, S_IWGRP: 16, S_IXGRP: 8, S_IRWXO: 7, S_IROTH: 4, S_IWOTH: 2, S_IXOTH: 1,
  COPYFILE_EXCL: 1, COPYFILE_FICLONE: 2, COPYFILE_FICLONE_FORCE: 4,
};

function watch() { throw new Error('fs.watch is not implemented'); }
function watchFile() { throw new Error('fs.watchFile is not implemented'); }
function unwatchFile() { throw new Error('fs.unwatchFile is not implemented'); }
function createReadStream() { throw new Error('fs.createReadStream is not implemented'); }
function createWriteStream() { throw new Error('fs.createWriteStream is not implemented'); }
function openSync() { throw new Error('fs.openSync is not implemented'); }
function closeSync() { throw new Error('fs.closeSync is not implemented'); }
function readSync() { throw new Error('fs.readSync is not implemented'); }
function writeSync() { throw new Error('fs.writeSync is not implemented'); }
function ftruncateSync() { throw new Error('fs.ftruncateSync is not implemented'); }
function fsyncSync() { throw new Error('fs.fsyncSync is not implemented'); }

module.exports = { readFileSync, writeFileSync, appendFileSync, existsSync, statSync, lstatSync, mkdirSync, rmdirSync, unlinkSync, readdirSync, renameSync, realpathSync, copyFileSync, chmodSync, accessSync, mkdtempSync, truncateSync, fstatSync, readFile, writeFile, appendFile, exists, stat, lstat, mkdir, rmdir, unlink, readdir, rename, realpath, copyFile, access, chmod, promises, constants, watch, watchFile, unwatchFile, createReadStream, createWriteStream, openSync, closeSync, readSync, writeSync, ftruncateSync, fsyncSync };