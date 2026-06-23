'use strict';

function lookup(hostname, options, callback) {
  if (typeof options === 'function') {
    callback = options;
    options = {};
  }
  if (typeof callback !== 'function') {
    return lookupPromise(hostname, options);
  }

  var family = (options && options.family) || 0;

  try {
    var resultJson = __koss_dns_lookup(hostname);
    var addresses = JSON.parse(resultJson);

    if (addresses.length === 0) {
      var err = new Error('getaddrinfo ENOTFOUND ' + hostname);
      err.code = 'ENOTFOUND';
      err.errno = -3008;
      err.syscall = 'getaddrinfo';
      process.nextTick(function() { callback(err); });
      return;
    }

    if (family === 4) {
      var v4 = addresses.filter(function(a) { return a.indexOf(':') === -1; });
      if (v4.length > 0) {
        process.nextTick(function() { callback(null, v4[0], 4); });
        return;
      }
    }
    if (family === 6) {
      var v6 = addresses.filter(function(a) { return a.indexOf(':') !== -1; });
      if (v6.length > 0) {
        process.nextTick(function() { callback(null, v6[0], 6); });
        return;
      }
    }

    var addr = addresses[0];
    var fam = addr.indexOf(':') === -1 ? 4 : 6;
    process.nextTick(function() { callback(null, addr, fam); });
  } catch (e) {
    process.nextTick(function() {
      var err = new Error('getaddrinfo ENOTFOUND ' + hostname);
      err.code = 'ENOTFOUND';
      err.syscall = 'getaddrinfo';
      callback(err);
    });
  }
}

function lookupPromise(hostname, options) {
  return new Promise(function(resolve, reject) {
    lookup(hostname, options, function(err, address, family) {
      if (err) return reject(err);
      resolve({ address: address, family: family });
    });
  });
}

function lookupService(address, port, callback) {
  if (typeof callback !== 'function') {
    return Promise.reject(new Error('lookupService requires a callback'));
  }
  process.nextTick(function() {
    callback(null, hostname, port);
  });
}

function resolve(hostname, rrtype, callback) {
  if (typeof rrtype === 'function') {
    callback = rrtype;
    rrtype = 'A';
  }
  if (typeof callback !== 'function') return;

  lookup(hostname, { family: rrtype === 'AAAA' ? 6 : 4 }, function(err, addr, fam) {
    if (err) return callback(err);
    callback(null, [addr]);
  });
}

var promises = {
  lookup: lookupPromise,
  resolve: function(hostname, rrtype) {
    return new Promise(function(resolve, reject) {
      resolve(hostname, rrtype, function(err, addresses) {
        if (err) return reject(err);
        resolve(addresses);
      });
    });
  },
};

module.exports = {
  lookup: lookup,
  lookupService: lookupService,
  resolve: resolve,
  resolve4: function(hostname, options, callback) { return resolve(hostname, 'A', callback); },
  resolve6: function(hostname, options, callback) { return resolve(hostname, 'AAAA', callback); },
  promises: promises,
};
