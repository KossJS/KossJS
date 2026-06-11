# 运行时 API 获取设计

## 全局对象

```javascript
KossJS = {
    version: "0.1.0-dev.7", // 从 src/version.rs 中的 get_version 动态获取
    runtime: "KossJS",       // 固定值
}
```

## 安全保护机制

### 双层保护：Rust 层 + JS 层

| 保护层 | 技术 | 作用 |
|--------|------|------|
| Rust 层 | `READONLY \| PERMANENT` 属性标志 | 阻止属性修改、删除、重新定义 |
| JS 层 | `Object.create(null)` + `Object.freeze()` | 无原型链、完全冻结 |

### 防护能力

| 攻击向量 | 防护措施 |
|----------|----------|
| `KossJS = {}` | Rust READONLY 阻止 |
| `KossJS.version = "x"` | Rust READONLY + JS freeze 阻止 |
| `delete KossJS.version` | Rust PERMANENT + JS freeze 阻止 |
| `Object.defineProperty(...)` | Rust PERMANENT + JS freeze 阻止 |
| 原型链污染 | `Object.create(null)` 无原型 |
| 嵌套对象修改 | 属性为字符串原始值，天然不可变 |

## 实现

### 文件：`src/runtime.rs`

#### 1. 添加 `register_koss_global` 函数

```rust
fn register_koss_global(ctx: &mut Context) {
    let version = match std::str::from_utf8(get_version()) {
        Ok(s) => s.trim_end_matches('\0').to_string(),
        Err(_) => "unknown".to_string(),
    };
    
    // Rust 层创建对象
    let mut obj = boa_engine::object::ObjectInitializer::new(ctx);
    obj.property(
        boa_engine::js_string!("version"),
        boa_engine::JsValue::from(boa_engine::js_string!(version)),
        boa_engine::property::Attribute::READONLY 
            | boa_engine::property::Attribute::ENUMERABLE 
            | boa_engine::property::Attribute::PERMANENT,
    );
    obj.property(
        boa_engine::js_string!("runtime"),
        boa_engine::JsValue::from(boa_engine::js_string!("KossJS")),
        boa_engine::property::Attribute::READONLY 
            | boa_engine::property::Attribute::ENUMERABLE 
            | boa_engine::property::Attribute::PERMANENT,
    );
    let koss_obj = obj.build();
    
    // Rust 层注册到 globalThis
    ctx.register_global_property(
        boa_engine::js_string!("KossJS"),
        koss_obj,
        boa_engine::property::Attribute::READONLY 
            | boa_engine::property::Attribute::PERMANENT,
    ).ok();
    
    // JS 层加固：无原型 + 冻结
    let harden_code = r#"
    (function() {
        var safe = Object.create(null);
        safe.version = globalThis.KossJS.version;
        safe.runtime = globalThis.KossJS.runtime;
        Object.freeze(safe);
        Object.defineProperty(globalThis, 'KossJS', {
            value: safe,
            writable: false,
            enumerable: false,
            configurable: false
        });
    })();
    "#;
    let source = boa_parser::Source::from_bytes(harden_code.as_bytes());
    if let Err(e) = ctx.eval(source) {
        eprintln!("Warning: Failed to harden KossJS global: {:?}", e);
    }
}
```

#### 2. 在 `koss_create_with_caps` 中调用

在 `register_console(&mut instance.context);` 之后添加：

```rust
register_koss_global(&mut instance.context);
```

## 使用示例

```javascript
console.log(KossJS.version);  // "0.1.0-dev.7"
console.log(KossJS.runtime);  // "KossJS"

// 以下操作全部失败
KossJS = {};                  // 静默失败或 TypeError
KossJS.version = "xxx";       // 静默失败或 TypeError
KossJS.newProp = 1;           // 静默失败或 TypeError
delete KossJS.version;        // 返回 false
Object.defineProperty(KossJS, 'x', {value: 1}); // TypeError
```
