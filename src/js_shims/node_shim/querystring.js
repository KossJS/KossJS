// koss:node/querystring - Node.js querystring module (L3)

function stringifyPrimitive(v) {
  if (typeof v === 'string') return v;
  if (typeof v === 'number' && isFinite(v)) return '' + v;
  return '';
}

function stringify(obj, sep, eq, options) {
  sep = sep || '&';
  eq = eq || '=';
  const encode = options?.encodeURIComponent || globalThis.encodeURIComponent;

  const keys = Object.keys(obj);
  const result = [];
  for (const key of keys) {
    const value = obj[key];
    if (Array.isArray(value)) {
      for (const v of value) {
        result.push(encode(stringifyPrimitive(key)) + eq + encode(stringifyPrimitive(v)));
      }
    } else {
      result.push(encode(stringifyPrimitive(key)) + eq + encode(stringifyPrimitive(value)));
    }
  }
  return result.join(sep);
}

function parse(str, sep, eq, options) {
  sep = sep || '&';
  eq = eq || '=';
  const decode = options?.decodeURIComponent || globalThis.decodeURIComponent;
  const maxKeys = options?.maxKeys !== undefined ? options.maxKeys : 1000;

  const result = {};
  if (typeof str !== 'string' || str.length === 0) return result;

  const pairs = str.split(sep);
  let keys = 0;
  for (const pair of pairs) {
    if (keys >= maxKeys) break;
    const idx = pair.indexOf(eq);
    let key, val;
    if (idx >= 0) {
      key = pair.slice(0, idx);
      val = pair.slice(idx + 1);
    } else {
      key = pair;
      val = '';
    }
    key = decode(key);
    val = decode(val);
    if (result[key] === undefined) {
      result[key] = val;
    } else if (Array.isArray(result[key])) {
      result[key].push(val);
    } else {
      result[key] = [result[key], val];
    }
    keys++;
  }
  return result;
}

function encode(str) {
  return globalThis.encodeURIComponent(str);
}

function decode(str) {
  return globalThis.decodeURIComponent(str);
}

function unescape(str) {
  return decodeURIComponent(str.replace(/\+/g, ' '));
}

function escape(str) {
  return encodeURIComponent(String(str)).replace(/[!'()*]/g, (c) => '%' + c.charCodeAt(0).toString(16).toUpperCase());
}

module.exports = { stringify, parse, encode, decode, unescape, escape };