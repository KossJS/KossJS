// koss:system — Koss 原生系统与进程模块
// 架构、平台、内存、CPU、环境变量等

var process = globalThis.process;

function arch() {
  return (process && process.arch) || 'unknown';
}

function platform() {
  return (process && process.platform) || 'unknown';
}

function hostname() {
  return (process && typeof process.hostname === 'function' && process.hostname()) || 'unknown';
}

function cpus() {
  if (process && typeof process.cpus === 'function') {
    return process.cpus();
  }
  return [];
}

function memory() {
  if (process && typeof process.memoryUsage === 'function') {
    var usage = process.memoryUsage();
    return {
      total: usage.rss + (usage.heapTotal || 0),
      free: (usage.heapTotal || 0) - (usage.heapUsed || 0),
      used: usage.heapUsed || 0,
    };
  }
  return { total: 0, free: 0, used: 0 };
}

function uptime() {
  if (process && typeof process.uptime === 'function') {
    return process.uptime();
  }
  return 0;
}

function loadavg() {
  if (process && typeof process.loadavg === 'function') {
    return process.loadavg();
  }
  return [0, 0, 0];
}

function env(key) {
  var envObj = (process && process.env) || {};
  if (key === undefined) {
    var result = {};
    for (var k in envObj) { result[k] = envObj[k]; }
    return result;
  }
  return envObj[String(key)] || undefined;
}

function pid() {
  return (process && process.pid) || 0;
}

function exit(code) {
  if (process && typeof process.exit === 'function') {
    process.exit(code || 0);
  }
  throw new Error('Process exit: ' + (code || 0));
}

function cwd() {
  if (process && typeof process.cwd === 'function') {
    return process.cwd();
  }
  return '.';
}

function chdir(path) {
  if (process && typeof process.chdir === 'function') {
    process.chdir(path);
  }
}

function version() {
  return (process && process.version) || 'unknown';
}

function versions() {
  return (process && process.versions) || {};
}

function nextTick(fn) {
  if (process && typeof process.nextTick === 'function') {
    process.nextTick(fn);
  } else {
    Promise.resolve().then(fn);
  }
}

module.exports = {
  arch: arch, platform: platform, hostname: hostname, cpus: cpus,
  memory: memory, uptime: uptime, loadavg: loadavg,
  env: env, pid: pid, exit: exit, cwd: cwd, chdir: chdir,
  version: version, versions: versions, nextTick: nextTick,
};
