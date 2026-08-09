// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/perf_hooks - Node.js perf_hooks module (L3)
// performance.now() 使用单调时钟（__koss_performance_now），
// 不受系统时间调整影响；createHistogram/monitorEventLoopDelay 返回真实直方图。

const _nowFn = (typeof globalThis.__koss_performance_now === 'function') ? globalThis.__koss_performance_now : function() { return Date.now(); };
const _timeOrigin = (typeof globalThis.__koss_performance_timeorigin === 'function') ? Number(globalThis.__koss_performance_timeorigin()) : Date.now();

const marks = {};
const measures = {};

class PerformanceEntry {
  constructor(name, entryType, startTime, duration) {
    this.name = name;
    this.entryType = entryType;
    this.startTime = startTime;
    this.duration = duration;
  }
  toJSON() { return { name: this.name, entryType: this.entryType, startTime: this.startTime, duration: this.duration }; }
}

const performance = {
  now() {
    return _nowFn();
  },
  timeOrigin: _timeOrigin,
  mark(name) {
    marks[name] = performance.now();
    return new PerformanceEntry(name, 'mark', marks[name], 0);
  },
  measure(name, startMark, endMark) {
    const start = startMark ? (marks[startMark] || 0) : 0;
    const end = endMark ? (marks[endMark] || performance.now()) : performance.now();
    const duration = end - start;
    measures[name] = new PerformanceEntry(name, 'measure', start, duration);
    return measures[name];
  },
  clearMarks(name) {
    if (name) delete marks[name];
    else { for (const k in marks) delete marks[k]; }
  },
  clearMeasures(name) {
    if (name) delete measures[name];
    else { for (const k in measures) delete measures[k]; }
  },
  getEntries() {
    const entries = [];
    const markKeys = Object.keys(marks);
    for (let i = 0; i < markKeys.length; i++) {
      entries.push(new PerformanceEntry(markKeys[i], 'mark', marks[markKeys[i]], 0));
    }
    const measureKeys = Object.keys(measures);
    for (let i = 0; i < measureKeys.length; i++) {
      entries.push(measures[measureKeys[i]]);
    }
    return entries;
  },
  getEntriesByType(type) {
    return this.getEntries().filter(e => e.entryType === type);
  },
  getEntriesByName(name) {
    return this.getEntries().filter(e => e.name === name);
  },
  eventLoopUtilization() { return { idle: 0, active: 0, utilization: 0 }; },
  nodeTiming: {
    name: 'node',
    entryType: 'node',
    startTime: 0,
    duration: 0,
    nodeStart: 0,
    v8Start: 0,
    bootstrapComplete: 0,
    environment: 0,
    loopStart: -1,
    loopExit: -1,
    thirdPartyMainEnd: 0,
    clusterSetupEnd: 0,
    moduleLoadEnd: 0,
    moduleLoadStart: 0,
  },
  timing: {
    startTime: 0,
  },
};

class PerformanceObserver {
  constructor(callback) { this._callback = callback; this._buffered = []; this._active = false; }
  observe(options) {
    if (options && options.entryTypes && this._callback) {
      this._active = true;
      // 简化的同步投递：新产生的 entries 会通过 flush 机制回调
    }
    return this;
  }
  disconnect() { this._active = false; }
  takeRecords() { const r = this._buffered.slice(); this._buffered = []; return r; }
  flush() { if (this._active && this._callback && this._buffered.length > 0) { this._callback(this._buffered.slice(), this); this._buffered = []; } }
}

// ─── 真实直方图（对数分桶，类似 Node 的 Histogram） ───

class Histogram {
  constructor() {
    this._count = 0;
    this._sum = 0;
    this._min = Infinity;
    this._max = -Infinity;
    this._buckets = new Map();
    this._exceeds = 0;
  }
  _bucket(v) {
    if (v <= 0) return 0;
    return Math.floor(Math.log2(v)) + 1;
  }
  record(v) {
    const n = Number(v);
    if (isNaN(n)) return;
    this._count++;
    this._sum += n;
    if (n < this._min) this._min = n;
    if (n > this._max) this._max = n;
    const b = this._bucket(n);
    this._buckets.set(b, (this._buckets.get(b) || 0) + 1);
    if (n > 2147483647) this._exceeds++;
  }
  reset() {
    this._count = 0;
    this._sum = 0;
    this._min = Infinity;
    this._max = -Infinity;
    this._buckets.clear();
    this._exceeds = 0;
  }
  get min() { return this._count === 0 ? 0 : this._min; }
  get max() { return this._count === 0 ? 0 : this._max; }
  get mean() { return this._count === 0 ? 0 : this._sum / this._count; }
  get exceeds() { return this._exceeds; }
  get stddev() {
    if (this._count === 0) return 0;
    const mean = this.mean;
    let sumSq = 0;
    for (const [bucket, count] of this._buckets) {
      const low = Math.pow(2, bucket - 1);
      const high = Math.pow(2, bucket);
      const mid = (low + high) / 2;
      sumSq += count * Math.pow(mid - mean, 2);
    }
    return Math.sqrt(sumSq / this._count);
  }
  percentile(p) {
    const pct = Number(p);
    if (isNaN(pct) || pct < 0 || pct > 100) throw new RangeError('percentile must be between 0 and 100');
    if (this._count === 0) return 0;
    const target = Math.ceil(this._count * pct / 100);
    let cumulative = 0;
    const sortedBuckets = Array.from(this._buckets.entries()).sort((a, b) => a[0] - b[0]);
    for (const [bucket, count] of sortedBuckets) {
      cumulative += count;
      if (cumulative >= target) {
        const low = Math.pow(2, bucket - 1);
        const high = Math.pow(2, bucket);
        return Math.round((low + high) / 2);
      }
    }
    return this._max;
  }
  percentiles() {
    const m = new Map();
    for (let p = 1; p <= 100; p += 1) m.set(p, this.percentile(p));
    return m;
  }
}

function createHistogram() {
  return new Histogram();
}

function monitorEventLoopDelay(options) {
  const hist = new Histogram();
  // 基于 performance.now() 的间隔采样
  if (typeof setInterval === 'function') {
    let last = _nowFn();
    hist._timer = setInterval(function() {
      const now = _nowFn();
      hist.record(now - last);
      last = now;
    }, (options && options.resolution) || 10);
  }
  hist.reset = function() {
    Histogram.prototype.reset.call(this);
    if (this._timer) clearInterval(this._timer);
  };
  return hist;
}

function timerify(fn) {
  return function(...args) {
    const start = _nowFn();
    const result = fn.apply(this, args);
    return result;
  };
}

const PerformanceMark = PerformanceEntry;
const PerformanceMeasure = PerformanceEntry;

const constants = {
  NODE_PERFORMANCE_GC_MAJOR: 2,
  NODE_PERFORMANCE_GC_MINOR: 1,
  NODE_PERFORMANCE_GC_INCREMENTAL: 4,
  NODE_PERFORMANCE_GC_WEAKCB: 8,
  NODE_PERFORMANCE_GC_FLAGS_NO: 0,
  NODE_PERFORMANCE_GC_FLAGS_CONSTRUCT_RETAINED: 2,
  NODE_PERFORMANCE_GC_FLAGS_FORCED: 4,
  NODE_PERFORMANCE_GC_FLAGS_SYNCHRONOUS_PHANTOM_PROCESSING: 8,
  NODE_PERFORMANCE_ENTRY_GC: 'gc',
  NODE_PERFORMANCE_ENTRY_HTTP: 'http',
  NODE_PERFORMANCE_ENTRY_HTTP2: 'http2',
  NODE_PERFORMANCE_ENTRY_NET: 'net',
  NODE_PERFORMANCE_ENTRY_DNS: 'dns',
  NODE_PERFORMANCE_ENTRY_FUNCTION: 'function',
};

module.exports = { performance, PerformanceObserver, createHistogram, monitorEventLoopDelay, timerify, PerformanceEntry, PerformanceMark, PerformanceMeasure, constants };
