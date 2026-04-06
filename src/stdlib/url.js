'use strict';

const { URL: URLPolyfill, URLSearchParams } = globalThis;

let _URL = globalThis.URL;
if (!_URL) _URL = URLPolyfill;

function URLConstructor(url, base) {
    if (this instanceof URLConstructor) {
        const u = base ? new _URL(url, base) : new _URL(url);
        this.href = u.href;
        this.origin = u.origin;
        this.protocol = u.protocol;
        this.username = u.username;
        this.password = u.password;
        this.host = u.host;
        this.hostname = u.hostname;
        this.port = u.port;
        this.pathname = u.pathname;
        this.search = u.search;
        this.searchParams = u.searchParams;
        this.hash = u.hash;
    } else {
        return new _URL(url, base);
    }
}

URLConstructor.prototype.toString = function() {
    return this.href;
};

URLConstructor.prototype.toJSON = function() {
    return this.href;
};

function urlParse(urlStr) {
    return new URLConstructor(urlStr);
}

function urlResolve(from, to) {
    try {
        return new URLConstructor(to, from).href;
    } catch (e) {
        return to;
    }
}

function urlFormat(urlObj) {
    if (typeof urlObj === 'string') {
        return urlObj;
    }
    return urlObj.href;
}

function domainToASCII(domain) {
    return domain;
}

function domainToUnicode(domain) {
    return domain;
}

function urlToHttpOptions(url) {
    const options = {
        protocol: url.protocol,
        hostname: url.hostname,
        port: url.port,
        path: url.pathname + url.search,
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
