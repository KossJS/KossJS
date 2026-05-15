from kossjs_interface import KossJS

koss = KossJS('./kossjs.dll')

r = koss.eval("typeof require('fs')")
print("type:", repr(r))

r = koss.eval("""
var m = require('fs');
var info = {
    type: typeof m,
    isNull: m === null,
    keys: Object.keys(m),
    ownKeys: Object.getOwnPropertyNames(m),
};
JSON.stringify(info)
""")
print("fs info:", r)

r = koss.eval("""
var path = require('path');
var info = {
    keys: Object.keys(path),
    ownKeys: Object.getOwnPropertyNames(path),
};
JSON.stringify(info)
""")
print("path info:", r)

# Test if basic require works with a simpler module
r = koss.eval("""
(function() {
    var m = { exports: {} };
    m.exports = { hello: 42, world: function() {} };
    m.exports.extra = true;
    return JSON.stringify({
        keys: Object.keys(m.exports),
        hello: m.exports.hello,
        extra: m.exports.extra
    });
})()
""")
print("basic test:", r)

# Check what __koss_load_module returns
r = koss.eval("""
var result = __koss_load_module('fs');
JSON.stringify({
    hasResult: typeof result !== 'undefined' && result !== null,
    type: typeof result,
    preview: typeof result === 'string' ? result.substring(0, 200) : String(result)
})
""")
print("loader result:", r)
