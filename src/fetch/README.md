# 关于本目录

本目录下的部分代码来源于 [node-fetch](https://github.com/node-fetch/node-fetch) (NodeJS 官方 fetch polyfill)。

以下是直接来自 node-fetch 的文件（未经修改）：

- `src/index.js`
- `src/body.js`
- `src/request.js`
- `src/response.js`
- `src/headers.js`
- `src/utils/referrer.js`
- `src/utils/multipart-parser.js`
- `src/utils/is.js`
- `src/utils/is-redirect.js`
- `src/utils/get-search.js`
- `src/errors/base.js`
- `src/errors/fetch-error.js`
- `src/errors/abort-error.js`

这些文件源自 node-fetch 官方源代码，为保持与 Web Fetch API 标准的一致性。
如有需要为适配 KossJS 而修改某些地方，请另行处理，不要直接修改本目录下的这些源文件。
