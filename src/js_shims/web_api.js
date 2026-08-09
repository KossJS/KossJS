// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// Web API 全局引导模块 — 将 Web 标准 API 安装到 globalThis。
// 这些 API 在 Node.js / Bun / Deno 中均为全局可用，因此 KossJS 也应提供。
// 纯 JS 实现，无外部依赖。由运行时初始化时通过 __koss_load_module 加载。

(function(globalThis) {
  'use strict';

  // ── self / globalThis ──
  if (typeof globalThis.self === 'undefined') {
    Object.defineProperty(globalThis, 'self', {
      get: function() { return globalThis; },
      configurable: true,
      enumerable: true,
    });
  }

  // ── queueMicrotask ──
  if (typeof globalThis.queueMicrotask !== 'function') {
    globalThis.queueMicrotask = function queueMicrotask(callback) {
      Promise.resolve().then(function() {
        if (typeof callback === 'function') callback();
      });
    };
  }

  // ── structuredClone（结构化克隆，支持循环引用与 Buffer/ArrayBuffer） ──
  if (typeof globalThis.structuredClone !== 'function') {
    function structuredClone(value) {
      return _deepClone(value, new WeakMap());
    }
    function _deepClone(value, seen) {
      if (value === null || value === undefined) return value;
      var t = typeof value;
      if (t === 'number' || t === 'string' || t === 'boolean' || t === 'bigint' || t === 'symbol') return value;
      if (t === 'function') return undefined;
      // 类型化数组 / Buffer
      if (value instanceof Uint8Array) {
        var u8 = new Uint8Array(value.length);
        u8.set(value);
        return u8;
      }
      if (value instanceof ArrayBuffer) {
        return value.slice(0);
      }
      if (ArrayBuffer.isView(value)) {
        var view = new value.constructor(value.length);
        for (var i = 0; i < value.length; i++) view[i] = value[i];
        return view;
      }
      if (seen.has(value)) return seen.get(value);
      var out;
      if (Array.isArray(value)) {
        out = [];
        seen.set(value, out);
        for (var j = 0; j < value.length; j++) out.push(_deepClone(value[j], seen));
        return out;
      }
      if (value instanceof Date) {
        return new Date(value.getTime());
      }
      if (value instanceof Map) {
        out = new Map();
        seen.set(value, out);
        var entries = value.entries();
        var e = entries.next();
        while (!e.done) {
          out.set(_deepClone(e.value[0], seen), _deepClone(e.value[1], seen));
          e = entries.next();
        }
        return out;
      }
      if (value instanceof Set) {
        out = new Set();
        seen.set(value, out);
        var s = value.values();
        var sv = s.next();
        while (!sv.done) {
          out.add(_deepClone(sv.value, seen));
          sv = s.next();
        }
        return out;
      }
      if (typeof value === 'object') {
        out = {};
        seen.set(value, out);
        var keys = Object.keys(value);
        for (var k = 0; k < keys.length; k++) {
          out[keys[k]] = _deepClone(value[keys[k]], seen);
        }
        return out;
      }
      return value;
    }
    globalThis.structuredClone = structuredClone;
  }

  // ── Event / CustomEvent ──
  if (typeof globalThis.Event !== 'function') {
    function Event(type, options) {
      if (typeof type !== 'string') throw new TypeError('Event type must be a string');
      this.type = type;
      this.target = null;
      this.currentTarget = null;
      this.bubbles = !!(options && options.bubbles);
      this.cancelable = !!(options && options.cancelable);
      this.defaultPrevented = false;
      this.eventPhase = 0;
      this.timeStamp = Date.now();
      this._propagated = false;
    }
    Event.prototype.stopPropagation = function() { this._propagated = true; };
    Event.prototype.stopImmediatePropagation = function() { this._propagated = true; };
    Event.prototype.preventDefault = function() { if (this.cancelable) this.defaultPrevented = true; };
    globalThis.Event = Event;
  }

  if (typeof globalThis.CustomEvent !== 'function') {
    function CustomEvent(type, options) {
      var opts = options || {};
      Event.call(this, type, opts);
      this.detail = opts.detail !== undefined ? opts.detail : null;
    }
    CustomEvent.prototype = Object.create(Event.prototype);
    CustomEvent.prototype.constructor = CustomEvent;
    globalThis.CustomEvent = CustomEvent;
  }

  // ── MessageEvent / ErrorEvent / CloseEvent ──
  if (typeof globalThis.MessageEvent !== 'function') {
    function MessageEvent(type, options) {
      var opts = options || {};
      Event.call(this, type, opts);
      this.data = opts.data !== undefined ? opts.data : null;
      this.origin = opts.origin !== undefined ? opts.origin : '';
      this.lastEventId = opts.lastEventId !== undefined ? opts.lastEventId : '';
      this.source = opts.source !== undefined ? opts.source : null;
      this.ports = opts.ports !== undefined ? opts.ports : [];
    }
    MessageEvent.prototype = Object.create(Event.prototype);
    MessageEvent.prototype.constructor = MessageEvent;
    globalThis.MessageEvent = MessageEvent;
  }

  if (typeof globalThis.ErrorEvent !== 'function') {
    function ErrorEvent(type, options) {
      var opts = options || {};
      Event.call(this, type, opts);
      this.message = opts.message !== undefined ? opts.message : '';
      this.filename = opts.filename !== undefined ? opts.filename : '';
      this.lineno = opts.lineno !== undefined ? opts.lineno : 0;
      this.colno = opts.colno !== undefined ? opts.colno : 0;
      this.error = opts.error !== undefined ? opts.error : null;
    }
    ErrorEvent.prototype = Object.create(Event.prototype);
    ErrorEvent.prototype.constructor = ErrorEvent;
    globalThis.ErrorEvent = ErrorEvent;
  }

  if (typeof globalThis.CloseEvent !== 'function') {
    function CloseEvent(type, options) {
      var opts = options || {};
      Event.call(this, type, opts);
      this.wasClean = opts.wasClean !== undefined ? opts.wasClean : false;
      this.code = opts.code !== undefined ? opts.code : 0;
      this.reason = opts.reason !== undefined ? opts.reason : '';
    }
    CloseEvent.prototype = Object.create(Event.prototype);
    CloseEvent.prototype.constructor = CloseEvent;
    globalThis.CloseEvent = CloseEvent;
  }

  // ── EventTarget（Web 标准事件目标基类） ──
  if (typeof globalThis.EventTarget !== 'function') {
    function EventTarget() {
      this._kossEventListeners = {};
    }
    EventTarget.prototype.addEventListener = function(type, listener, options) {
      if (typeof listener !== 'function') return;
      var key = String(type);
      if (!this._kossEventListeners[key]) this._kossEventListeners[key] = [];
      var opts = (options === true) ? { capture: true } : (options || {});
      var entry = {
        listener: listener,
        once: !!(opts.once),
        capture: !!(opts.capture),
      };
      this._kossEventListeners[key].push(entry);
    };
    EventTarget.prototype.removeEventListener = function(type, listener, options) {
      var key = String(type);
      var arr = this._kossEventListeners[key];
      if (!arr) return;
      for (var i = 0; i < arr.length; i++) {
        if (arr[i].listener === listener) { arr.splice(i, 1); break; }
      }
    };
    EventTarget.prototype.dispatchEvent = function(event) {
      if (!event || event.type === undefined) return false;
      var key = String(event.type);
      var arr = (this._kossEventListeners[key] || []).slice();
      event.target = this;
      event.currentTarget = this;
      for (var i = 0; i < arr.length; i++) {
        var entry = arr[i];
        if (entry.once) this.removeEventListener(key, entry.listener);
        try {
          entry.listener.call(this, event);
        } catch (e) {
          // 监听器异常不应中断其他监听器
        }
        if (event._propagated) break;
      }
      return true;
    };
    globalThis.EventTarget = EventTarget;
  }

  // ── DOMException ──
  if (typeof globalThis.DOMException !== 'function') {
    function DOMException(message, name) {
      var e = new Error(message === undefined || message === null ? '' : String(message));
      e.name = name || 'Error';
      e.message = message === undefined || message === null ? '' : String(message);
      return e;
    }
    DOMException.prototype = Object.create(Error.prototype);
    DOMException.prototype.constructor = DOMException;
    globalThis.DOMException = DOMException;
  }

  // ── AbortController / AbortSignal ──
  if (typeof globalThis.AbortController !== 'function') {
    var kkAbortListeners = Symbol('abortListeners');
    function AbortSignal() {
      this.aborted = false;
      this.reason = undefined;
      this.onabort = null;
      this[kkAbortListeners] = [];
    }
    AbortSignal.prototype.addEventListener = function(type, listener, options) {
      if (type === 'abort' && typeof listener === 'function') {
        var entry = { listener: listener, once: !!(options && options.once) };
        this[kkAbortListeners].push(entry);
      }
    };
    AbortSignal.prototype.removeEventListener = function(type, listener) {
      if (type === 'abort') {
        var arr = this[kkAbortListeners];
        for (var i = 0; i < arr.length; i++) {
          if (arr[i].listener === listener) { arr.splice(i, 1); break; }
        }
      }
    };
    AbortSignal.prototype.dispatchEvent = function(event) {
      if (event && event.type === 'abort') {
        this.aborted = true;
        this.reason = event.reason !== undefined ? event.reason : new DOMException('This operation was aborted', 'AbortError');
        if (typeof this.onabort === 'function') this.onabort(event);
        var arr = this[kkAbortListeners].slice();
        for (var i = 0; i < arr.length; i++) {
          var entry = arr[i];
          if (entry.once) this.removeEventListener('abort', entry.listener);
          entry.listener(event);
        }
      }
      return true;
    };
    AbortSignal.prototype.throwIfAborted = function() {
      if (this.aborted) throw this.reason instanceof Error ? this.reason : new DOMException('The operation was aborted', 'AbortError');
    };
    AbortSignal.abort = function(reason) {
      var signal = new AbortSignal();
      signal.aborted = true;
      signal.reason = reason !== undefined ? reason : new DOMException('This operation was aborted', 'AbortError');
      return signal;
    };
    AbortSignal.timeout = function(ms) {
      var controller = new AbortController();
      setTimeout(function() {
        controller.abort(new DOMException('The operation was aborted due to timeout', 'TimeoutError'));
      }, Number(ms) || 0);
      return controller.signal;
    };
    AbortSignal.any = function(signals) {
      var controller = new AbortController();
      var list = signals || [];
      for (var i = 0; i < list.length; i++) {
        var sig = list[i];
        if (!sig) continue;
        if (sig.aborted) {
          controller.abort(sig.reason);
          break;
        }
        if (typeof sig.addEventListener === 'function') {
          sig.addEventListener('abort', function(event) {
            controller.abort(event && event.reason);
          }, { once: true });
        }
      }
      return controller.signal;
    };
    globalThis.AbortSignal = AbortSignal;

    function AbortController() {
      this.signal = new AbortSignal();
    }
    AbortController.prototype.abort = function(reason) {
      if (this.signal.aborted) return;
      var event = { type: 'abort', reason: reason };
      this.signal.dispatchEvent(event);
    };
    globalThis.AbortController = AbortController;
  }

  // ── URL / URLSearchParams ──
  if (typeof globalThis.URL !== 'function') {
    try {
      var urlModule = require('koss:url');
      if (urlModule && typeof urlModule.URL === 'function') {
        globalThis.URL = urlModule.URL;
        globalThis.URLSearchParams = urlModule.URLSearchParams;
      }
    } catch (e) { /* koss:url 不可用时忽略 */ }
  }

  // ── URLPattern ──
  if (typeof globalThis.URLPattern !== 'function') {
    function _patternToRegex(pattern) {
      // 转换 URLPattern 语法：:name → 捕获组，* → .*，其余正则特殊字符转义
      var p = String(pattern || '');
      var regex = '';
      var i = 0;
      var groups = [];
      while (i < p.length) {
        var ch = p[i];
        if (ch === '*') {
          regex += '.*';
          i++;
        } else if (ch === ':') {
          // 捕获参数名直到非字母数字
          var j = i + 1;
          var name = '';
          while (j < p.length && /[A-Za-z0-9_]/.test(p[j])) { name += p[j]; j++; }
          groups.push(name || '');
          regex += '([^/]+)';
          i = j;
        } else if (ch === '{') {
          // 可选组
          regex += '(?:';
          i++;
        } else if (ch === '}') {
          regex += ')?';
          i++;
        } else if (ch === '(') {
          regex += '(';
          i++;
        } else if (ch === ')') {
          regex += ')';
          i++;
        } else if ('\\.+?^$|[]'.indexOf(ch) !== -1) {
          regex += '\\' + ch;
          i++;
        } else {
          regex += ch;
          i++;
        }
      }
      return { regex: new RegExp('^' + regex + '$'), groups: groups };
    }
    function URLPattern(input, baseURL) {
      var init;
      if (typeof input === 'string') {
        init = { pathname: input };
      } else {
        init = input || {};
      }
      if (baseURL && typeof baseURL === 'string') {
        init.baseURL = baseURL;
      }
      this._baseURL = init.baseURL || null;
      var pathname = init.pathname !== undefined ? init.pathname : '*';
      var search = init.search !== undefined ? init.search : '*';
      var hash = init.hash !== undefined ? init.hash : '*';
      var protocol = init.protocol !== undefined ? init.protocol : '*';
      var hostname = init.hostname !== undefined ? init.hostname : '*';
      var port = init.port !== undefined ? init.port : '*';
      this._pathname = _patternToRegex(pathname);
      this._search = _patternToRegex(search);
      this._hash = _patternToRegex(hash);
      this._protocol = _patternToRegex(protocol);
      this._hostname = _patternToRegex(hostname);
      this._port = _patternToRegex(port);
    }
    URLPattern.prototype.test = function(url) {
      return this.exec(url) !== null;
    };
    URLPattern.prototype.exec = function(url) {
      var target = typeof url === 'string' ? url : (url && url.href) || '';
      var parsed = null;
      try {
        parsed = this._baseURL ? new URL(target, this._baseURL) : new URL(target, 'https://kossjs.invalid/');
      } catch (e) {
        parsed = null;
      }
      if (!parsed) return null;
      var path = parsed.pathname || '/';
      // search 匹配时去掉前导 '?'
      var rawSearch = parsed.search || '';
      var search = rawSearch.charAt(0) === '?' ? rawSearch.substring(1) : rawSearch;
      var hash = parsed.hash || '';
      var protocol = parsed.protocol || '';
      var hostname = parsed.hostname || '';
      var port = parsed.port || '';
      if (!this._pathname.regex.test(path)) return null;
      if (!this._search.regex.test(search)) return null;
      if (!this._hash.regex.test(hash)) return null;
      if (!this._protocol.regex.test(protocol)) return null;
      if (!this._hostname.regex.test(hostname)) return null;
      if (!this._port.regex.test(port)) return null;
      var pathMatch = path.match(this._pathname.regex);
      var searchMatch = search.match(this._search.regex);
      var hashMatch = hash.match(this._hash.regex);
      var groups = {};
      for (var i = 0; i < this._pathname.groups.length; i++) {
        groups[this._pathname.groups[i]] = pathMatch[i + 1];
      }
      for (var j = 0; j < this._search.groups.length; j++) {
        groups[this._search.groups[j]] = searchMatch[j + 1];
      }
      for (var k = 0; k < this._hash.groups.length; k++) {
        groups[this._hash.groups[k]] = hashMatch[k + 1];
      }
      return {
        input: target,
        pathname: { input: path, groups: groups },
        search: { input: search, groups: groups },
        hash: { input: hash, groups: groups },
        protocol: { input: protocol, groups: {} },
        hostname: { input: hostname, groups: {} },
        port: { input: port, groups: {} },
        groups: groups,
      };
    };
    URLPattern.prototype[Symbol.toStringTag] = 'URLPattern';
    globalThis.URLPattern = URLPattern;
  }

  // ── performance ──
  if (typeof globalThis.performance === 'undefined') {
    try {
      var perfModule = require('koss:node/perf_hooks');
      if (perfModule && perfModule.performance) {
        globalThis.performance = perfModule.performance;
      }
    } catch (e) { /* perf_hooks 不可用时忽略 */ }
  }

  // ── File（基于 Blob） ──
  if (typeof globalThis.File !== 'function') {
    function File(fileBits, fileName, options) {
      var BlobCtor = globalThis.Blob;
      var opts = options || {};
      var blob = new BlobCtor(fileBits || [], { type: opts.type || '' });
      // 复用 Blob 内部数据
      var parts = blob._parts || [];
      BlobCtor.call(this, parts, { type: opts.type || '' });
      this.name = String(fileName);
      this.lastModified = opts.lastModified !== undefined ? Number(opts.lastModified) : Date.now();
    }
    try {
      File.prototype = Object.create(globalThis.Blob.prototype);
      File.prototype.constructor = File;
    } catch (e) { /* 忽略 */ }
    globalThis.File = File;
  }

  // ── FormData ──
  if (typeof globalThis.FormData !== 'function') {
    function FormData() {
      this._entries = [];
    }
    FormData.prototype.append = function(name, value, filename) {
      this._entries.push({ name: String(name), value: value, filename: filename !== undefined ? String(filename) : undefined });
    };
    FormData.prototype.delete = function(name) {
      var nameStr = String(name);
      this._entries = this._entries.filter(function(e) { return e.name !== nameStr; });
    };
    FormData.prototype.get = function(name) {
      var nameStr = String(name);
      for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i].name === nameStr) return this._entries[i].value;
      }
      return null;
    };
    FormData.prototype.getAll = function(name) {
      var nameStr = String(name);
      var out = [];
      for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i].name === nameStr) out.push(this._entries[i].value);
      }
      return out;
    };
    FormData.prototype.has = function(name) {
      var nameStr = String(name);
      for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i].name === nameStr) return true;
      }
      return false;
    };
    FormData.prototype.set = function(name, value, filename) {
      var nameStr = String(name);
      this.delete(nameStr);
      this.append(nameStr, value, filename);
    };
    FormData.prototype.entries = function() {
      var arr = [];
      for (var i = 0; i < this._entries.length; i++) arr.push([this._entries[i].name, this._entries[i].value]);
      return arr[Symbol.iterator]();
    };
    FormData.prototype.keys = function() {
      var self = this;
      return (function*() {
        for (var i = 0; i < self._entries.length; i++) yield self._entries[i].name;
      })();
    };
    FormData.prototype.values = function() {
      var self = this;
      return (function*() {
        for (var i = 0; i < self._entries.length; i++) yield self._entries[i].value;
      })();
    };
    FormData.prototype.forEach = function(callback, thisArg) {
      for (var i = 0; i < this._entries.length; i++) {
        var e = this._entries[i];
        callback.call(thisArg, e.value, e.name, this);
      }
    };
    FormData.prototype[Symbol.iterator] = function() { return this.entries(); };
    globalThis.FormData = FormData;
  }

  // ── Request（Web Fetch 标准） ──
  if (typeof globalThis.Request !== 'function') {
    function Request(input, init) {
      var opts = init || {};
      var url;
      var baseRequest = null;
      if (typeof input === 'string') {
        url = input;
      } else if (input && typeof input === 'object') {
        if (input instanceof globalThis.Request) {
          baseRequest = input;
          url = input.url;
        } else if (typeof input.url === 'string') {
          url = input.url;
        } else {
          url = String(input);
        }
      } else {
        throw new TypeError('Request input must be a string or object');
      }
      this.url = url;
      this.method = (opts.method || (baseRequest ? baseRequest.method : 'GET')).toUpperCase();
      this.headers = new Headers(baseRequest ? baseRequest.headers : {});
      if (opts.headers) {
        var h = new Headers(opts.headers);
        var it = h.entries();
        var entry = it.next();
        while (!entry.done) {
          this.headers.append ? this.headers.append(entry.value[0], entry.value[1]) : this.headers.set(entry.value[0], entry.value[1]);
          entry = it.next();
        }
      }
      this.body = opts.body !== undefined ? opts.body : (baseRequest ? baseRequest.body : null);
      this.signal = opts.signal || (baseRequest ? baseRequest.signal : null);
      this.credentials = opts.credentials || (baseRequest ? baseRequest.credentials : 'same-origin');
      this.mode = opts.mode || (baseRequest ? baseRequest.mode : 'cors');
      this.cache = opts.cache || (baseRequest ? baseRequest.cache : 'default');
      this.redirect = opts.redirect || (baseRequest ? baseRequest.redirect : 'follow');
      this.referrer = opts.referrer || (baseRequest ? baseRequest.referrer : 'about:client');
      this.duplex = opts.duplex;
      this._bodyUsed = false;
    }
    Request.prototype.clone = function() {
      if (this._bodyUsed) throw new TypeError('Cannot clone a used Request');
      return new Request(this);
    };
    Object.defineProperty(Request.prototype, 'bodyUsed', {
      get: function() { return this._bodyUsed; }
    });
    Request.prototype.arrayBuffer = function() {
      var body = this.body;
      this._bodyUsed = true;
      if (body === null || body === undefined) return Promise.resolve(new ArrayBuffer(0));
      if (body instanceof ArrayBuffer) return Promise.resolve(body.slice(0));
      if (typeof body === 'string') {
        var bytes = [];
        for (var i = 0; i < body.length; i++) bytes.push(body.charCodeAt(i) & 0xff);
        var buf = new ArrayBuffer(bytes.length);
        new Uint8Array(buf).set(bytes);
        return Promise.resolve(buf);
      }
      return Promise.resolve(body);
    };
    Request.prototype.text = function() {
      var self = this;
      return this.arrayBuffer().then(function() {
        var body = self.body;
        if (body === null || body === undefined) return '';
        if (typeof body === 'string') return body;
        return String(body);
      });
    };
    Request.prototype.json = function() {
      var self = this;
      return this.text().then(function(t) { return JSON.parse(t); });
    };
    Request.prototype.blob = function() {
      var self = this;
      return this.arrayBuffer().then(function() {
        return new Blob([new Uint8Array(self.body && self.body.byteLength ? self.body : 0)]);
      });
    };
    Request.prototype.formData = function() {
      var self = this;
      return this.text().then(function(t) {
        var fd = new FormData();
        var pairs = t.split('&');
        for (var i = 0; i < pairs.length; i++) {
          if (!pairs[i]) continue;
          var parts = pairs[i].split('=');
          var key = parts[0] ? decodeURIComponent(parts[0].replace(/\+/g, ' ')) : '';
          var val = parts[1] ? decodeURIComponent(parts[1].replace(/\+/g, ' ')) : '';
          fd.append(key, val);
        }
        return fd;
      });
    };
    globalThis.Request = Request;
  }

  // ── Headers.append 补齐（Web 标准支持多值合并） ──
  if (typeof globalThis.Headers === 'function') {
    var _HeadersProto = globalThis.Headers.prototype;
    if (typeof _HeadersProto.append !== 'function') {
      _HeadersProto.append = function(name, value) {
        var key = String(name).toLowerCase();
        var existing = this.get ? this.get(key) : null;
        if (existing !== null && existing !== undefined) {
          this.set(key, existing + ', ' + String(value));
        } else {
          this.set(key, String(value));
        }
      };
    }
  }

  // ── Response.formData 补齐 ──
  if (typeof globalThis.Response === 'function') {
    var _ResponseProto = globalThis.Response.prototype;
    if (typeof _ResponseProto.formData !== 'function') {
      _ResponseProto.formData = function() {
        var self = this;
        return this.text().then(function(t) {
          var fd = new FormData();
          var pairs = t.split('&');
          for (var i = 0; i < pairs.length; i++) {
            if (!pairs[i]) continue;
            var parts = pairs[i].split('=');
            var key = parts[0] ? decodeURIComponent(parts[0].replace(/\+/g, ' ')) : '';
            var val = parts[1] ? decodeURIComponent(parts[1].replace(/\+/g, ' ')) : '';
            fd.append(key, val);
          }
          return fd;
        });
      };
    }
  }

  // ── fetch 的 signal / AbortController 支持 ──
  var _origFetch = globalThis.fetch;
  if (typeof _origFetch === 'function') {
    globalThis.fetch = function(input, init) {
      var req = (typeof input === 'string' || input instanceof globalThis.Request) ? new Request(input, init) : input;
      var signal = (init && init.signal) || (req && req.signal) || null;
      if (signal && signal.aborted) {
        return Promise.reject(signal.reason instanceof Error ? signal.reason : new DOMException('The operation was aborted', 'AbortError'));
      }
      if (signal && typeof signal.addEventListener === 'function') {
        return new Promise(function(resolve, reject) {
          var onAbort = function() {
            reject(signal.reason instanceof Error ? signal.reason : new DOMException('The operation was aborted', 'AbortError'));
          };
          signal.addEventListener('abort', onAbort, { once: true });
          _origFetch(input, init).then(function(res) {
            signal.removeEventListener('abort', onAbort);
            resolve(res);
          }, function(err) {
            signal.removeEventListener('abort', onAbort);
            reject(err);
          });
        });
      }
      return _origFetch(input, init);
    };
  }

  // ── global（Node 风格全局别名） ──
  if (typeof globalThis.global === 'undefined') {
    Object.defineProperty(globalThis, 'global', {
      get: function() { return globalThis; },
      configurable: true,
      enumerable: false,
    });
  }

  // ── navigator ──
  if (typeof globalThis.navigator === 'undefined') {
    var kossVersion = (typeof globalThis.KossJS !== 'undefined' && globalThis.KossJS.version) ? globalThis.KossJS.version : '';
    var kossPlatform = 'unknown';
    var kossArch = 'unknown';
    try {
      var sysModule = require('koss:system');
      kossPlatform = sysModule.platform();
      kossArch = sysModule.arch();
    } catch (e) { /* ignore */ }
    var navigatorObj = {};
    Object.defineProperty(navigatorObj, 'userAgent', {
      get: function() { return 'KossJS/' + kossVersion; },
      configurable: true,
    });
    Object.defineProperty(navigatorObj, 'platform', {
      get: function() { return kossPlatform; },
      configurable: true,
    });
    Object.defineProperty(navigatorObj, 'hardwareConcurrency', {
      get: function() {
        try { return require('koss:system').availableParallelism(); }
        catch (e) { return 1; }
      },
      configurable: true,
    });
    Object.defineProperty(navigatorObj, 'language', {
      get: function() { return 'en-US'; },
      configurable: true,
    });
    Object.defineProperty(navigatorObj, 'languages', {
      get: function() { return ['en-US']; },
      configurable: true,
    });
    globalThis.navigator = navigatorObj;
  }

  // ── reportError ──
  if (typeof globalThis.reportError !== 'function') {
    globalThis.reportError = function(e) {
      if (typeof console !== 'undefined' && console.error) {
        console.error(e);
      }
      return undefined;
    };
  }

  // ── Web Crypto：全局 crypto（getRandomValues / randomUUID / subtle） ──
  if (typeof globalThis.crypto === 'undefined') {
    var kossCryptoModule = null;
    try { kossCryptoModule = require('koss:crypto'); } catch (e) { kossCryptoModule = null; }

    var cryptoObj = {};
    if (kossCryptoModule) {
      cryptoObj.getRandomValues = function(array) {
        if (!array || typeof array.length !== 'number') throw new TypeError('getRandomValues: expected a typed array');
        var bytes = kossCryptoModule.randomBytes(array.length);
        for (var i = 0; i < array.length && i < bytes.length; i++) array[i] = bytes[i];
        return array;
      };
      cryptoObj.randomUUID = function() { return kossCryptoModule.uuid(); };
      cryptoObj.randomBytes = function(size) { return kossCryptoModule.randomBytes(Number(size) || 0); };
      cryptoObj.subtle = {
        digest: function(algorithm, data) {
          return Promise.resolve().then(function() {
            var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'SHA-256';
            var normalized = String(algo).toLowerCase().replace('-', '');
            var bytes = _toUint8Array(data);
            var out = kossCryptoModule.hashBytes(normalized, bytes);
            return out.buffer;
          });
        },
        encrypt: function(algorithm, key, data) {
          return Promise.resolve().then(function() {
            var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'AES-GCM';
            var keyBytes = _toUint8Array(key);
            var dataBytes = _toUint8Array(data);
            var combined = kossCryptoModule.encrypt(keyBytes, dataBytes);
            return combined.buffer;
          });
        },
        decrypt: function(algorithm, key, data) {
          return Promise.resolve().then(function() {
            var keyBytes = _toUint8Array(key);
            var dataBytes = _toUint8Array(data);
            var out = kossCryptoModule.decrypt(keyBytes, dataBytes);
            return out.buffer;
          });
        },
        generateKey: function(algorithm, extractable, keyUsages) {
          return Promise.resolve().then(function() {
            var algo = typeof algorithm === 'string' ? algorithm : (algorithm && algorithm.name) || 'Ed25519';
            if (algo === 'Ed25519' || algo === 'ed25519') {
              var kp = kossCryptoModule.ed25519KeyPair();
              return {
                publicKey: kp.publicKey,
                privateKey: kp.privateKey,
                extractable: !!extractable,
                type: 'private',
                algorithm: { name: 'Ed25519' },
                usages: keyUsages || [],
              };
            }
            if (algo === 'AES-GCM' || algo === 'aes-gcm') {
              var len = (algorithm && algorithm.length) || 256;
              return kossCryptoModule.randomBytes(Math.floor(Number(len) / 8));
            }
            var keyLen = (algorithm && algorithm.length) || 32;
            return kossCryptoModule.randomBytes(Math.floor(Number(keyLen) / 8));
          });
        },
        sign: function(algorithm, key, data) {
          return Promise.resolve().then(function() {
            var keyBytes = _toUint8Array(key);
            var dataBytes = _toUint8Array(data);
            var out = kossCryptoModule.sign(keyBytes, dataBytes);
            return out.buffer;
          });
        },
        verify: function(algorithm, key, signature, data) {
          return Promise.resolve().then(function() {
            var keyBytes = _toUint8Array(key);
            var sigBytes = _toUint8Array(signature);
            var dataBytes = _toUint8Array(data);
            return kossCryptoModule.verify(keyBytes, dataBytes, sigBytes);
          });
        },
        importKey: function(format, keyData, algorithm, extractable, keyUsages) {
          return Promise.resolve().then(function() {
            if (format === 'raw') return _toUint8Array(keyData);
            throw new DOMException('Unsupported key format: ' + format, 'NotSupportedError');
          });
        },
        exportKey: function(format, key) {
          return Promise.resolve().then(function() {
            var bytes = _toUint8Array(key);
            return bytes.buffer;
          });
        },
      };
    } else {
      cryptoObj.getRandomValues = function(array) {
        for (var i = 0; i < array.length; i++) array[i] = Math.floor(Math.random() * 256);
        return array;
      };
      cryptoObj.randomUUID = function() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
          var r = Math.random() * 16 | 0;
          var v = c === 'x' ? r : (r & 0x3 | 0x8);
          return v.toString(16);
        });
      };
      cryptoObj.subtle = {};
    }
    globalThis.crypto = cryptoObj;
  }

  function _toUint8Array(data) {
    if (data instanceof Uint8Array) return data;
    if (data && data.byteLength !== undefined && typeof data.byteLength === 'number') {
      return new Uint8Array(data);
    }
    if (Array.isArray(data)) return new Uint8Array(data);
    if (typeof data === 'string') {
      var bytes = new Uint8Array(data.length);
      for (var i = 0; i < data.length; i++) bytes[i] = data.charCodeAt(i) & 0xff;
      return bytes;
    }
    return new Uint8Array(0);
  }

  // ── QueuingStrategy ──
  if (typeof globalThis.CountQueuingStrategy !== 'function') {
    function CountQueuingStrategy(init) {
      var opts = init || {};
      this.highWaterMark = opts.highWaterMark !== undefined ? Number(opts.highWaterMark) : 1;
    }
    CountQueuingStrategy.prototype.size = function(chunk) { return 1; };
    globalThis.CountQueuingStrategy = CountQueuingStrategy;
  }
  if (typeof globalThis.ByteLengthQueuingStrategy !== 'function') {
    function ByteLengthQueuingStrategy(init) {
      var opts = init || {};
      this.highWaterMark = opts.highWaterMark !== undefined ? Number(opts.highWaterMark) : 0;
    }
    ByteLengthQueuingStrategy.prototype.size = function(chunk) {
      if (chunk === null || chunk === undefined) return 0;
      return chunk.byteLength !== undefined ? chunk.byteLength : (chunk.length || 0);
    };
    globalThis.ByteLengthQueuingStrategy = ByteLengthQueuingStrategy;
  }

  // ── ReadableStream（Web 标准，支持 start/pull/cancel + enqueue/close/error） ──
  if (typeof globalThis.ReadableStream !== 'function') {
    function ReadableStream(underlyingSource, strategy) {
      var source = underlyingSource || {};
      this._controller = new ReadableStreamDefaultController(this, source);
      this._strategy = strategy || new CountQueuingStrategy();
      this._locked = false;
      this._disturbed = false;
      this._reader = null;
      this._ended = false;
      this._error = undefined;
      this._queue = [];
      this._closeRequested = false;
      this._pendingReads = [];
      if (typeof source.start === 'function') {
        source.start(this._controller);
      }
    }
    ReadableStream.prototype.getReader = function() {
      if (this._locked) throw new TypeError('ReadableStream is already locked');
      this._locked = true;
      var self = this;
      var closed = false;
      var reader = {
        get closed() { return closed ? Promise.resolve() : new Promise(function(res) { reader._closeResolve = res; }); },
        get locked() { return self._locked; },
        read: function() {
          self._disturbed = true;
          return new Promise(function(resolve, reject) {
            if (self._error !== undefined) { reject(self._error); return; }
            if (self._queue.length > 0) {
              resolve({ value: self._queue.shift(), done: false });
              return;
            }
            if (self._ended) {
              resolve({ value: undefined, done: true });
              return;
            }
            // pull 尝试填充
            if (typeof self._controller._source.pull === 'function') {
              try { self._controller._source.pull(self._controller); } catch (e) {}
            }
            if (self._queue.length > 0) {
              resolve({ value: self._queue.shift(), done: false });
              return;
            }
            if (self._ended) {
              resolve({ value: undefined, done: true });
              return;
            }
            self._pendingReads.push({ resolve: resolve, reject: reject });
          });
        },
        cancel: function(reason) {
          self._locked = false;
          closed = true;
          if (reader._closeResolve) reader._closeResolve();
          if (typeof self._controller._source.cancel === 'function') {
            try { self._controller._source.cancel(reason); } catch (e) {}
          }
          return Promise.resolve();
        },
        releaseLock: function() {
          self._locked = false;
        },
      };
      this._reader = reader;
      return reader;
    };
    ReadableStream.prototype.cancel = function(reason) {
      var reader = this.getReader();
      return reader.cancel(reason);
    };
    ReadableStream.prototype[Symbol.asyncIterator] = function() {
      var reader = this.getReader();
      return {
        next: function() { return reader.read(); },
        return: function() { return reader.cancel(); },
      };
    };
    ReadableStream.prototype.pipeTo = function(dest) {
      var reader = this.getReader();
      var writer = dest.getWriter();
      function pump() {
        return reader.read().then(function(result) {
          if (result.done) { writer.close(); return; }
          return writer.write(result.value).then(pump);
        });
      }
      return pump();
    };
    ReadableStream.prototype.pipeThrough = function(transform) {
      var output = transform.readable;
      var writer = transform.writable.getWriter();
      var reader = this.getReader();
      function pump() {
        return reader.read().then(function(result) {
          if (result.done) { writer.close(); return; }
          return writer.write(result.value).then(pump);
        });
      }
      pump();
      return output;
    };
    Object.defineProperty(ReadableStream.prototype, 'locked', {
      get: function() { return this._locked; }
    });

    function ReadableStreamDefaultController(stream, source) {
      this._stream = stream;
      this._source = source;
    }
    ReadableStreamDefaultController.prototype.enqueue = function(chunk) {
      var stream = this._stream;
      if (stream._closeRequested) throw new TypeError('Cannot enqueue after close');
      stream._queue.push(chunk);
      // 满足等待中的 read
      if (stream._pendingReads.length > 0) {
        var pending = stream._pendingReads.shift();
        if (pending.resolve) pending.resolve({ value: stream._queue.shift(), done: false });
      }
    };
    ReadableStreamDefaultController.prototype.close = function() {
      var stream = this._stream;
      if (stream._closeRequested) return;
      stream._closeRequested = true;
      stream._ended = true;
      while (stream._pendingReads.length > 0) {
        var pending = stream._pendingReads.shift();
        if (pending.resolve) pending.resolve({ value: undefined, done: true });
      }
      var reader = stream._reader;
      if (reader && reader._closeResolve) reader._closeResolve();
    };
    ReadableStreamDefaultController.prototype.error = function(e) {
      var stream = this._stream;
      stream._error = e;
      stream._ended = true;
      while (stream._pendingReads.length > 0) {
        var pending = stream._pendingReads.shift();
        if (pending.reject) pending.reject(e);
      }
    };
    ReadableStreamDefaultController.prototype.byobRequest = null;
    Object.defineProperty(ReadableStreamDefaultController.prototype, 'desiredSize', {
      get: function() { return 1; }
    });

    globalThis.ReadableStream = ReadableStream;
    globalThis.ReadableStreamDefaultController = ReadableStreamDefaultController;
  }

  // ── WritableStream ──
  if (typeof globalThis.WritableStream !== 'function') {
    function WritableStream(underlyingSink, strategy) {
      var sink = underlyingSink || {};
      this._sink = sink;
      this._locked = false;
      this._closed = false;
      this._closedPromise = null;
      this._writeQueue = [];
      if (typeof sink.start === 'function') {
        sink.start({});
      }
    }
    WritableStream.prototype.getWriter = function() {
      if (this._locked) throw new TypeError('WritableStream is already locked');
      this._locked = true;
      var self = this;
      var readyResolve = null;
      return {
        get closed() {
          return self._closedPromise || (self._closedPromise = new Promise(function(res) {
            self._closedResolve = res;
          }));
        },
        get ready() {
          return Promise.resolve();
        },
        get desiredSize() { return 1; },
        write: function(chunk) {
          var self2 = this;
          if (typeof self._sink.write === 'function') {
            try {
              var result = self._sink.write(chunk);
              if (result && typeof result.then === 'function') return result;
            } catch (e) {
              return Promise.reject(e);
            }
          }
          return Promise.resolve();
        },
        close: function() {
          self._locked = false;
          self._closed = true;
          if (typeof self._sink.close === 'function') {
            try { self._sink.close(); } catch (e) {}
          }
          if (self._closedResolve) self._closedResolve();
          return Promise.resolve();
        },
        abort: function(reason) {
          self._locked = false;
          self._closed = true;
          if (typeof self._sink.abort === 'function') {
            try { self._sink.abort(reason); } catch (e) {}
          }
          return Promise.resolve();
        },
        releaseLock: function() { self._locked = false; },
      };
    };
    WritableStream.prototype.close = function() {
      var writer = this.getWriter();
      return writer.close();
    };
    WritableStream.prototype.abort = function(reason) {
      var writer = this.getWriter();
      return writer.abort(reason);
    };
    Object.defineProperty(WritableStream.prototype, 'locked', {
      get: function() { return this._locked; }
    });
    globalThis.WritableStream = WritableStream;
    globalThis.WritableStreamDefaultWriter = WritableStream.prototype.getWriter;
    globalThis.WritableStreamDefaultController = function() {};
  }

  // ── TransformStream ──
  if (typeof globalThis.TransformStream !== 'function') {
    function TransformStream(transformer, writableStrategy, readableStrategy) {
      var transform = transformer || {};
      var self = this;
      this._readable = new ReadableStream({
        start: function(controller) {
          self._readableController = controller;
        },
      });
      this._writable = new WritableStream({
        start: function() {
          if (typeof transform.start === 'function') transform.start(self._readableController);
        },
        write: function(chunk) {
          if (typeof transform.transform === 'function') {
            var result = transform.transform(chunk, self._readableController);
            if (result && typeof result.then === 'function') return result;
          } else {
            self._readableController.enqueue(chunk);
          }
          return Promise.resolve();
        },
        close: function() {
          if (typeof transform.flush === 'function') {
            transform.flush(self._readableController);
          }
          self._readableController.close();
        },
      });
    }
    Object.defineProperty(TransformStream.prototype, 'readable', {
      get: function() { return this._readable; },
      configurable: true,
    });
    Object.defineProperty(TransformStream.prototype, 'writable', {
      get: function() { return this._writable; },
      configurable: true,
    });
    globalThis.TransformStream = TransformStream;
    globalThis.TransformStreamDefaultController = function() {};
  }

  // ── TextEncoderStream / TextDecoderStream ──
  if (typeof globalThis.TextEncoderStream !== 'function') {
    function TextEncoderStream() {
      var self = this;
      this._ts = new TransformStream({
        transform: function(chunk, controller) {
          var str = typeof chunk === 'string' ? chunk : String(chunk);
          controller.enqueue(_utf8EncodeBytes(str));
        },
      });
    }
    Object.defineProperty(TextEncoderStream.prototype, 'readable', { get: function() { return this._ts.readable; } });
    Object.defineProperty(TextEncoderStream.prototype, 'writable', { get: function() { return this._ts.writable; } });
    Object.defineProperty(TextEncoderStream.prototype, 'encoding', { get: function() { return 'utf-8'; } });
    globalThis.TextEncoderStream = TextEncoderStream;
  }
  if (typeof globalThis.TextDecoderStream !== 'function') {
    function TextDecoderStream(label, options) {
      var self = this;
      var dec = null;
      try { dec = new TextDecoder(label, options); } catch (e) { dec = new TextDecoder(); }
      this._decoder = dec;
      this._ts = new TransformStream({
        transform: function(chunk, controller) {
          var u8 = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
          controller.enqueue(dec.decode(u8, { stream: true }));
        },
        flush: function(controller) {
          var tail = dec.decode();
          if (tail) controller.enqueue(tail);
        },
      });
    }
    Object.defineProperty(TextDecoderStream.prototype, 'readable', { get: function() { return this._ts.readable; } });
    Object.defineProperty(TextDecoderStream.prototype, 'writable', { get: function() { return this._ts.writable; } });
    Object.defineProperty(TextDecoderStream.prototype, 'encoding', { get: function() { return this._decoder.encoding; } });
    globalThis.TextDecoderStream = TextDecoderStream;
  }

  function _utf8EncodeBytes(str) {
    var bytes = [];
    for (var i = 0; i < str.length; i++) {
      var cc = str.charCodeAt(i);
      if (cc < 0x80) bytes.push(cc);
      else if (cc < 0x800) { bytes.push(0xc0 | (cc >> 6)); bytes.push(0x80 | (cc & 0x3f)); }
      else if (cc >= 0xd800 && cc <= 0xdbff) {
        i++;
        var cc2 = str.charCodeAt(i) || 0;
        var cp = ((cc - 0xd800) << 10) + (cc2 - 0xdc00) + 0x10000;
        bytes.push(0xf0 | (cp >> 18)); bytes.push(0x80 | ((cp >> 12) & 0x3f));
        bytes.push(0x80 | ((cp >> 6) & 0x3f)); bytes.push(0x80 | (cp & 0x3f));
      } else {
        bytes.push(0xe0 | (cc >> 12)); bytes.push(0x80 | ((cc >> 6) & 0x3f)); bytes.push(0x80 | (cc & 0x3f));
      }
    }
    return new Uint8Array(bytes);
  }

  // ── CompressionStream / DecompressionStream（复用 koss:zlib） ──
  function _getZlibModule() {
    try { return require('koss:zlib'); } catch (e) { return null; }
  }
  if (typeof globalThis.CompressionStream !== 'function') {
    function CompressionStream(format) {
      var self = this;
      this._format = String(format || 'gzip').toLowerCase().replace('_', '-');
      this._chunks = [];
      this._ts = new TransformStream({
        transform: function(chunk, controller) {
          var u8 = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
          self._chunks.push(u8);
        },
        flush: function(controller) {
          var zlib = _getZlibModule();
          if (!zlib) { controller.error(new Error('zlib not available')); return; }
          var total = 0;
          for (var i = 0; i < self._chunks.length; i++) total += self._chunks[i].length;
          var input = new Uint8Array(total);
          var offset = 0;
          for (var j = 0; j < self._chunks.length; j++) { input.set(self._chunks[j], offset); offset += self._chunks[j].length; }
          var out;
          if (self._format === 'gzip') out = zlib.gzipSync(input);
          else if (self._format === 'deflate') out = zlib.deflateSync(input);
          else if (self._format === 'deflate-raw') out = zlib.deflateRawSync ? zlib.deflateRawSync(input) : zlib.deflateSync(input);
          else if (self._format === 'brotli') out = zlib.brotliCompressSync ? zlib.brotliCompressSync(input) : null;
          else { controller.error(new Error('Unsupported compression format: ' + self._format)); return; }
          if (out) controller.enqueue(out instanceof Uint8Array ? out : new Uint8Array(out));
        },
      });
    }
    Object.defineProperty(CompressionStream.prototype, 'readable', { get: function() { return this._ts.readable; } });
    Object.defineProperty(CompressionStream.prototype, 'writable', { get: function() { return this._ts.writable; } });
    globalThis.CompressionStream = CompressionStream;
  }
  if (typeof globalThis.DecompressionStream !== 'function') {
    function DecompressionStream(format) {
      var self = this;
      this._format = String(format || 'gzip').toLowerCase().replace('_', '-');
      this._chunks = [];
      this._ts = new TransformStream({
        transform: function(chunk, controller) {
          var u8 = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
          self._chunks.push(u8);
        },
        flush: function(controller) {
          var zlib = _getZlibModule();
          if (!zlib) { controller.error(new Error('zlib not available')); return; }
          var total = 0;
          for (var i = 0; i < self._chunks.length; i++) total += self._chunks[i].length;
          var input = new Uint8Array(total);
          var offset = 0;
          for (var j = 0; j < self._chunks.length; j++) { input.set(self._chunks[j], offset); offset += self._chunks[j].length; }
          var out;
          if (self._format === 'gzip') out = zlib.gunzipSync(input);
          else if (self._format === 'deflate') out = zlib.inflateSync(input);
          else if (self._format === 'deflate-raw') out = zlib.inflateRawSync ? zlib.inflateRawSync(input) : zlib.inflateSync(input);
          else if (self._format === 'brotli') out = zlib.brotliDecompressSync ? zlib.brotliDecompressSync(input) : null;
          else { controller.error(new Error('Unsupported compression format: ' + self._format)); return; }
          if (out) controller.enqueue(out instanceof Uint8Array ? out : new Uint8Array(out));
        },
      });
    }
    Object.defineProperty(DecompressionStream.prototype, 'readable', { get: function() { return this._ts.readable; } });
    Object.defineProperty(DecompressionStream.prototype, 'writable', { get: function() { return this._ts.writable; } });
    globalThis.DecompressionStream = DecompressionStream;
  }

  // ── MessageChannel / MessagePort（同线程消息传递） ──
  if (typeof globalThis.MessageChannel !== 'function') {
    function MessagePort() {
      this._listeners = {};
      this._started = false;
      this.onmessage = null;
      this.onmessageerror = null;
    }
    MessagePort.prototype.postMessage = function(data, transfer) {
      var self = this;
      var other = this._other;
      if (!other) return;
      var cloned;
      try { cloned = globalThis.structuredClone ? globalThis.structuredClone(data) : data; }
      catch (e) { cloned = data; }
      // 异步投递
      setTimeout(function() {
        other._dispatch(new MessageEvent('message', { data: cloned }));
      }, 0);
    };
    MessagePort.prototype.start = function() { this._started = true; };
    MessagePort.prototype.close = function() {
      this._closed = true;
      this._other = null;
    };
    MessagePort.prototype._dispatch = function(event) {
      if (this._closed) return;
      if (typeof this.onmessage === 'function') this.onmessage(event);
      var arr = this._listeners['message'];
      if (arr) {
        var copy = arr.slice();
        for (var i = 0; i < copy.length; i++) copy[i].call(this, event);
      }
    };
    MessagePort.prototype.addEventListener = function(type, listener, options) {
      if (typeof listener !== 'function') return;
      var key = String(type);
      if (!this._listeners[key]) this._listeners[key] = [];
      this._listeners[key].push(listener);
    };
    MessagePort.prototype.removeEventListener = function(type, listener) {
      var key = String(type);
      var arr = this._listeners[key];
      if (!arr) return;
      for (var i = 0; i < arr.length; i++) {
        if (arr[i] === listener) { arr.splice(i, 1); break; }
      }
    };
    Object.defineProperty(MessagePort.prototype, 'onmessage', {
      get: function() { return this._onmessage; },
      set: function(fn) { this._onmessage = fn; },
      configurable: true,
    });
    globalThis.MessagePort = MessagePort;

    function MessageChannel() {
      this.port1 = new MessagePort();
      this.port2 = new MessagePort();
      this.port1._other = this.port2;
      this.port2._other = this.port1;
    }
    globalThis.MessageChannel = MessageChannel;
  }

  // ── BroadcastChannel（同进程广播） ──
  if (typeof globalThis.BroadcastChannel !== 'function') {
    var __broadcastChannels = {};
    function BroadcastChannel(name) {
      EventTarget.call(this);
      this.name = String(name);
      this._closed = false;
      if (!__broadcastChannels[this.name]) __broadcastChannels[this.name] = [];
      __broadcastChannels[this.name].push(this);
      this.onmessage = null;
      this.onmessageerror = null;
    }
    BroadcastChannel.prototype = Object.create(EventTarget.prototype);
    BroadcastChannel.prototype.constructor = BroadcastChannel;
    BroadcastChannel.prototype.postMessage = function(data) {
      var self = this;
      var channels = __broadcastChannels[this.name] || [];
      var cloned;
      try { cloned = globalThis.structuredClone ? globalThis.structuredClone(data) : data; }
      catch (e) { cloned = data; }
      setTimeout(function() {
        for (var i = 0; i < channels.length; i++) {
          if (channels[i] !== self && !channels[i]._closed) {
            var event = new MessageEvent('message', { data: cloned });
            channels[i].dispatchEvent(event);
            if (typeof channels[i].onmessage === 'function') channels[i].onmessage(event);
          }
        }
      }, 0);
    };
    BroadcastChannel.prototype.close = function() {
      this._closed = true;
      var arr = __broadcastChannels[this.name];
      if (arr) {
        var idx = arr.indexOf(this);
        if (idx !== -1) arr.splice(idx, 1);
      }
    };
    Object.defineProperty(BroadcastChannel.prototype, 'onmessage', {
      get: function() { return this._onmessage; },
      set: function(fn) { this._onmessage = fn; },
      configurable: true,
    });
    globalThis.BroadcastChannel = BroadcastChannel;
  }

  // ── Storage / localStorage / sessionStorage ──
  if (typeof globalThis.Storage !== 'function') {
    function Storage() {
      this._store = {};
    }
    Storage.prototype.getItem = function(key) {
      return Object.prototype.hasOwnProperty.call(this._store, key) ? this._store[key] : null;
    };
    Storage.prototype.setItem = function(key, value) {
      this._store[key] = String(value);
    };
    Storage.prototype.removeItem = function(key) {
      delete this._store[key];
    };
    Storage.prototype.clear = function() {
      this._store = {};
    };
    Storage.prototype.key = function(index) {
      var keys = Object.keys(this._store);
      return index >= 0 && index < keys.length ? keys[index] : null;
    };
    Object.defineProperty(Storage.prototype, 'length', {
      get: function() { return Object.keys(this._store).length; }
    });
    globalThis.Storage = Storage;

    if (typeof globalThis.localStorage === 'undefined') {
      globalThis.localStorage = new Storage();
    }
    if (typeof globalThis.sessionStorage === 'undefined') {
      globalThis.sessionStorage = new Storage();
    }
  }

  // ── console 补全（table/trace/count/countReset/clear/assert） ──
  if (typeof console !== 'undefined' && typeof console.table !== 'function') {
    console.table = function(data) {
      if (data === null || data === undefined) return console.log(data);
      if (Array.isArray(data)) {
        for (var i = 0; i < data.length; i++) {
          if (typeof data[i] === 'object' && data[i] !== null) {
            console.log(i, JSON.stringify(data[i]));
          } else {
            console.log(i, data[i]);
          }
        }
      } else if (typeof data === 'object') {
        console.log(JSON.stringify(data, null, 2));
      } else {
        console.log(data);
      }
    };
  }
  if (typeof console !== 'undefined' && typeof console.trace !== 'function') {
    console.trace = function() {
      var args = Array.prototype.slice.call(arguments);
      var msg = args.join(' ');
      try {
        console.error(msg + '\n' + new Error('trace').stack);
      } catch (e) {
        console.error(msg);
      }
    };
  }
  if (typeof console !== 'undefined' && typeof console.count !== 'function') {
    var __consoleCounts = {};
    console.count = function(label) {
      var key = label === undefined ? 'default' : String(label);
      __consoleCounts[key] = (__consoleCounts[key] || 0) + 1;
      console.log(key + ': ' + __consoleCounts[key]);
    };
    console.countReset = function(label) {
      var key = label === undefined ? 'default' : String(label);
      delete __consoleCounts[key];
    };
  }
  if (typeof console !== 'undefined' && typeof console.clear !== 'function') {
    console.clear = function() { /* 嵌入式环境无终端清屏 */ };
  }
  if (typeof console !== 'undefined' && typeof console.assert !== 'function') {
    console.assert = function(condition) {
      if (!condition) {
        var args = Array.prototype.slice.call(arguments, 1);
        var msg = 'Assertion failed';
        if (args.length > 0) {
          var formatted = [];
          for (var i = 0; i < args.length; i++) {
            formatted.push(typeof args[i] === 'object' ? JSON.stringify(args[i]) : String(args[i]));
          }
          msg += ': ' + formatted.join(' ');
        }
        console.error(msg);
      }
    };
  }
  if (typeof console !== 'undefined' && typeof console.groupCollapsed !== 'function') {
    console.groupCollapsed = console.group || function() {};
  }
  if (typeof console !== 'undefined' && typeof console.groupEnd !== 'function') {
    console.groupEnd = function() {};
  }
  if (typeof console !== 'undefined' && typeof console.timeLog !== 'function') {
    console.timeLog = function(label) { console.log((label === undefined ? 'default' : String(label)) + ': ' + Date.now() + 'ms'); };
  }
  if (typeof console !== 'undefined' && typeof console.dirxml !== 'function') {
    console.dirxml = console.dir || console.log;
  }

})(globalThis);
