'use strict';

const { basename: basename_, dirname: dirname_, extname: extname_, join: join_, parse: parse_, format: format_, isAbsolute: isAbsolute_, normalize: normalize_, resolve: resolve_, relative: relative_, sep: sep_ } = require('path/posix');

function basename(path, ext) {
    const parts = path.split('/');
    let name = parts[parts.length - 1];
    if (ext && name.endsWith(ext)) {
        name = name.substring(0, name.length - ext.length);
    }
    return name;
}

function dirname(path) {
    const parts = path.split('/');
    parts.pop();
    return parts.join('/') || '/';
}

function extname(path) {
    const lastDot = path.lastIndexOf('.');
    const lastSlash = path.lastIndexOf('/');
    if (lastDot > lastSlash && lastDot > 0) {
        return path.substring(lastDot);
    }
    return '';
}

function join(...paths) {
    return paths.filter(p => p).join('/').replace(/\/+/g, '/');
}

function normalize(path) {
    const parts = path.split('/');
    const result = [];
    for (const part of parts) {
        if (part === '..') {
            result.pop();
        } else if (part !== '.' && part) {
            result.push(part);
        }
    }
    let resultPath = result.join('/');
    if (path.startsWith('/')) {
        resultPath = '/' + resultPath;
    }
    return resultPath.replace(/\/+$/, '') || '.';
}

function isAbsolute(path) {
    return path.startsWith('/');
}

function resolve(...paths) {
    let resolved = '/';
    for (const path of paths) {
        if (path.startsWith('/')) {
            resolved = path;
        } else if (path) {
            resolved = resolved + '/' + path;
        }
    }
    return normalize(resolved);
}

function relative(from, to) {
    const fromParts = from.split('/').filter(p => p);
    const toParts = to.split('/').filter(p => p);
    let i = 0;
    while (i < fromParts.length && i < toParts.length && fromParts[i] === toParts[i]) {
        i++;
    }
    const ups = fromParts.slice(i).map(() => '..');
    const downs = toParts.slice(i);
    return [...ups, ...downs].join('/') || '.';
}

function parse(path) {
    const base = basename(path);
    const dir = dirname(path);
    const ext = extname(base);
    const name = ext ? base.substring(0, base.length - ext.length) : base;
    return { root: '/', dir, base, ext, name };
}

function format(obj) {
    if (obj.root) return obj.root + (obj.dir ? obj.dir.substring(1) + '/' : '') + obj.base;
    return obj.dir + '/' + obj.base;
}

function sep() {
    return '/';
}

module.exports = {
    basename,
    dirname,
    extname,
    join,
    normalize,
    isAbsolute,
    resolve,
    relative,
    parse,
    format,
    sep,
    win32: require('path/win32'),
    posix: require('path/posix'),
    delimiter: ':',
};
