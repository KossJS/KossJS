'use strict';

var kMaxTracingCount = 10;
var enabledTracingObjects = [];
var enabledCategories = '';

function normalizeCategories(categories) {
  if (Array.isArray(categories)) return categories;
  if (typeof categories === 'string') return categories.split(',');
  return [];
}

class Tracing {
  constructor(categories) {
    this._categories = normalizeCategories(categories);
    this._enabled = false;
  }

  enable() {
    if (!this._enabled) {
      this._enabled = true;
      enabledTracingObjects.push(this);
      enabledCategories = this._categories.join(',');
      if (enabledTracingObjects.length > kMaxTracingCount) {
        if (typeof process !== 'undefined' && typeof process.emitWarning === 'function') {
          process.emitWarning(
            'Possible trace_events memory leak detected. There are more than ' +
            kMaxTracingCount + ' enabled Tracing objects.',
          );
        }
      }
    }
  }

  disable() {
    if (this._enabled) {
      this._enabled = false;
      var idx = enabledTracingObjects.indexOf(this);
      if (idx !== -1) enabledTracingObjects.splice(idx, 1);
      enabledCategories = '';
      for (var i = 0; i < enabledTracingObjects.length; i++) {
        if (enabledCategories) enabledCategories += ',';
        enabledCategories += enabledTracingObjects[i]._categories.join(',');
      }
    }
  }

  get enabled() { return this._enabled; }
  get categories() { return this._categories.join(','); }
}

function createTracing(options) {
  if (typeof options !== 'object' || options === null) {
    throw new TypeError('options must be an object');
  }
  if (!Array.isArray(options.categories) || options.categories.length <= 0) {
    throw new Error('categories is required');
  }
  return new Tracing(options.categories);
}

function getEnabledCategories() {
  return enabledCategories || undefined;
}

module.exports = {
  Tracing: Tracing,
  createTracing: createTracing,
  getEnabledCategories: getEnabledCategories,
};
