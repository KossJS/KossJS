'use strict';

var perfHooks;

try {
  perfHooks = internalBinding('performance');
} catch (_) {
  perfHooks = {};
}

var constants = perfHooks.constants || {};

var performance = globalThis.performance || {
  now: function() { return Date.now(); },
  mark: function(name) {},
  measure: function(name, startMark, endMark) {},
  clearMarks: function(name) {},
  clearMeasures: function(name) {},
  getEntries: function() { return []; },
  getEntriesByType: function(type) { return []; },
  getEntriesByName: function(name) { return []; },
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
    idleTime: 0,
  },
  timeOrigin: Date.now(),
  timing: {
    startTime: 0,
  },
};

class PerformanceEntry {
  constructor() {
    this.name = '';
    this.entryType = '';
    this.startTime = 0;
    this.duration = 0;
  }
  toJSON() {
    return {
      name: this.name,
      entryType: this.entryType,
      startTime: this.startTime,
      duration: this.duration,
    };
  }
}

class PerformanceMark extends PerformanceEntry {
  constructor(name, options) {
    super();
    this.name = name;
    this.entryType = 'mark';
    this.startTime = typeof performance !== 'undefined' ? performance.now() : Date.now();
    this.duration = 0;
  }
}

class PerformanceMeasure extends PerformanceEntry {
  constructor() {
    super();
    this.entryType = 'measure';
  }
}

class PerformanceObserver {
  constructor(callback) {
    if (typeof callback !== 'function') throw new TypeError('callback must be a function');
    this._callback = callback;
  }
  observe(options) {}
  disconnect() {}
  takeRecords() { return []; }
}

PerformanceObserver.supportedEntryTypes = [];

class PerformanceObserverEntryList {
  constructor(entries) {
    this._entries = entries || [];
  }
  getEntries() { return this._entries; }
  getEntriesByType(type) { return this._entries.filter(function(e) { return e.entryType === type; }); }
  getEntriesByName(name, type) {
    return this._entries.filter(function(e) {
      if (type) return e.name === name && e.entryType === type;
      return e.name === name;
    });
  }
}

class PerformanceResourceTiming extends PerformanceEntry {
  constructor() {
    super();
    this.entryType = 'resource';
    this.initiatorType = '';
    this.nextHopProtocol = '';
    this.workerStart = 0;
    this.redirectStart = 0;
    this.redirectEnd = 0;
    this.fetchStart = 0;
    this.domainLookupStart = 0;
    this.domainLookupEnd = 0;
    this.connectStart = 0;
    this.connectEnd = 0;
    this.secureConnectionStart = 0;
    this.requestStart = 0;
    this.responseStart = 0;
    this.responseEnd = 0;
    this.transferSize = 0;
    this.encodedBodySize = 0;
    this.decodedBodySize = 0;
    this.responseStatus = 200;
  }
}

function monitorEventLoopDelay(options) {
  return {
    enable: function() {},
    disable: function() {},
    percentile: function(p) { return 0; },
    min: 0,
    max: 0,
    mean: 0,
    stddev: 0,
    count: 0,
    raw: [],
    histogram: {},
  };
}

function eventLoopUtilization(util1, util2) {
  return {
    idle: 0,
    active: 0,
    utilization: 0,
  };
}

function timerify(fn) {
  if (typeof fn !== 'function') throw new TypeError('fn must be a function');
  return fn;
}

function createHistogram(options) {
  return {
    min: 0,
    max: 0,
    mean: 0,
    stddev: 0,
    count: 0,
    percentile: function(p) { return 0; },
    reset: function() {},
    record: function(val) {},
  };
}

var exports = {
  Performance: PerformanceEntry,
  PerformanceEntry: PerformanceEntry,
  PerformanceMark: PerformanceMark,
  PerformanceMeasure: PerformanceMeasure,
  PerformanceObserver: PerformanceObserver,
  PerformanceObserverEntryList: PerformanceObserverEntryList,
  PerformanceResourceTiming: PerformanceResourceTiming,
  monitorEventLoopDelay: monitorEventLoopDelay,
  eventLoopUtilization: eventLoopUtilization,
  timerify: timerify,
  createHistogram: createHistogram,
  performance: performance,
  constants: constants,
};

module.exports = exports;
