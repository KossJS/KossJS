'use strict';

// Simple URL implementation when globalThis.URL is not available
function parseURLImpl(urlStr, baseStr) {
    // Simple URL parser: protocol://host:port/path?query#hash
    var result = {
        href: urlStr,
        protocol: '',
        host: '',
        hostname: '',
        port: '',
        pathname: '/',
        search: '',
        hash: '',
    };

    var remaining = urlStr;

    // Extract hash
    var hashIdx = remaining.indexOf('#');
    if (hashIdx >= 0) {
        result.hash = remaining.substring(hashIdx);
        remaining = remaining.substring(0, hashIdx);
    }

    // Extract search/query
    var searchIdx = remaining.indexOf('?');
    if (searchIdx >= 0) {
        result.search = remaining.substring(searchIdx);
        remaining = remaining.substring(0, searchIdx);
    }

    // Extract protocol
    var protoIdx = remaining.indexOf('://');
    if (protoIdx >= 0) {
        result.protocol = remaining.substring(0, protoIdx + 1); // "https:"
        remaining = remaining.substring(protoIdx + 3);
    }

    // Extract path
    var pathIdx = remaining.indexOf('/');
    var authority = remaining;
    if (pathIdx >= 0) {
        authority = remaining.substring(0, pathIdx);
        result.pathname = remaining.substring(pathIdx);
    }

    // Extract host:port
    var portIdx = authority.lastIndexOf(':');
    if (portIdx >= 0) {
        result.hostname = authority.substring(0, portIdx);
        result.port = authority.substring(portIdx + 1);
    } else {
        result.hostname = authority;
    }
    result.host = authority;

    // Rebuild href
    var href = result.protocol + '//' + result.host + result.pathname;
    if (result.search) href += result.search;
    if (result.hash) href += result.hash;
    result.href = href;

    // resolve against base
    if (baseStr && !result.protocol) {
        var base = parseURLImpl(baseStr, null);
        if (result.href.startsWith('/')) {
            result.href = base.protocol + '//' + base.host + result.href;
        } else {
            result.href = base.href.replace(/\/[^/]*$/, '/') + result.href;
        }
        // Re-parse
        return parseURLImpl(result.href, null);
    }

    return result;
}

// Simple URLSearchParams
function URLSearchParamsImpl(init) {
    var params = this;
    params._entries = [];
    params._onChange = arguments[1] || null;

    if (typeof init === 'string') {
        var pairs = init.replace(/^[?]/, '').split('&');
        for (var i = 0; i < pairs.length; i++) {
            if (pairs[i]) {
                var eq = pairs[i].indexOf('=');
                if (eq >= 0) {
                    params._entries.push([decodeURIComponent(pairs[i].substring(0, eq)), decodeURIComponent(pairs[i].substring(eq + 1))]);
                } else {
                    params._entries.push([decodeURIComponent(pairs[i]), '']);
                }
            }
        }
    }
}

function notifyChange(sp) {
    if (typeof sp._onChange === 'function') sp._onChange(sp.toString());
}

URLSearchParamsImpl.prototype.get = function(name) {
    for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i][0] === name) return this._entries[i][1];
    }
    return null;
};

URLSearchParamsImpl.prototype.getAll = function(name) {
    var result = [];
    for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i][0] === name) result.push(this._entries[i][1]);
    }
    return result;
};

URLSearchParamsImpl.prototype.set = function(name, value) {
    for (var i = this._entries.length - 1; i >= 0; i--) {
        if (this._entries[i][0] === name) this._entries.splice(i, 1);
    }
    this._entries.push([name, String(value)]);
    notifyChange(this);
};

URLSearchParamsImpl.prototype.append = function(name, value) {
    this._entries.push([name, String(value)]);
    notifyChange(this);
};

URLSearchParamsImpl.prototype.delete = function(name) {
    for (var i = this._entries.length - 1; i >= 0; i--) {
        if (this._entries[i][0] === name) this._entries.splice(i, 1);
    }
    notifyChange(this);
};

URLSearchParamsImpl.prototype.has = function(name) {
    for (var i = 0; i < this._entries.length; i++) {
        if (this._entries[i][0] === name) return true;
    }
    return false;
};

Object.defineProperty(URLSearchParamsImpl.prototype, 'toString', {
    writable: true, configurable: true,
    value: function() {
        return this._entries.map(function(e) {
            return encodeURIComponent(e[0]) + '=' + encodeURIComponent(e[1]);
        }).join('&');
    }
});

URLSearchParamsImpl.prototype.entries = function() {
    return this._entries[Symbol.iterator]();
};

function URLConstructor(url, base) {
    if (!(this instanceof URLConstructor)) {
        return new URLConstructor(url, base);
    }
    var parsed = parseURLImpl(url, base);
    this.href = parsed.href;
    this.protocol = parsed.protocol;
    this.host = parsed.host;
    this.hostname = parsed.hostname;
    this.port = parsed.port;
    this.pathname = parsed.pathname;
    this.hash = parsed.hash;
    var self = this;
    this.searchParams = new URLSearchParamsImpl(parsed.search, function(query) {
        self.search = query ? '?' + query : '';
    });
    this.search = parsed.search;
}

Object.defineProperty(URLConstructor.prototype, 'toString', {
    writable: true, configurable: true,
    value: function() {
        return this.href;
    }
});

URLConstructor.prototype.toJSON = function() {
    return this.href;
};

function urlParse(urlStr) {
    return new URLConstructor(urlStr);
}

function urlResolve(from, to) {
    return new URLConstructor(to, from).href;
}

function urlFormat(urlObj) {
    if (typeof urlObj === 'string') return urlObj;
    if (urlObj && urlObj.href) return urlObj.href;
    if (urlObj && urlObj.protocol && urlObj.host) {
        var s = urlObj.protocol + '//' + urlObj.host;
        if (urlObj.pathname) s += urlObj.pathname;
        else s += '/';
        if (urlObj.search) s += urlObj.search;
        if (urlObj.hash) s += urlObj.hash;
        return s;
    }
    return '';
}

function domainToASCII(domain) {
    return domain;
}

function domainToUnicode(domain) {
    return domain;
}

function urlToHttpOptions(url) {
    var options = {
        protocol: url.protocol,
        hostname: url.hostname,
        port: url.port,
        path: (url.pathname || '') + (url.search || ''),
        method: 'GET',
    };
    if (url.username) {
        options.auth = url.username + (url.password ? ':' + url.password : '');
    }
    return options;
}

module.exports = {
    URL: URLConstructor,
    URLConstructor,
    parse: urlParse,
    resolve: urlResolve,
    format: urlFormat,
    domainToASCII,
    domainToUnicode,
    urlToHttpOptions,
};
