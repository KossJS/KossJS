// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "独立模块闭源组合例外" ("Independent Module Exception for Closed-Source Combinations")

// koss:node/console - Node.js console module (L3)
// 返回全局 console（Node 中 console 既是全局对象也是模块）

module.exports = globalThis.console;
