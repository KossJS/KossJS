# KossJS 的 Node.js 内置库支持状态表

> 更新时间：2026-06-21
> 全部测试：536 passed
>
> KossJS 定位为**同时兼容 Node.js 与 Web 标准**的嵌入式运行时。
> Node.js 模块通过 `require()` 加载；Web API（如 `fetch`、`Headers`、`Response`）作为全局对象直接可用。
> 两者在功能上有重叠的部分（如 `timers` 模块导出与全局 `setTimeout` 是同一实现），不重复列出。

---

## ✓ 完整支持（可直接 require 使用）

| 模块 | 说明 | 实现方式 |
|------|------|---------|
| **assert** | 完整 Node.js 断言库 | Node.js 源码适配 |
| **buffer** | Buffer、Blob、TextEncoder/Decoder | Rust binding + JS 包装 |
| **events** | EventEmitter | Node.js 源码适配 |
| **os** | 操作系统信息（部分值硬编码） | 纯 JS shim |
| **path** | 路径处理（posix/win32） | Node.js 源码适配 |
| **process** | process 全局对象 | Rust 运行时注入 |
| **querystring** | 查询字符串解析/序列化 | Node.js 源码适配 |
| **timers** | setTimeout/setInterval/setImmediate | Rust binding + JS 包装 |
| **url** | URL 解析/格式化/URLPattern | Rust binding + JS 包装 |
| **string_decoder** | 字符串解码器 | Node.js 源码适配 |
| **constants** | 系统常量（信号、错误码等） | Rust binding |

---

## ✓ 部分支持（核心 API 可用，非全部 Node.js API）

| 模块 | 支持 API | 说明 |
|------|---------|------|
| **net** | `Socket`(connect/write/end/destroy/on('data')), `Server`(listen/close/on('connection')), `connect()`, `createServer()`, `isIP()`, `isIPv4()`, `isIPv6()` | 纯 JS shim，TCP 客户端/服务器基于 Rust TcpStream |
| **dns** | `lookup()`(callback + Promise), `resolve()`, `resolve4()`, `resolve6()`, `promises.lookup()` | 基于 Rust DNS 解析 |
| **tls** | `connect()`, `createServer()`, `TLSSocket`, `createSecureContext()`, `checkServerIdentity()` | 基于 net 模块的轻量封装 |
| **http** | `createServer()`(HTTP 请求/响应处理), `IncomingMessage`, `ServerResponse`, `METHODS`, `STATUS_CODES` | 纯 JS shim |
| **https** | `createServer()`, `request()`, `get()` | 基于 http + net |
| **crypto** | `randomBytes()`, `createHash()`(sha256/sha1/md5), `randomUUID()`, `timingSafeEqual()`, `randomFill()`, `randomFillSync()`, `getHashes()` | Rust 实现（非密码学安全哈希） |
| **stream** | `Readable`, `Writable`, `Duplex`, `Transform`, `PassThrough`, `pipeline()`, `finished()` | 纯 JS shim |
| **zlib** | `gzipSync/gunzipSync()`, `deflateSync/inflateSync()`, `gzip()/gunzip()`(async), `constants` | Rust flate2 crate |
| **dgram** | `createSocket()`, `socket.bind()`, `socket.send()`, `socket.close()`, `socket.address()` | UDP 基础封装（基于 TCP 桥接） |

---

## ✓ 纯 JS Shim（基础功能可用）

| 模块 | 说明 |
|------|------|
| **util** | `format()`, `inspect()`, `deprecate()`, `promisify()`, `inherits()`, `types.*`, `debuglog()`, `stripVTControlCharacters()`, `getSystemErrorName()` |
| **trace_events** | `createTracing()`, `getEnabledCategories()`, `Tracing` class |
| **perf_hooks** | `performance`, `PerformanceObserver`, `PerformanceMark/Measure`, `createHistogram()`, `monitorEventLoopDelay()`, `timerify()` |
| **diagnostics_channel** | `channel()`, `subscribe()`, `unsubscribe()`, `publish()`, `hasSubscribers()` |

---

## ❌ 不支持的模块（架构级否决）

这些模块**从设计上就不在 KossJS 的能力模型中**，不会支持。

| 模块 | 否决原因 |
|------|---------|
| **child_process** | 无对应能力位。且进程派生会完全破坏沙箱隔离 |
| **cluster** | 依赖 child_process.fork()，同上 |
| **wasi** | WASI 是另一套系统接口，与 KossJS 能力位模型平行不兼容 |
| **sea** | 单可执行文件打包是构建时工具，与运行时无关 |
| **punycode** | 已废弃模块 |
| **v8** | Boa 引擎无此 C++ 接口 |
| **inspector** | Boa 引擎无 V8 Inspector 协议 |
| **async_hooks** | Boa 无异步资源追踪机制 |
| **readline** | TTY 环境缺失（宿主通常无终端） |
| **repl** | 交互式解释器在嵌入式场景中无意义 |
| **vm** | 沙箱模型冲突，vm 的上下文隔离与 KossJS 沙箱不兼容 |

---

## 搁置模块（未来可能支持）

| 模块 | 说明 | 前置条件 |
|------|------|---------|
| **http2** | HTTP/2 协议支持 | 需 Rust h2 crate |
| **quic** | QUIC 协议 | 需 Rust quinn crate |
| **sqlite** | SQLite 数据库 | 需 Rust rusqlite crate |
| **worker_threads** | Web Worker | 能力位已预留，stable 模式剥离 |

---

## ✓ Web API（全局可用，无需 require）

以下 API 是 Web 标准，不通过 `require()` 加载，直接作为全局对象使用。
全局定时器（`setTimeout`/`setInterval` 等）与 `require('timers')` 的导出为同一实现，此处不再重复列出。

| API | 说明 | 实现方式 |
|-----|------|---------|
| **fetch** | `fetch(url, init)` 标准 Web API, GET/POST/PUT/DELETE, 自定义 headers, request body, HTTPS/TLS, Response.text()/json()/arrayBuffer()/blob(), Headers 迭代, SSRF 防护 | Rust reqwest + rustls 完整实现 |
| **Headers** | Web API Headers 类 (get/set/has/delete/forEach/keys/values/entries) | 纯 JS |
| **Response** | Web API Response 类 (text/json/arrayBuffer/blob/clone/error/redirect) | 纯 JS |

---

## 模块统计

| 分类 | 数量 |
|------|------|
| 完整支持 | 11 |
| 部分支持（核心 API） | 10 |
| 纯 JS Shim | 4 |
| 不支持（架构否决） | 11 |
| 搁置 | 4 |
| Web API | 3 |
| **总计** | **43** |

---

## 测试覆盖

- 测试文件数：17
- 总测试用例：536
- 测试框架：pytest
