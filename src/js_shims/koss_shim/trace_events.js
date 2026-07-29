// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:trace_events — Koss 原生跟踪事件标准库

var enabledCategories = new Set();

function Tracing(options) {
  if (!options || !options.categories || options.categories.length === 0) {
    throw new TypeError('At least one trace category must be specified');
  }
  this.categories = options.categories.join(',');
  this.enabled = false;
}

Tracing.prototype.enable = function() {
  this.enabled = true;
  enabledCategories.add(this.categories);
};

Tracing.prototype.disable = function() {
  this.enabled = false;
  enabledCategories.delete(this.categories);
};

function createTracing(options) {
  return new Tracing(options);
}

function getEnabledCategories() {
  return Array.from(enabledCategories).join(',') || '';
}

module.exports = { createTracing: createTracing, getEnabledCategories: getEnabledCategories, Tracing: Tracing };
