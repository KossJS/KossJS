# 更新日志 (Changelog)

本项目所有重要变更都将记录在此文件中。

## 0.1.0-dev.10 - 2026-08-06

### 安全强化 (Security)

- **审核掩码与审核回调的关系强化** — 新增 `KossConfigError` 错误类型，明确 "Audit Mask ≠ 0 但未注册审核回调" 时的行为：
  - `Audit Mask = 0`：不触发审核，直接依据 Capability 放行或拒绝
  - `Audit Mask ≠ 0` 且 `Callback ≠ NULL`：调用回调，由宿主逻辑决定放行或拒绝
  - `Audit Mask ≠ 0` 且 `Callback = NULL`：视为安全策略配置不完整，直接拒绝（Deny），并抛出 `KossConfigError`，消息为 `Audit mask is set but no callback is registered`
  - 修复：此前"掩码≠0 但无回调"会静默放行，形成潜在的安全绕过
- **JS 层审核回调（两级审核链）** — 新增 `__koss_set_audit_callback(fn)` 全局函数与 `koss_clear_js_audit()` C ABI：
  - 宿主审核回调返回 `true` 后，可选的 JS 策略函数 `(target, args[], pwd) => boolean` 做进一步限制
  - JS 回调只能进一步拒绝（返回 `false`/抛异常 → `KossSecurityError`），不能放行宿主已拒绝的操作或绕过能力位
  - 宿主回调为 `NULL` 时（即使已注册 JS 回调），掩码覆盖的操作仍抛 `KossConfigError`
  - JS 回调执行期间再次触发审核（重入）直接拒绝，防止死循环
- **审核回调 `pwd` 修正** — `pwd` 由进程 `current_dir()` 改为「当前执行模块的目录」；`eval`/`run_string`/`run_module_string` 时为 `NULL`
- **`process.dlopen` 能力位门控** — 增加 `NATIVE_ADDON` 门控，防止绕过稳定模式加载原生插件
- **FFI 审核路径统一** — `open`/`call`/`alloc`/`callback` 等 FFI 操作统一经过审核回调路径，支持动态策略变更
- **TOCTOU 修复** — 模块加载路径校验后使用 verified 路径读取
- **细粒度能力位注册** — 网络、加密、文件系统操作按细粒度能力位独立注册与校验
- **TCP socket 表生命周期** — 全局 TCP socket 表移入实例生命周期管理

### 移除 (Removed)

- **Worker 线程池** — 删除 `src/worker.rs`、`koss_shim/worker.js`、`__koss_worker_*` C ABI 与 Python 绑定、`KOSS_CAP_WORKER` 能力位、`koss:worker` 模块；`FS_MKDIR` 恢复自身门控（此前与 `KOSS_CAP_WORKER` 位冲突）
- **默认构造器能力变更** — `koss_create()` / `koss_create_with_modules()` 默认能力由 `KOSS_CAP_ALL` 改为 `KOSS_CAP_SANDBOX`，需显式传入 `KOSS_CAP_ALL`

### 新增 (Added)

- **Node 内置模块 shim 扩充** — 新增 `koss:assert`、`koss:buffer`、`koss:constants`、`koss:querystring`、`koss:zlib` 等模块
- **os shim 增强** — 新增 `version()`/`machine()` 与 `os.constants`（信号、errno 表）
- **crypto shim 调整** — `hash`/`hmac` 直接返回 hex；`sign`/`verify` 改用 HMAC-SHA256 加密钥派生
- **buffer shim 增强** — 支持索引访问与 `Array` 构造，`toString` 改为不可枚举
- **path shim** — 兼容 Windows 分隔符
- **Deno shim** — `Deno.mkdir` 幂等处理 `EEXIST`
- **全平台 shim 迁移** — shim 迁移到 koss 标准库，扩充 Rust 原生实现
- **ESM `koss:/*` 协议** — ESM import 支持 `koss:/*` 协议模块 + Rust 单元测试补全
- **删除 `src/stdlib` 目录** — 正式移除（不影响编译）
- **Python 接口** — 优化动态库寻找逻辑；新增 `clear_js_audit()`
- **TypeScript 接口** — 清理 Worker 绑定；新增 `clearJsAudit()`

### 修复 (Fixed)

- FFI 按 FS 能力位门控 `__koss_fs_*` + 无损二进制写入（C1/H6）
- FFI struct `char*` 字段 CString 生命周期（C2）
- N-API 统一 tagged value 表示（C3/H1-H5）
- FFI 回调分配回收不泄漏（H7）
- 跨平台 CI：macOS dylib 产物名、CI 无法测试
- 所有源代码文件添加头部版权标识

### 测试

- Rust 单元测试 156 passed（新增 JS 审核回调 / pwd 相关 7 个）
- Python 集成测试 744 passed, 8 skipped（新增 `test_sandbox_js_audit.py` 11 个）

## 0.1.0-dev.5.fix - 2026-05-15

### 新增 (Added)

- **内联模块加载器** — 新增 `register_internal_module_loader()`，为 CommonJS `require()` 提供内置模块加载支持：
  - 优先从嵌入式 stdlib（编译时嵌入的 JS 文件）加载模块
  - 未找到时回退到外部模块加载器回调（`external_module_loader`）
- **`KossInstance.external_module_loader` 字段** — 可选的外部模块加载器回调，供 C ABI 注册
- **`safe_js_value_to_json()`** — 安全的对象→JSON 转换函数，处理循环引用和函数类型
- **`embedded_stdlib` 模块公开** — `src/lib.rs` 中公开 `embedded_stdlib` 模块，供模块加载器直接读取嵌入式源码
- **`koss_register_module_loader` 支持清除** — 传入 `null` 回调可清除已注册的外部加载器

### 更改 (Changed)

- **模块系统重构**：
  - `module_loader.rs`：内置模块从文件系统读取改为直接从嵌入的 Rust 字符串读取，减少运行时 I/O
  - `resolver.rs`：stdlib 路径匹配使用 `embedded_stdlib_exists()` 替代 `file_exists()`，避免依赖磁盘文件存在性
  - JS 端 `require()` 重写：移除硬编码的内置模块列表，移除 `primordials`/`internalBinding` 依赖，改进 `exports` 自定义支持
- **`koss_eval` 返回值改进**：对象类型结果自动返回 JSON 字符串（循环安全），不再一律调用 `.toString()`
- **`koss_register_module_loader` 改为存储回调引用**：不再直接注册 JS 函数，而是将回调存储在 `external_module_loader` 字段中，由 `register_internal_module_loader` 统一调度
- **Python 测试接口**：`eval()` 返回自动解析 JSON（对象/数组），测试 stdlib 路径改为相对于项目根目录

### 修复 (Fixed)

- **Android 崩溃彻底修复**（`src/runtime.rs`）：
  - `KossEventLoop::new()`：tokio 多线程运行时 → `new_current_thread()` 单线程运行时，避免 Android 上线程创建限制导致的 SIGABRT；失败时返回 `None` 而非 panic
  - `koss_create()` / `koss_create_with_modules()`：`ContextBuilder::build()` 失败时返回 `null` 指针，而非 fallback 到 `Context::default()`（后者在 Android 上同样崩溃）
  - `KossInstance::new()`：适配可选的 `event_loop`，无事件循环时仍可正常使用（同步执行）
- **`koss_register_module_loader`**：允许 `callback` 为 `null` 以清除加载器

## 0.1.0-dev.5 - 2026-05-14

### 新增 (Added)

- **异步/事件循环系统** — 新增 `KossEventLoop` 结构体，集成 tokio 运行时驱动异步 I/O：
  - `register_native_fetch` 改为基于 Promise 的异步 fetch
  - `reqwest` 从 blocking 改为 async，新增 tokio 依赖
  - 新增 `koss_run_async`、`koss_tick` C ABI 接口
- **Worker 线程池** — 新增 `src/worker.rs`：
  - `WorkerPool` 实现，每个 worker 独立 OS 线程 + 独立 Boa Context
  - JS 端 worker API（`__koss_create_worker_pool`、`post_message`、`execute` 等）
  - 对应 C ABI 接口
- **C ABI 扩展**：
  - 内存管理：`koss_free_string`、`koss_free_result`
  - 类型化全局变量设置：`koss_set_global_number/bool/null/undefined/json`
  - 函数注册：`koss_register_function`（支持点号路径，如 `Math.max`）
  - 模块加载器注册：`koss_register_module_loader`
  - 类注册：`koss_register_class`（基于原生回调的 JS 类）
- **测试目录** — 新增 `test/` 测试目录

### 更改 (Changed)

- **模块系统改进**：
  - `primordials` 新增 SafeMap/Set/WeakMap/WeakSet
  - 新增 `internalBinding` / `getInternalBinding` / `getLinkedBinding` 桩
  - 模块加载支持 type 分发（module/object）
  - 新增 `register_native_bindings()` 结构化绑定系统
- **标准库重写**：
  - `buffer.js`：移除 primordials 依赖，改用原生 API 兼容 Boa
  - `events.js`：惰性加载 `internal/util/inspect` 避免循环依赖
  - `assert.js`：`assert.ok` → `assertok` 避免关键字冲突
  - `path.js` 系列：自包含实现，消除循环依赖
  - `internal/worker.js` + `worker_threads.js`：重写为 KossJS 原生 worker API
  - 移除对 Node.js C++ 内部绑定的依赖
- **依赖更新** (`Cargo.toml`)：
  - `reqwest` 0.13.2 blocking → 0.12 async（feature: json）
  - 新增 `tokio` 1.x（features: rt-multi-thread, macros, sync, fs, net, time）

### 修复 (Fixed)

- **Android 崩溃修复** — `koss_create` 改用 `ContextBuilder` 替代 `Context::default()`，解决安卓上 Boa 引擎初始化崩溃的问题
- Worker 线程 Context 创建也用 `ContextBuilder`，失败时优雅返回错误事件

## 0.1.0-dev.4 - 2026-04-13

### 修复 (Fixed)

- 各种Rust编译警告
- 更新库版本造成的错误和警告
- Android构建时用 strip 安装时找不到系统包

## 0.1.0-dev.3 - 2026-04-06

### 新增 (Added)

- **Rust绑定 (`src/bindings.rs`)** - 新文件，包含原生Rust实现：
  - `fs` 模块：文件系统操作（读取、写入、stat、mkdir、rmdir、复制等）
  - `os` 模块：CPU信息、内存、主机名、运行时间、用户信息等
  - `constants` 模块：文件系统标志、操作系统错误码、信号
  - `buffer` 模块：字节操作（比较、复制、填充、交换等）
  - `timers` 模块：定时器管理（调度、清除、获取当前时间）
  - `crypto` 模块：随机值、UUID、哈希函数（SHA1/SHA256/MD5）、HMAC、PBKDF2
  - `net` 模块：IP验证、TCP/UDP套接字、DNS查询
  - `http_parser` 模块：HTTP请求/响应解析
  - `url` 模块：URL解析和格式化
  - `util` 模块：系统错误处理
  - `trace_events` 模块：跟踪事件支持

- **模块系统** - ES模块支持：
  - `src/module_loader.rs` - ES模块的自定义模块加载器
  - `src/resolver.rs` - 模块路径解析器
  - `src/lib.rs` - 集成模块加载器

- **标准库模块** - Node.js标准库实现：
  - `src/stdlib/timers.js` - 完整定时器（setTimeout/setInterval/setImmediate）
  - `src/stdlib/timers/promises.js` - Promise版本的定时器
  - `src/stdlib/os.js` - OS模块
  - `src/stdlib/buffer.js` - Buffer实现
  - `src/stdlib/fs.js` - 文件系统模块
  - `src/stdlib/internal/` - 内部模块（错误、验证器、工具等）
  - `src/stdlib/` 中的其他Node.js标准库模块

- **运行时更新** (`src/runtime.rs`)：
  - 添加 `koss_get_binding()` FFI函数以支持internalBinding
  - 在JavaScript中添加 `internalBinding`，调用 `__koss_bindings`
  - 添加 `primordials` 用于Node.js兼容性
  - 添加全局 `process` 对象
  - 添加CommonJS模块系统（`require()`、`module`、`exports`）

- **Python接口** (`kossjs_interface.py`)：
  - 添加 `_get_binding()` 方法用于访问Rust绑定
  - 添加 `get_binding()` 获取内部绑定信息

- **Cargo依赖** (`Cargo.toml`)：
  - 新增：`once_cell`、`base64`、`num_cpus`

- **Fetch模块** (`src/fetch/simple-fetch.js`)：
  - 新的简化fetch实现

### 更改 (Changed)

- **`Cargo.toml`**：更新依赖
- **`kossjs_interface.py`**：增强绑定支持和模块加载器
- **`src/runtime.rs`**：为internalBinding、primordials、process和模块系统进行重大更新
- **`src/lib.rs`**：添加模块加载器集成

### 修复 (Fixed)

- 各种Rust编译警告
- 模块解析问题

---

## 0.1.0-dev.1 - 2026-04-03

- KossJS初始实现
- Boa引擎集成
- 基本JavaScript执行
- Python FFI接口
- Fetch API实现
- 基本测试套件
