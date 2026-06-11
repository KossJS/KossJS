# 运行时 API 获取设计 - 实现步骤

## 步骤 1：添加 `register_koss_global` 函数

**文件**：`src/runtime.rs`
**位置**：在 `register_console` 函数之后添加

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
    let _ = ctx.register_global_property(
        boa_engine::js_string!("KossJS"),
        koss_obj,
        boa_engine::property::Attribute::READONLY
            | boa_engine::property::Attribute::PERMANENT,
    );

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

## 步骤 2：在 `koss_create_with_caps` 中调用

**文件**：`src/runtime.rs`
**位置**：`koss_create_with_caps` 函数内，在 `register_console(&mut instance.context);` 之后添加

```rust
register_koss_global(&mut instance.context);
```

## 步骤 3：在 `koss_create_with_modules_and_caps` 中调用

**文件**：`src/runtime.rs`
**位置**：`koss_create_with_modules_and_caps` 函数内，在 `register_console(&mut instance.context);` 之后添加

```rust
register_koss_global(&mut instance.context);
```

## 验证

运行 cargo build 确认编译通过：

```bash
cargo build
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
