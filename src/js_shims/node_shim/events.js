// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:node/events - Node.js events module (L3)
// Maps to koss:events standard library

var kossEvents = require('koss:events');

module.exports = kossEvents;
module.exports.EventEmitter = kossEvents.EventEmitter;
module.exports.once = kossEvents.once;
module.exports.on = kossEvents.on;
module.exports.getEventListeners = kossEvents.getEventListeners;
