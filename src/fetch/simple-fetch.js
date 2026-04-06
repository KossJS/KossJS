'use strict';

class AbortError extends Error {
    constructor(message) {
        super(message);
        this.name = 'AbortError';
    }
}

class FetchError extends Error {
    constructor(message, type, cause) {
        super(message);
        this.name = 'FetchError';
        this.type = type;
        this.cause = cause;
    }
}

class Headers {
    constructor(init) {
        this._headers = {};
        
        if (init instanceof Headers) {
            for (const [key, value] of init.entries()) {
                this.set(key, value);
            }
        } else if (init) {
            if (typeof init === 'object') {
                for (const [key, value] of Object.entries(init)) {
                    this.set(key, value);
                }
            } else if (typeof init === 'string') {
                const lines = init.split('\r\n');
                for (const line of lines) {
                    const idx = line.indexOf(':');
                    if (idx > 0) {
                        const key = line.substring(0, idx).trim();
                        const value = line.substring(idx + 1).trim();
                        this.set(key, value);
                    }
                }
            }
        }
    }
    
    get(name) {
        return this._headers[name.toLowerCase()] || null;
    }
    
    set(name, value) {
        this._headers[name.toLowerCase()] = value;
    }
    
    has(name) {
        return name.toLowerCase() in this._headers;
    }
    
    delete(name) {
        delete this._headers[name.toLowerCase()];
    }
    
    forEach(callback, thisArg) {
        for (const [key, value] of Object.entries(this._headers)) {
            callback.call(thisArg, value, key, this);
        }
    }
}

class Response {
    constructor(body, options = {}) {
        this._body = typeof body === 'string' ? body : (body || '');
        this.status = options.status || 200;
        this.statusText = options.statusText || 'OK';
        this.headers = options.headers instanceof Headers ? options.headers : new Headers(options.headers || {});
        this.url = options.url || '';
        this.ok = this.status >= 200 && this.status < 300;
        this.redirected = false;
        this.type = options.type || 'basic';
        this._used = false;
    }
    
    get body() {
        return this._body;
    }
    
    get bodyUsed() {
        return this._used;
    }
    
    clone() {
        if (this._used) {
            throw new Error('Body already used');
        }
        return new Response(this._body, {
            status: this.status,
            statusText: this.statusText,
            headers: new Headers(this.headers),
            url: this.url,
        });
    }
    
    text() {
        if (this._used) {
            throw new Error('Body already used');
        }
        this._used = true;
        return this._body;
    }
    
    json() {
        if (this._used) {
            throw new Error('Body already used');
        }
        this._used = true;
        return JSON.parse(this._body);
    }
}

function buildRequest(url, options) {
    options = options || {};
    return {
        url: url,
        method: options.method || 'GET',
        headers: options.headers || {},
        body: options.body,
    };
}

function fetchSync(url, options) {
    const req = buildRequest(url, options);
    
    let responseJson;
    try {
        responseJson = __fetch(req.url, JSON.stringify({
            method: req.method,
            headers: req.headers,
            body: req.body,
        }));
    } catch (e) {
        throw new FetchError('network error', 'system', e);
    }
    
    let response;
    try {
        response = JSON.parse(responseJson);
    } catch (e) {
        throw new FetchError('invalid response JSON', 'invalid-json', e);
    }
    
    if (!response || typeof response.status === 'undefined') {
        throw new FetchError('invalid response from server', 'invalid-response');
    }
    
    return new Response(response.body || '', {
        status: response.status,
        statusText: response.statusText || '',
        headers: response.headers || {},
        url: req.url,
    });
}

globalThis.Headers = Headers;
globalThis.Response = Response;
globalThis.AbortError = AbortError;
globalThis.FetchError = FetchError;
globalThis.fetch = fetchSync;
globalThis.fetchSync = fetchSync;

export { fetchSync as fetch, Headers, Response, FetchError, AbortError };
