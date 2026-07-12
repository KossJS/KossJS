// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/perf_hooks - Node.js perf_hooks module (L3)

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
    return Date.now();
  },
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
    return [...Object.values(marks).map((v, i) => new PerformanceEntry(Object.keys(marks)[i] || `mark_${i}`, 'mark', v, 0)),
            ...Object.values(measures)];
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
  timeOrigin: Date.now(),
  timing: {
    startTime: 0,
  },
};

class PerformanceObserver {
  constructor(callback) { this._callback = callback; }
  observe(options) { return this; }
  disconnect() {}
  takeRecords() { return []; }
}

function createHistogram() {
  return { min: 0, max: 0, mean: 0, exceeds: 0, stddev: 0, percentiles: new Map(), percentile: (p) => 0, reset: () => {}, record: (v) => {} };
}

function monitorEventLoopDelay(options) {
  return createHistogram();
}

function timerify(fn) {
  return function(...args) {
    const start = performance.now();
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
