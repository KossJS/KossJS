// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/url - Node.js url module (L3)

const globalURL = globalThis.URL;
const globalURLSearchParams = globalThis.URLSearchParams;

function createURLClass() {
    if (globalURL && typeof globalURL === 'function') {
        return { URL: globalURL, URLSearchParams: globalURLSearchParams };
    }

    class URLSearchParams {
        constructor(init) {
            this._params = [];
            if (init) {
                if (typeof init === 'string') {
                    const str = init.startsWith('?') ? init.slice(1) : init;
                    if (str.length > 0) {
                        str.split('&').forEach(pair => {
                            const eqIdx = pair.indexOf('=');
                            const key = eqIdx >= 0 ? pair.slice(0, eqIdx) : pair;
                            const value = eqIdx >= 0 ? pair.slice(eqIdx + 1) : '';
                            if (key.length > 0) this._params.push([decodeURIComponent(key.replace(/\+/g, ' ')), decodeURIComponent(value.replace(/\+/g, ' '))]);
                        });
                    }
                } else if (Array.isArray(init)) {
                    init.forEach(([key, value]) => this._params.push([String(key), String(value)]));
                } else if (typeof init === 'object') {
                    for (const [key, value] of Object.entries(init)) {
                        this._params.push([String(key), String(value)]);
                    }
                }
            }
        }

        append(name, value) {
            this._params.push([String(name), String(value)]);
        }

        delete(name) {
            const n = String(name);
            this._params = this._params.filter(([k]) => k !== n);
        }

        get(name) {
            const n = String(name);
            const pair = this._params.find(([k]) => k === n);
            return pair ? pair[1] : null;
        }

        getAll(name) {
            const n = String(name);
            return this._params.filter(([k]) => k === n).map(([, v]) => v);
        }

        has(name) {
            const n = String(name);
            return this._params.some(([k]) => k === n);
        }

        set(name, value) {
            this.delete(name);
            this.append(name, value);
        }

        sort() {
            this._params.sort((a, b) => a[0].localeCompare(b[0]));
        }

        toString() {
            return this._params.map(([k, v]) => encodeURIComponent(k) + '=' + encodeURIComponent(v)).join('&');
        }

        forEach(callback, thisArg) {
            this._params.forEach(([key, value]) => callback.call(thisArg, value, key, this));
        }

        keys() {
            return this._params.map(([k]) => k)[Symbol.iterator]();
        }

        values() {
            return this._params.map(([, v]) => v)[Symbol.iterator]();
        }

        entries() {
            return this._params[Symbol.iterator]();
        }

        [Symbol.iterator]() {
            return this.entries();
        }
    }

    class URL {
        constructor(input, base) {
            let urlStr = input;
            if (base && !input.startsWith('http://') && !input.startsWith('https://') && !input.startsWith('file://')) {
                const baseUrl = new URL(base);
                if (input.startsWith('/')) {
                    urlStr = baseUrl.protocol + '//' + baseUrl.host + input;
                } else {
                    const basePath = baseUrl.pathname.replace(/\/[^/]*$/, '/');
                    urlStr = baseUrl.protocol + '//' + baseUrl.host + basePath + input;
                }
            }
            this._parse(urlStr);
            this._syncSearchParams();
        }

        _parse(urlStr) {
            const match = urlStr.match(/^(https?):\/\/([^/:?#]+)(?::(\d+))?([^?#]*)(\?[^#]*)?(#.*)?$/);
            if (!match) {
                throw new TypeError('Invalid URL');
            }
            this.protocol = match[1] + ':';
            this.hostname = match[2];
            this.port = match[3] || '';
            this.pathname = match[4] || '/';
            this.search = match[5] || '';
            this.hash = match[6] || '';
            this.host = this.port ? this.hostname + ':' + this.port : this.hostname;
            this._searchParams = new URLSearchParams(this.search.slice(1));
            this.href = urlStr;
            this.origin = this.protocol + '//' + this.host;
        }

        _syncSearchParams() {
            const self = this;
            const origAppend = this._searchParams.append.bind(this._searchParams);
            const origDelete = this._searchParams.delete.bind(this._searchParams);
            const origSet = this._searchParams.set.bind(this._searchParams);

            function updateSearch() {
                const str = self._searchParams.toString();
                self.search = str.length > 0 ? '?' + str : '';
                self._rebuildHref();
            }

            this._searchParams.append = function(name, value) {
                origAppend(name, value);
                updateSearch();
            };
            this._searchParams.delete = function(name) {
                origDelete(name);
                updateSearch();
            };
            this._searchParams.set = function(name, value) {
                origSet(name, value);
                updateSearch();
            };
        }

        _rebuildHref() {
            this.href = this.protocol + '//' + this.host + this.pathname + this.search + this.hash;
            this.origin = this.protocol + '//' + this.host;
        }

        get searchParams() {
            return this._searchParams;
        }

        toString() {
            return this.href;
        }

        toJSON() {
            return this.href;
        }

        get username() { return ''; }
        get password() { return ''; }
    }

    return { URL, URLSearchParams };
}

const { URL, URLSearchParams } = createURLClass();

function domainToASCII(domain) {
    try { return new URL(`http://${domain}`).hostname; }
    catch { return domain; }
}

function domainToUnicode(domain) { return domain; }

function fileURLToPath(url, options) {
    const urlObj = typeof url === 'string' ? new URL(url) : url;
    if (urlObj.protocol !== 'file:') throw new Error('Must be a file URL');
    let path = urlObj.pathname;
    if (path.startsWith('/')) path = path.slice(1);
    if (process?.platform === 'win32') {
        path = path.replace(/\//g, '\\');
    }
    return decodeURIComponent(path);
}

function pathToFileURL(path) {
    let resolved = path;
    if (process?.platform === 'win32') {
        resolved = '/' + path.replace(/\\/g, '/');
    }
    return new URL('file://' + resolved);
}

function urlToHttpOptions(url) {
    return {
        protocol: url.protocol,
        hostname: url.hostname,
        hash: url.hash,
        search: url.search,
        pathname: url.pathname,
        path: url.pathname + url.search,
        href: url.href,
        port: url.port,
        host: url.host,
        auth: url.username ? `${url.username}:${url.password}` : undefined,
        agent: undefined,
    };
}

function format(urlObj, options) {
    if (urlObj instanceof URL) return urlObj.href;
    let result = '';
    if (urlObj.protocol) {
        result += urlObj.protocol;
        if (!urlObj.protocol.endsWith(':')) result += ':';
        result += '//';
    }
    if (urlObj.auth) result += urlObj.auth + '@';
    result += urlObj.hostname || urlObj.host || '';
    if (urlObj.port) result += ':' + urlObj.port;
    result += urlObj.path || urlObj.pathname || '';
    if (urlObj.search) {
        const s = urlObj.search;
        result += s.startsWith('?') ? s : '?' + s;
    }
    if (urlObj.hash) {
        const h = urlObj.hash;
        result += h.startsWith('#') ? h : '#' + h;
    }
    return result;
}

function parse(urlStr, parseQueryString, slashesDenoteHost) {
    try {
        const url = new URL(urlStr);
        const auth = url.username ? `${url.username}:${url.password}` : null;
        return {
            protocol: url.protocol,
            slashes: true,
            auth,
            host: url.host,
            port: url.port,
            hostname: url.hostname,
            hash: url.hash,
            search: url.search,
            query: parseQueryString ? url.searchParams : (url.search ? url.search.slice(1) : null),
            pathname: url.pathname,
            path: url.pathname + url.search,
            href: url.href,
        };
    } catch {
        return null;
    }
}

function resolve(from, to) {
    try {
        const resolved = new URL(to, from);
        return resolved.href;
    } catch {
        return from;
    }
}

function resolveObject(from, to) {
    return parse(resolve(from, to));
}

module.exports = { URL, URLSearchParams, domainToASCII, domainToUnicode, fileURLToPath, pathToFileURL, urlToHttpOptions, format, parse, resolve, resolveObject };
