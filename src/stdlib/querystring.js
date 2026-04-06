'use strict';

module.exports = {
    parse: parseQueryString,
    stringify: stringifyQueryString,
    encode: stringifyQueryString,
    decode: parseQueryString,
};

function parseQueryString(str) {
    if (!str) return {};
    const params = {};
    const pairs = str.split('&');
    for (const pair of pairs) {
        const idx = pair.indexOf('=');
        if (idx >= 0) {
            const key = decodeURIComponent(pair.substring(0, idx));
            const val = decodeURIComponent(pair.substring(idx + 1));
            params[key] = val;
        } else if (pair) {
            params[decodeURIComponent(pair)] = '';
        }
    }
    return params;
}

function stringifyQueryString(obj) {
    if (!obj) return '';
    const pairs = [];
    for (const key of Object.keys(obj)) {
        pairs.push(encodeURIComponent(key) + '=' + encodeURIComponent(obj[key]));
    }
    return pairs.join('&');
}
