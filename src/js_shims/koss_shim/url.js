// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:url — Koss 原生 URL 处理模块
// URL 解析、格式化、路径转换、查询参数处理

var _URLSearchParams = globalThis.URLSearchParams || function URLSearchParams(init) {
  this._pairs = [];
  if (init) {
    if (typeof init === 'string') {
      this._parse(init);
    } else if (Array.isArray(init)) {
      for (var i = 0; i < init.length; i++) {
        var pair = init[i];
        this._pairs.push([String(pair[0]), String(pair[1] || '')]);
      }
    } else if (typeof init === 'object') {
      var keys = Object.keys(init);
      for (var j = 0; j < keys.length; j++) {
        this._pairs.push([keys[j], String(init[keys[j]] || '')]);
      }
    }
  }
};

_URLSearchParams.prototype._parse = function(str) {
  if (str.charAt(0) === '?') str = str.substring(1);
  if (!str) return;
  var parts = str.split('&');
  for (var i = 0; i < parts.length; i++) {
    var idx = parts[i].indexOf('=');
    if (idx === -1) {
      this._pairs.push([decodeURIComponent(parts[i]), '']);
    } else {
      this._pairs.push([
        decodeURIComponent(parts[i].substring(0, idx)),
        decodeURIComponent(parts[i].substring(idx + 1))
      ]);
    }
  }
};

_URLSearchParams.prototype.append = function(key, value) {
  this._pairs.push([String(key), String(value === undefined || value === null ? '' : value)]);
};

_URLSearchParams.prototype.delete = function(key) {
  key = String(key);
  for (var i = this._pairs.length - 1; i >= 0; i--) {
    if (this._pairs[i][0] === key) this._pairs.splice(i, 1);
  }
};

_URLSearchParams.prototype.get = function(key) {
  key = String(key);
  for (var i = 0; i < this._pairs.length; i++) {
    if (this._pairs[i][0] === key) return this._pairs[i][1];
  }
  return null;
};

_URLSearchParams.prototype.getAll = function(key) {
  key = String(key);
  var result = [];
  for (var i = 0; i < this._pairs.length; i++) {
    if (this._pairs[i][0] === key) result.push(this._pairs[i][1]);
  }
  return result;
};

_URLSearchParams.prototype.has = function(key) {
  return this.get(key) !== null;
};

_URLSearchParams.prototype.set = function(key, value) {
  key = String(key);
  value = String(value === undefined || value === null ? '' : value);
  for (var i = 0; i < this._pairs.length; i++) {
    if (this._pairs[i][0] === key) {
      this._pairs[i][1] = value;
      return;
    }
  }
  this._pairs.push([key, value]);
};

_URLSearchParams.prototype.keys = function() {
  var self = this;
  return (function*() {
    for (var i = 0; i < self._pairs.length; i++) yield self._pairs[i][0];
  })();
};

_URLSearchParams.prototype.values = function() {
  var self = this;
  return (function*() {
    for (var i = 0; i < self._pairs.length; i++) yield self._pairs[i][1];
  })();
};

_URLSearchParams.prototype.entries = function() {
  var self = this;
  return (function*() {
    for (var i = 0; i < self._pairs.length; i++) yield [self._pairs[i][0], self._pairs[i][1]];
  })();
};

_URLSearchParams.prototype[Symbol.iterator] = function() {
  return this.entries();
};

_URLSearchParams.prototype.forEach = function(callback, thisArg) {
  for (var i = 0; i < this._pairs.length; i++) {
    callback.call(thisArg, this._pairs[i][1], this._pairs[i][0], this);
  }
};

Object.defineProperty(_URLSearchParams.prototype, 'toString', {
  value: function() {
  var parts = [];
  for (var i = 0; i < this._pairs.length; i++) {
    var key = encodeURIComponent(this._pairs[i][0]);
    var val = encodeURIComponent(this._pairs[i][1]);
    parts.push(key + (val ? '=' + val : ''));
  }
  return parts.join('&');
  },
  writable: true,
  configurable: true,
  enumerable: false,
});

var _URL = globalThis.URL || function URL(input, base) {
  if (typeof input !== 'string') throw new TypeError('URL input must be a string');
  this.href = '';
  this.protocol = '';
  this.auth = '';
  this.hostname = '';
  this.port = '';
  this.pathname = '';
  this.search = '';
  this.hash = '';
  this.searchParams = new _URLSearchParams();
  this._parse(input, base);
};

_URL.prototype._parse = function(input, base) {
  var urlStr = String(input).trim();
  var baseUrl = base ? new _URL(base) : null;
  if (!urlStr && baseUrl) {
    this.href = baseUrl.href;
    this.protocol = baseUrl.protocol;
    this.auth = baseUrl.auth;
    this.hostname = baseUrl.hostname;
    this.port = baseUrl.port;
    this.pathname = baseUrl.pathname;
    this.search = baseUrl.search;
    this.hash = baseUrl.hash;
    this.searchParams = baseUrl.searchParams;
    return;
  }
  var protocolMatch = urlStr.match(/^([a-zA-Z][a-zA-Z0-9+\-.]*):/);
  if (protocolMatch) {
    this.protocol = protocolMatch[1].toLowerCase() + ':';
    urlStr = urlStr.substring(protocolMatch[0].length);
  } else if (baseUrl) {
    this.protocol = baseUrl.protocol;
  } else {
    this.protocol = 'https:';
  }
  var slashSlash = urlStr.indexOf('//');
  if (slashSlash === 0) {
    var authorityEnd = urlStr.indexOf('/', 2);
    var queryStart = urlStr.indexOf('?', 2);
    var hashStart = urlStr.indexOf('#', 2);
    var authorityEndIdx = urlStr.length;
    if (authorityEnd !== -1) authorityEndIdx = Math.min(authorityEndIdx, authorityEnd);
    if (queryStart !== -1) authorityEndIdx = Math.min(authorityEndIdx, queryStart);
    if (hashStart !== -1) authorityEndIdx = Math.min(authorityEndIdx, hashStart);
    var authority = urlStr.substring(2, authorityEndIdx);
    urlStr = urlStr.substring(authorityEndIdx);
    var atIdx = authority.indexOf('@');
    if (atIdx !== -1) {
      this.auth = authority.substring(0, atIdx);
      authority = authority.substring(atIdx + 1);
    } else {
      this.auth = '';
    }
    var colonIdx = authority.lastIndexOf(':');
    if (colonIdx !== -1) {
      this.hostname = authority.substring(0, colonIdx);
      this.port = authority.substring(colonIdx + 1);
    } else {
      this.hostname = authority;
      this.port = '';
    }
  } else if (urlStr.charAt(0) === '/' || urlStr.charAt(0) === '?') {
    if (baseUrl) {
      this.hostname = baseUrl.hostname;
      this.port = baseUrl.port;
      this.auth = baseUrl.auth;
    }
  } else {
    if (baseUrl) {
      this.hostname = baseUrl.hostname;
      this.port = baseUrl.port;
      this.auth = baseUrl.auth;
      if (!urlStr && baseUrl.pathname !== '/') {
        this.pathname = baseUrl.pathname;
      }
    }
  }
  var hashIdx = urlStr.indexOf('#');
  if (hashIdx !== -1) {
    this.hash = urlStr.substring(hashIdx);
    urlStr = urlStr.substring(0, hashIdx);
  } else {
    this.hash = '';
  }
  var questionIdx = urlStr.indexOf('?');
  if (questionIdx !== -1) {
    this.search = urlStr.substring(questionIdx);
    this.searchParams = new _URLSearchParams(urlStr.substring(questionIdx + 1));
    this.pathname = urlStr.substring(0, questionIdx);
  } else {
    this.search = '';
    this.searchParams = new _URLSearchParams();
    this.pathname = urlStr;
  }
  if (this.protocol === 'file:') {
    if (!this.pathname) this.pathname = '/';
  }
  this._syncSearchParams();
  this._buildHref();
};

_URL.prototype._buildHref = function() {
  var href = '';
  if (this.protocol) href += this.protocol + '//';
  if (this.auth) href += this.auth + '@';
  if (this.hostname) href += this.hostname;
  if (this.port) href += ':' + this.port;
  if (this.pathname) href += this.pathname;
  if (this.search) href += this.search;
  if (this.hash) href += this.hash;
  this.href = href;
};

_URL.prototype._syncSearchParams = function() {
  var self = this;
  var origSet = _URLSearchParams.prototype.set;
  var origAppend = _URLSearchParams.prototype.append;
  var origDelete = _URLSearchParams.prototype.delete;
  self.searchParams.set = function(key, value) {
    origSet.call(self.searchParams, key, value);
    self.search = '?' + self.searchParams.toString();
    self._buildHref();
  };
  self.searchParams.append = function(key, value) {
    origAppend.call(self.searchParams, key, value);
    self.search = '?' + self.searchParams.toString();
    self._buildHref();
  };
  self.searchParams.delete = function(key) {
    origDelete.call(self.searchParams, key);
    self.search = self.searchParams.toString() ? '?' + self.searchParams.toString() : '';
    self._buildHref();
  };
};

_URL.prototype.toJSON = function() {
  return this.href;
};

Object.defineProperty(_URL.prototype, 'toString', {
  value: function() {
  return this.href;
  },
  writable: true,
  configurable: true,
  enumerable: false,
});

_URLSearchParams = globalThis.URLSearchParams || _URLSearchParams;
_URL = globalThis.URL || _URL;

function parse(input, base) {
  return new _URL(input, base);
}

function format(url, options) {
  if (typeof url !== 'object' || url === null) throw new TypeError('url must be an object');
  var auth = options && options.auth !== false;
  var unicode = options && options.unicode !== false;
  var fragment = options && options.fragment !== false;
  var search = options && options.search !== false;
  var result = '';
  if (url.protocol) result += url.protocol + '//';
  if (auth && url.auth) result += url.auth + '@';
  if (url.host) result += url.host;
  else if (url.hostname) {
    result += url.hostname;
    if (url.port) result += ':' + url.port;
  }
  if (url.pathname) result += url.pathname;
  if (search && url.search) result += url.search;
  if (fragment && url.hash) result += url.hash;
  return result;
}

function resolve(source, relative) {
  var srcUrl = (source instanceof _URL) ? source : new _URL(source);
  var relUrl = (relative instanceof _URL) ? relative : new _URL(relative, source);
  if (relUrl.protocol && relUrl.hostname) {
    return new _URL(
      relUrl.protocol + '//' + (relUrl.auth || '') + (relUrl.hostname || '') +
      (relUrl.port ? ':' + relUrl.port : '') +
      relUrl.pathname + relUrl.search + relUrl.hash,
      srcUrl.href
    );
  }
  var pathname = relUrl.pathname || '';
  if (pathname.charAt(0) === '/') {
    return new _URL(
      srcUrl.protocol + '//' + (relUrl.auth || srcUrl.auth) +
      (srcUrl.hostname || '') + (srcUrl.port ? ':' + srcUrl.port : '') +
      pathname + relUrl.search + relUrl.hash,
      srcUrl.href
    );
  }
  var srcPath = srcUrl.pathname || '/';
  var base = srcPath.substring(0, srcPath.lastIndexOf('/'));
  var segments = pathname.split('/');
  var resultPath = base;
  for (var i = 0; i < segments.length; i++) {
    if (segments[i] === '..') {
      var lastSlash = resultPath.lastIndexOf('/');
      if (lastSlash !== -1) resultPath = resultPath.substring(0, lastSlash);
      else resultPath = '';
    } else if (segments[i] !== '.') {
      resultPath += '/' + segments[i];
    }
  }
  if (!resultPath) resultPath = '/';
  return new _URL(
    srcUrl.protocol + '//' + (relUrl.auth || srcUrl.auth) +
    srcUrl.hostname + (srcUrl.port ? ':' + srcUrl.port : '') +
    resultPath + relUrl.search + relUrl.hash,
    srcUrl.href
  );
}

function resolveObject(source, relative) {
  return resolve(source, relative);
}

var _asciiLookup = null;
var _unicodeLookup = null;

function _initAsciiTable() {
  if (_asciiLookup) return;
  _asciiLookup = {};
  _unicodeLookup = {};
  for (var i = 0; i < 256; i++) {
    _asciiLookup[i] = String.fromCharCode(i);
  }
}

function domainToASCII(domain) {
  _initAsciiTable();
  if (typeof domain !== 'string') throw new TypeError('domain must be a string');
  try {
    var url = new _URL('https://' + domain);
    return url.hostname;
  } catch (e) {
    throw new TypeError('Invalid domain: ' + domain);
  }
}

function domainToUnicode(domain) {
  _initAsciiTable();
  if (typeof domain !== 'string') throw new TypeError('domain must be a string');
  try {
    var url = new _URL('https://' + domain);
    return decodeURIComponent(url.hostname);
  } catch (e) {
    throw new TypeError('Invalid domain: ' + domain);
  }
}

function fileURLToPath(input) {
  var url = (input instanceof _URL) ? input : new _URL(String(input));
  if (url.protocol !== 'file:') throw new TypeError('URL must be a file URL');
  var pathname = url.pathname;
  if (pathname.charAt(0) === '/' && pathname.length >= 3 &&
      pathname.substring(1, 3).match(/^[A-Za-z]:/)) {
    pathname = pathname.substring(1);
  }
  if (url.hostname) {
    pathname = '\\\\' + url.hostname + pathname;
  }
  return decodeURIComponent(pathname.replace(/\//g, '\\'));
}

function pathToFileURL(path) {
  if (typeof path !== 'string') throw new TypeError('path must be a string');
  var resolved = path.replace(/\\/g, '/');
  if (resolved.charAt(0) === '/' && resolved.length >= 3 &&
      resolved.substring(1, 3).match(/^[A-Za-z]:/)) {
    resolved = resolved.substring(1);
  }
  resolved = encodeURIComponent(resolved.replace(/%/g, '%25'));
  resolved = resolved.replace(/@/g, '%40');
  resolved = resolved.replace(/#/g, '%23');
  resolved = resolved.replace(/\?/g, '%3F');
  resolved = resolved.replace(/=/g, '%3D');
  resolved = resolved.replace(/:/g, '%3A');
  resolved = resolved.replace(/;/g, '%3B');
  resolved = resolved.replace(/,/g, '%2C');
  resolved = resolved.replace(/\(/g, '%28');
  resolved = resolved.replace(/\)/g, '%29');
  resolved = resolved.replace(/\+/g, '%2B');
  resolved = resolved.replace(/!/g, '%21');
  resolved = resolved.replace(/'/g, '%27');
  resolved = resolved.replace(/\*/g, '%2A');
  resolved = resolved.replace(/~/g, '%7E');
  if (resolved.charAt(0) === '/') {
    return new _URL('file://' + resolved);
  }
  return new _URL('file:///' + resolved);
}

function urlToHttpOptions(url) {
  if (typeof url !== 'object' || url === null) throw new TypeError('url must be an object');
  var options = {
    protocol: url.protocol,
    hostname: typeof url.hostname === 'string' ? url.hostname.replace(/\[|]/g, '') : undefined,
    hash: url.hash || undefined,
    search: url.search || undefined,
    pathname: url.pathname || undefined,
    path: (url.pathname || '') + (url.search || '') || undefined,
    href: url.href || undefined,
  };
  if (url.port) options.port = Number(url.port);
  if (url.auth) {
    var authParts = url.auth.split(':');
    options.auth = decodeURIComponent(authParts[0]);
    if (authParts.length > 1) options.auth += ':' + decodeURIComponent(authParts.slice(1).join(':'));
  }
  return options;
}



module.exports = {
  URL: _URL,
  URLSearchParams: _URLSearchParams,
  parse: parse,
  format: format,
  resolve: resolve,
  resolveObject: resolveObject,
  domainToASCII: domainToASCII,
  domainToUnicode: domainToUnicode,
  fileURLToPath: fileURLToPath,
  pathToFileURL: pathToFileURL,
  urlToHttpOptions: urlToHttpOptions,
};
