# 关于本目录

本目录下的代码来源于 [Node.js 官方源代码](https://github.com/nodejs/node)。

## 来源说明

这些文件最初来自 Node.js 的内置模块（lib）实现。为了适配 KossJS 运行时有以下修改：

1. **模块系统适配**：将 CommonJS 格式转换为 ES Module 格式
2. **内置模块引用**：将 `node:` 前缀的内置模块引用调整为 KossJS 可识别的形式
3. **依赖简化**：移除或替换了部分 Node.js 特有的底层依赖
4. **全局变量替换**：将部分 Node.js 全局变量（如 `process`、`Buffer`）进行适配

## 文件结构

- `*.js` - 主要模块实现
- `internal/` - 内部实现模块
- `*/` - 子模块目录

如需查看原始 Node.js 源码，请访问 https://github.com/nodejs/node
