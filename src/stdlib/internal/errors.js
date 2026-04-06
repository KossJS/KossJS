'use strict';

const {
  Error,
  RangeError,
  SyntaxError,
  TypeError,
  URIError,
  Symbol,
  ObjectDefineProperty,
  ObjectDefineProperties,
} = globalThis;

const kIsNodeError = Symbol('kIsNodeError');

const codes = {};

class NodeError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = `${code} [${Error.prototype.constructor.name}]`;
    this[kIsNodeError] = true;
    
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

class NodeRangeError extends RangeError {
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = `${code} [${RangeError.prototype.constructor.name}]`;
    this[kIsNodeError] = true;
    
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

class NodeTypeError extends TypeError {
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = `${code} [${TypeError.prototype.constructor.name}]`;
    this[kIsNodeError] = true;
    
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

class NodeSyntaxError extends SyntaxError {
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = `${code} [${SyntaxError.prototype.constructor.name}]`;
    this[kIsNodeError] = true;
    
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

class AbortError extends Error {
  constructor(message, options) {
    super(message, options);
    this.code = 'ERR_CANCELED';
    this.name = 'AbortError';
    
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

codes.ERR_INVALID_ARG_TYPE = class extends NodeTypeError {
  constructor(name, expected, actual) {
    super('ERR_INVALID_ARG_TYPE', `The ${name} argument must be of type ${expected}. Received ${actual}`);
  }
};

codes.ERR_INVALID_ARG_VALUE = class extends NodeRangeError {
  constructor(name, value, reason) {
    let message = `The argument '${name}' is invalid`;
    if (reason) {
      message += `: ${reason}`;
    }
    if (value !== undefined) {
      message += `. Received ${value}`;
    }
    super('ERR_INVALID_ARG_VALUE', message);
  }
};

codes.ERR_OUT_OF_RANGE = class extends NodeRangeError {
  constructor(name, range, actual) {
    super('ERR_OUT_OF_RANGE', `The ${name} argument must be ${range}. Received ${actual}`);
  }
};

codes.ERR_MISSING_ARGS = class extends NodeTypeError {
  constructor(...args) {
    super('ERR_MISSING_ARGS', `The ${args.join(', ')} argument${args.length > 1 ? 's' : ''} must be specified`);
  }
};

codes.ERR_UNKNOWN_ENCODING = class extends NodeTypeError {
  constructor(encoding) {
    super('ERR_UNKNOWN_ENCODING', `Unknown encoding: ${encoding}`);
  }
};

codes.ERR_BUFFER_OUT_OF_BOUNDS = class extends NodeRangeError {
  constructor(name) {
    super('ERR_BUFFER_OUT_OF_BOUNDS', name ? `Index out of range (${name})` : 'Index out of range');
  }
};

codes.ERR_FS_FILE_TOO_LARGE = class extends NodeRangeError {
  constructor(size) {
    super('ERR_FS_FILE_TOO_LARGE', `File is too large (${size} bytes)`);
  }
};

codes.ERR_ACCESS_DENIED = class extends NodeError {
  constructor(path, code, message) {
    let msg = `Access denied`;
    if (path) msg += `: ${path}`;
    if (message) msg += ` (${message})`;
    super('ERR_ACCESS_DENIED', msg);
  }
};

codes.ERR_INVALID_BUFFER_SIZE = class extends NodeRangeError {
  constructor(size) {
    super('ERR_INVALID_BUFFER_SIZE', `Invalid buffer size: ${size}`);
  }
};

codes.ERR_SYSTEM_ERROR = class extends NodeError {
  constructor(ctx) {
    super('ERR_SYSTEM_ERROR', ctx?.message || 'System error');
    this.errno = ctx?.errno;
    this.syscall = ctx?.syscall;
  }
};

function hideStackFrames(fn) {
  return fn;
}

function aggregateTwoErrors(err1, err2) {
  if (!err1) return err2;
  if (!err2) return err1;
  
  const agg = new Error(err1.message);
  agg.code = 'ERR_MULTIPLE_EXCEPTIONS';
  agg.cause = err2;
  return agg;
}

module.exports = {
  Error,
  RangeError,
  SyntaxError,
  TypeError,
  URIError,
  NodeError,
  NodeRangeError,
  NodeTypeError,
  NodeSyntaxError,
  AbortError,
  hideStackFrames,
  aggregateTwoErrors,
  codes,
  kIsNodeError,
};