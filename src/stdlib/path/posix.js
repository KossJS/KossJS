'use strict';

// Self-contained posix module (avoid circular dependency with path module)
function basename(path, ext) {
    var parts = path.split('/');
    var name = parts[parts.length - 1];
    if (ext && name.endsWith(ext)) {
        name = name.substring(0, name.length - ext.length);
    }
    return name;
}

function dirname(path) {
    var parts = path.split('/');
    parts.pop();
    return parts.join('/') || '/';
}

function extname(path) {
    var lastDot = path.lastIndexOf('.');
    var lastSlash = path.lastIndexOf('/');
    if (lastDot > lastSlash && lastDot > 0) {
        return path.substring(lastDot);
    }
    return '';
}

function join() {
    var paths = Array.prototype.slice.call(arguments);
    return paths.filter(function(p) { return p; }).join('/').replace(/\/+/g, '/');
}

function normalize(path) {
    var parts = path.split('/');
    var result = [];
    for (var i = 0; i < parts.length; i++) {
        var part = parts[i];
        if (part === '..') {
            result.pop();
        } else if (part !== '.' && part) {
            result.push(part);
        }
    }
    var resultPath = result.join('/');
    if (path.startsWith('/')) {
        resultPath = '/' + resultPath;
    }
    return resultPath.replace(/\/+$/, '') || '.';
}

function isAbsolute(path) {
    return path.startsWith('/');
}

function resolve() {
    var paths = Array.prototype.slice.call(arguments);
    var resolved = '/';
    for (var i = 0; i < paths.length; i++) {
        var p = paths[i];
        if (p.startsWith('/')) {
            resolved = p;
        } else if (p) {
            resolved = resolved + '/' + p;
        }
    }
    return normalize(resolved);
}

function relative(from, to) {
    var fromParts = from.split('/').filter(function(p) { return p; });
    var toParts = to.split('/').filter(function(p) { return p; });
    var i = 0;
    while (i < fromParts.length && i < toParts.length && fromParts[i] === toParts[i]) {
        i++;
    }
    var ups = fromParts.slice(i).map(function() { return '..'; });
    var downs = toParts.slice(i);
    return ups.concat(downs).join('/') || '.';
}

function parse(path) {
    var base = basename(path);
    var dir = dirname(path);
    var ext = extname(base);
    var name = ext ? base.substring(0, base.length - ext.length) : base;
    return { root: '/', dir: dir, base: base, ext: ext, name: name };
}

function format(obj) {
    if (obj.root) return obj.root + (obj.dir ? obj.dir.substring(1) + '/' : '') + obj.base;
    return obj.dir + '/' + obj.base;
}

function sep() {
    return '/';
}

module.exports = {
    basename: basename,
    dirname: dirname,
    extname: extname,
    join: join,
    normalize: normalize,
    isAbsolute: isAbsolute,
    resolve: resolve,
    relative: relative,
    parse: parse,
    format: format,
    sep: sep,
    delimiter: ':',
    win32: null,
    posix: module.exports,
};

