// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/buffer - Node.js buffer module (L3)
// Maps to koss:buffer standard library

var kossBuffer = require('koss:buffer');

module.exports = kossBuffer;
module.exports.Buffer = kossBuffer.Buffer;
module.exports.Blob = kossBuffer.Blob;
module.exports.TextEncoder = kossBuffer.TextEncoder;
module.exports.TextDecoder = kossBuffer.TextDecoder;
module.exports.atob = kossBuffer.atob;
module.exports.btoa = kossBuffer.btoa;
module.exports.constants = kossBuffer.constants;
