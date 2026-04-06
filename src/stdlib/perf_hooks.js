/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const {
  ObjectDefineProperty,
} = primordials;

const {
  constants,
} = internalBinding('performance');

const { PerformanceEntry } = require('internal/perf/performance_entry');
const { PerformanceResourceTiming } = require('internal/perf/resource_timing');
const {
  PerformanceObserver,
  PerformanceObserverEntryList,
} = require('internal/perf/observe');
const {
  PerformanceMark,
  PerformanceMeasure,
} = require('internal/perf/usertiming');
const {
  Performance,
  performance,
} = require('internal/perf/performance');

const {
  createHistogram,
} = require('internal/histogram');

const monitorEventLoopDelay = require('internal/perf/event_loop_delay');
const { eventLoopUtilization } = require('internal/perf/event_loop_utilization');
const timerify = require('internal/perf/timerify');

module.exports = {
  Performance,
  PerformanceEntry,
  PerformanceMark,
  PerformanceMeasure,
  PerformanceObserver,
  PerformanceObserverEntryList,
  PerformanceResourceTiming,
  monitorEventLoopDelay,
  eventLoopUtilization,
  timerify,
  createHistogram,
  performance,
};

ObjectDefineProperty(module.exports, 'constants', {
  __proto__: null,
  configurable: false,
  enumerable: true,
  value: constants,
});

