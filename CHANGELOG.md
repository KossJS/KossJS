# 更新日志 (Changelog)

本项目所有重要变更都将记录在此文件中。

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
