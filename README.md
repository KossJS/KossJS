# KossJS

**可嵌入的 JavaScript 运行时** — 一个轻量级、跨平台、以动态链接库（`.dll` / `.so` / `.dylib`）形式分发的 JS 引擎。

基于 [Boa Engine](https://boajs.dev/)（Rust 实现的 ECMAScript 引擎）构建，KossJS 对外暴露 **标准 C ABI**，可被 **任何编程语言** 调用 —— Python (ctypes)、C# (P/Invoke)、Java (JNA)、C++、TypeScript (koffi) 等。

## 特性

- **通用语言互操作** — 标准 C ABI，任何支持 C FFI 的语言均可调用
- **Node.js 兼容标准库** — 40+ 内置模块（`fs`、`net`、`http`、`crypto`、`path`、`os`、`buffer`、`stream`、`events`、`child_process`、`dgram`、`dns`、`zlib`、`assert`、`url`、`util`、`vm` 等）
- **双模块系统** — 同时支持 ES Module（`import`/`export`）和 CommonJS（`require()`/`module.exports`）
- **内置 `fetch()`** — 异步 HTTP 客户端，支持 TLS 指纹伪装
- **Worker 线程池** — 基于 OS 线程的并行 JS 执行
- **N-API 兼容层** — 加载已有的 `.node` 原生插件
- **原生 FFI（Senri）** — 运行时动态加载和调用任意 C 函数库（仅桌面端）
- **基于能力的沙箱** — 细粒度权限控制（`FS`、`NET`、`CRYPTO`、`WORKER`、`EXTERNAL_LOADER`）
- **内置标准库** — 所有标准库模块编译时嵌入二进制，运行时无需文件系统依赖

## 其他

### 关于商业使用的说明

本项目默认采用 GNU AGPL v3.0 + 附加权限进行许可。  
该协议已经允许你在**不修改源码、使用官方二进制库的前提下闭源使用**，无需任何额外许可。

如果你因公司内部政策等原因，**认为必须获得一份书面的商业许可**，请注意：

- 作者发布本项目时为未成年人，**本项目的开发与发布已获得法定代理人的知情与同意**；
- 尽管如此，本项目**目前不提供任何形式的商业许可销售**；
- 建议直接遵守 AGPL v3.0 + 附加权限，该协议已覆盖绝大多数商业使用场景；
- 作者成年后才会发布正式可用版本，其他版本不保证是否可用。

若你通过公开信息或推算发现作者已成年而此处尚未更新，可提醒作者修改，并届时获取**一份书面的商业许可**。

如有疑问，可通过 [B站链接](https://space.bilibili.com/1532090388) 联系作者（但不保证回复时效）。

## 快速开始

### C

```c
#include "kossjs.h"

KossInstance *js = koss_create(KOSS_CAP_ALL, NULL);
KossResult   res = koss_eval(js, "1 + 2", "main.js");

printf("%s\n", res.value); /* 3 */
koss_free_result(&res);
koss_destroy(js);
```

### Python

```python
from kossjs_interface import KossJS

js = KossJS()
result = js.eval("'你好, ' + '世界'")
print(result)  # 你好, 世界
```

### TypeScript（Node.js，基于 koffi）

```typescript
import { KossJS } from './kossjs_interface'

const js = new KossJS()
const result = js.eval('1 + 2')
console.log(result) // 3
```

> 完整文档请参阅 [KossJS 文档](https://docss.sxxyrry.qzz.io/KossJS/)

## 构建

```bash
# 编译动态库
cargo build --release

# 运行测试
cargo test --workspace
python -m pytest test/ -v
```

编译产物位于 `target/release/kossjs.dll`（Windows）、`target/release/libkossjs.so`（Linux）或 `target/release/libkossjs.dylib`（macOS）。

## 支持平台

| 平台 | 架构 |
|---|---|
| Windows | x86_64, i686, aarch64 |
| Linux | x86_64, aarch64 |
| macOS | x86_64, aarch64 |
| Android | aarch64, armv7, x86_64 |
| iOS | aarch64（设备 + 模拟器） |
| HarmonyOS | aarch64, x86_64 |

跨编译：`cargo ndk`（Android）、`cross`（Linux ARM64）、nightly Rust（HarmonyOS）。

## API 概览

| 函数 | 说明 |
|---|---|
| `koss_create` | 创建 JS 实例，指定能力标志 |
| `koss_destroy` | 销毁 JS 实例 |
| `koss_eval` | 同步执行 JavaScript 代码 |
| `koss_eval_async` | 执行 JavaScript 代码（等待 Promise） |
| `koss_inject_global` | 向全局作用域注入值 |
| `koss_register_function` | 将原生回调注册为 JS 函数 |
| `koss_register_class` | 注册原生类构造函数 |
| `koss_load_module` | 按路径加载 ES 模块 |
| `koss_register_module_loader` | 注册自定义模块加载器回调 |
| `koss_create_worker_pool` | 创建 Worker 线程池 |
| `koss_push_value` / `koss_pop_value` | Worker 线程池消息传递 |
| `koss_get_version` | 返回库版本字符串 |

完整的 C API 文档见 [`include/kossjs.h`](include/kossjs.h)。

## 许可证

**GNU Affero General Public License v3.0**，并附带 TT23XR Studio 第7节附加权限《非本软件模块的源代码公开义务例外》。

**闭源例外**：未经修改的官方预编译二进制库（`.dll`/`.so`/`.dylib`）可在注明出处的前提下链接到专有应用程序中。详见 [`LICENSE.md`](LICENSE.md)。

---

**KossJS** © 2026 TT23XR Studio
