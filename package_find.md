# KossJS 模块解析器设计文档

## 1. 目标

KossJS 是一个嵌入式 JavaScript 运行时，以动态库形式提供给宿主程序（Java/Python/C++等）调用。为了让 JavaScript 代码能够复用 npm 生态，KossJS 需要实现 **模块解析器**，即支持 `require` 和 `import` 语句按照 Node.js 的规则查找并加载依赖文件。

**核心需求**：
- 支持相对路径 (`./mod`, `../lib`)
- 支持绝对路径 (`/path/to/mod`)
- 支持裸模块名 (`lodash`, `express`) – 从 `node_modules` 目录中查找
- 自动处理文件扩展名（`.js`, `.mjs`, `.cjs`，可选 `.json`）
- 支持 `package.json` 中的 `main` 字段（简化版，先支持 `main`，暂不支持 `exports`）
- 向上递归查找 `node_modules` 直到文件系统根目录
- 不负责下载依赖，依赖由用户通过 `npm install` 等工具预先安装

## 2. 模块解析算法（Node.js 风格）

### 2.1 输入参数
- `specifier`：用户代码中的模块标识符（字符串）
- `parent_path`：当前执行模块的绝对路径（用于计算相对路径）

### 2.2 输出
- 模块文件的绝对路径，或解析失败错误

### 2.3 解析步骤

#### 步骤 1：判断标识符类型
- 若以 `'/'` 开头 → 视为绝对路径，直接尝试加载该路径
- 若以 `'./'` 或 `'../'` 开头 → 视为相对路径，基于 `parent_path` 解析
- 否则视为**裸模块名**（bare specifier），进入 `node_modules` 查找流程

#### 步骤 2：相对/绝对路径解析
1. 将 `specifier` 与 `parent_path` 的目录部分拼接，得到候选路径 `candidate`
2. 尝试按以下顺序查找文件（首次命中即返回）：
   - `candidate` 本身（如果存在且是文件）
   - `candidate.js`
   - `candidate.mjs`
   - `candidate.cjs`
   - `candidate.json`
   - `candidate/index.js`
   - `candidate/index.mjs`
   - `candidate/index.cjs`
   - `candidate/index.json`
3. 若均不存在，则报错 `MODULE_NOT_FOUND`

#### 步骤 3：裸模块名（node_modules 查找）
1. 从 `parent_path` 所在的目录开始，逐级向上查找 `node_modules/<specifier>` 目录：
   ```
   /project/src/current.js
   → /project/src/node_modules/lodash
   → /project/node_modules/lodash
   → /node_modules/lodash
   ```
2. 一旦找到 `node_modules/<specifier>` 目录，则进入该目录，解析其**入口文件**：
   - 优先读取目录下的 `package.json`，提取 `main` 字段（如果存在）
   - 若 `main` 字段存在，则将其解析为相对于该目录的路径，并执行相对路径解析（步骤 2）
   - 若 `main` 不存在或解析失败，则依次尝试：
     - `index.js`
     - `index.mjs`
     - `index.cjs`
     - `index.json`
3. 如果整个向上查找过程未找到有效文件，则报错 `MODULE_NOT_FOUND`

## 3. 与 BOA 引擎集成

BOA 提供了 `ModuleLoader` trait，允许自定义模块加载逻辑。KossJS 将实现自己的 `KossModuleLoader`，关键接口：

```rust
impl ModuleLoader for KossModuleLoader {
    fn load(
        &self,
        specifier: &str,
        referrer: Option<&str>,
        _is_dynamic: bool,
    ) -> Result<Module, ModuleLoadError> {
        // 1. 确定父路径（referrer）
        // 2. 调用模块解析器获得绝对路径
        // 3. 读取文件内容
        // 4. 创建 BOA Module 对象
    }
}
```

**注意**：由于 KossJS 以动态库形式存在，`ModuleLoader` 可能需要访问文件系统，同时需要考虑多实例隔离（每个实例独立缓存）。

## 4. 路径缓存与性能优化

为了避免重复的文件系统查询，模块解析器应实现**缓存**：

- **解析结果缓存**：`(specifier, parent_path) -> resolved_path` 的映射。使用 LRU 策略，默认容量 256。
- **文件存在性缓存**：对于已确认不存在的路径，也应短暂缓存（负向缓存），防止重复 stat。
- **package.json 内容缓存**：对解析过的 `package.json` 中的 `main` 字段进行缓存，避免重复读取和解析 JSON。

缓存生命周期：与 KossJS 实例绑定，实例销毁时释放。无需跨实例共享。

## 5. 错误处理与用户反馈

解析失败时，应返回明确的错误信息，包含：
- 错误类型（如 `MODULE_NOT_FOUND`）
- 失败的模块标识符
- 尝试过的搜索路径（便于调试）

例如：
```
Cannot find module 'lodash' from '/project/src/current.js'
Searched in:
  - /project/src/node_modules/lodash
  - /project/node_modules/lodash
  - /node_modules/lodash
```

## 6. 跨平台路径处理

- 统一使用 `/` 作为路径分隔符在 API 中表现，内部转换为平台原生分隔符（`std::path::MAIN_SEPARATOR`）。
- 绝对路径判断：在 Unix 上以 `/` 开头，在 Windows 上以盘符+`:`或 `\\` 开头。
- 使用 `std::path::Path` 和 `std::fs` 进行跨平台文件操作，避免手动拼接字符串。

## 7. 支持范围与限制

### 7.1 支持
- `require('相对/绝对路径')`
- `require('包名')`（从 `node_modules` 查找）
- `package.json` 的 `main` 字段
- 默认扩展名自动补全：`.js`, `.mjs`, `.cjs`, `.json`
- 目录默认入口：`index.js`, `index.mjs`, `index.cjs`, `index.json`

### 7.2 暂不支持
- `exports` 字段（条件导出、子路径等）
- `import` 映射（import maps）
- 循环依赖检测（由 BOA 负责）
- 远程模块（http:// 或 https://）
- `.node` 原生模块（需要另行绑定）

## 8. 使用示例（预期行为）

假设项目结构：
```
/project
  package.json
  node_modules/
    lodash/
      index.js
    chalk/
      package.json  -> { "main": "source/index.js" }
      source/
        index.js
  src/
    main.js
```

`main.js` 内容：
```js
const _ = require('lodash');
const chalk = require('chalk');
console.log(chalk.green('Hello'));
```

解析过程：
- `require('lodash')` → 查找 `/project/node_modules/lodash` → 存在目录 → 无 `package.json` → 默认 `index.js` → 返回 `/project/node_modules/lodash/index.js`
- `require('chalk')` → 查找 `/project/node_modules/chalk` → 存在目录 → 读取 `package.json` → `main` 指向 `source/index.js` → 返回 `/project/node_modules/chalk/source/index.js`

## 9. 后续扩展计划

- **阶段二**：支持 `exports` 字段（简化子路径）
- **阶段三**：支持 `require.resolve` API
- **阶段四**：可选的“打包模式”开关，当启用时直接执行 bundle 文件，跳过模块解析（提升性能）
