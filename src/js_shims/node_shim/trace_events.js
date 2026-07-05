// koss:node/trace_events - Node.js trace_events module (L3)

const enabledCategories = new Set();

class Tracing {
  constructor(options) {
    if (!options || !options.categories || options.categories.length === 0) {
      throw new TypeError('At least one trace category must be specified');
    }
    this.categories = options.categories.join(',');
    this.enabled = false;
  }
  enable() { this.enabled = true; enabledCategories.add(this.categories); }
  disable() { this.enabled = false; enabledCategories.delete(this.categories); }
}

function createTracing(options) {
  return new Tracing(options);
}

function getEnabledCategories() {
  return Array.from(enabledCategories).join(',') || '';
}

module.exports = { createTracing, getEnabledCategories, Tracing };
