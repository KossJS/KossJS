/**
 * KossJS — Embeddable JavaScript Runtime
 * C ABI Header
 *
 * Usage from any language:
 *   Java  → JNA/JNI
 *   Python → ctypes / cffi
 *   C++   → direct include
 *   C#    → P/Invoke
 */

#ifndef KOSSJS_H
#define KOSSJS_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to a JS instance */
typedef void KossInstance;

/* Result type returned by most APIs.
 * `value` is a heap-allocated string; the caller MUST free it with
 * koss_free_string() or koss_free_result(). */
typedef struct {
    int code;       /* 0=ok, 1=js error, 2=bad argument */
    char *value;    /* heap string — free with koss_free_string */
} KossResult;

/* Native function callback type.
   Receives (argc, argv) and returns a heap-allocated string that the
   caller must free with koss_free_string(). Return NULL for undefined. */
typedef char* (*KossNativeFn)(int argc, const char **argv);

/* ── Capability flags ─────────────────────────────────────────────── */
typedef enum {
    /* 文件系统（6 个细粒度操作） */
    FS_READ         = 1u << 0,
    FS_WRITE        = 1u << 1,
    FS_DELETE       = 1u << 2,
    FS_MKDIR        = 1u << 3,
    FS_RENAME       = 1u << 4,
    FS_CHMOD        = 1u << 5,

    /* 网络（5 个细粒度操作） */
    NET_TCP_CLIENT  = 1u << 6,
    NET_TCP_SERVER  = 1u << 7,
    NET_UDP         = 1u << 8,
    NET_DNS         = 1u << 9,
    NET_FETCH       = 1u << 10,

    /* 加密（4 个细粒度操作） */
    CRYPTO_HASH     = 1u << 11,
    CRYPTO_HMAC     = 1u << 12,
    CRYPTO_RANDOM   = 1u << 13,
    CRYPTO_PBKDF2   = 1u << 14,

    /* 内置 FFI（5 个细粒度操作） */
    FFI_OPEN        = 1u << 15,
    FFI_CALL        = 1u << 16,
    FFI_ALLOC       = 1u << 17,
    FFI_CALLBACK    = 1u << 18,
    FFI_STRUCT      = 1u << 19,

    /* 其他模块（8 个操作） */
    NATIVE_ADDON    = 1u << 20,
    WASM            = 1u << 21,
    SHARED_MEMORY   = 1u << 22,
    HIGHRES_TIME    = 1u << 23,
    SYSINFO         = 1u << 24,
    MODULE_LOAD     = 1u << 25,
    DYNAMIC_CODE    = 1u << 26,
    DEBUG_CAP       = 1u << 27
} KossCapability;

#define KOSS_CAP_SANDBOX    0
#define KOSS_CAP_ALL_FS     (FS_READ | FS_WRITE | FS_DELETE | FS_MKDIR | FS_RENAME | FS_CHMOD)
#define KOSS_CAP_ALL_NET    (NET_TCP_CLIENT | NET_TCP_SERVER | NET_UDP | NET_DNS | NET_FETCH)
#define KOSS_CAP_ALL_CRYPTO (CRYPTO_HASH | CRYPTO_HMAC | CRYPTO_RANDOM | CRYPTO_PBKDF2)
#define KOSS_CAP_ALL_FFI    (FFI_OPEN | FFI_CALL | FFI_ALLOC | FFI_CALLBACK | FFI_STRUCT)
#define KOSS_CAP_ALL        0xFFFFFFFF

/* 兼容别名（用于旧宿主代码过渡） */
#define KOSS_CAP_FS              KOSS_CAP_ALL_FS
#define KOSS_CAP_NET             KOSS_CAP_ALL_NET
#define KOSS_CAP_CRYPTO          KOSS_CAP_ALL_CRYPTO
#define KOSS_CAP_WORKER          (1u << 3)
#define KOSS_CAP_EXTERNAL_LOADER MODULE_LOAD

/* ── Builtin module flags ──────────────────────────────────────────── */
typedef enum {
    KOSS_BUILTIN_NONE      = 0,
    KOSS_BUILTIN_NODE      = 1 << 0,
    KOSS_BUILTIN_BUN       = 1 << 1,
    KOSS_BUILTIN_DENO      = 1 << 2,
    KOSS_BUILTIN_KOSS      = 1 << 3,
    KOSS_BUILTIN_ALL       = 0xFFFFFFFF,
} KossBuiltin;

/* ── Instance lifecycle ─────────────────────────────────────────────── */
KossInstance *koss_create_with_caps(uint32_t caps, bool stable);
KossInstance *koss_create_with_builtins(uint32_t caps, uint32_t builtins, bool stable);
KossInstance *koss_create_with_modules_and_caps(const char *root_dir, uint32_t caps, bool stable);
KossInstance *koss_create_with_modules_and_builtins(const char *root_dir, uint32_t caps, uint32_t builtins, bool stable);
bool          koss_is_stable(KossInstance *inst);
uint32_t      koss_get_builtins(KossInstance *inst);
bool          koss_is_builtin_enabled(KossInstance *inst, uint32_t flag);

/* Backward-compatible wrappers — default stable=true, builtins=ALL */
static inline KossInstance *koss_create(void) {
    return koss_create_with_builtins(KOSS_CAP_ALL, KOSS_BUILTIN_ALL, true);
}
static inline KossInstance *koss_create_with_modules(const char *root_dir) {
    return koss_create_with_modules_and_builtins(root_dir, KOSS_CAP_ALL, KOSS_BUILTIN_ALL, true);
}

void koss_destroy(KossInstance *inst);

/* ── Capability query ───────────────────────────────────────────────── */
uint32_t koss_get_capabilities(KossInstance *inst);

/* ── Audit mask ─────────────────────────────────────────────────────── */
KossResult koss_set_audit_mask(KossInstance *inst, uint32_t mask);
uint32_t koss_get_audit_mask(KossInstance *inst);

/* ── Audit debug mode ──────────────────────────────────────────────── */
/* Enable or disable audit debug mode. When enabled, error messages include
   detailed information about denials, timeouts, and callback failures.
   Production environments should disable debug mode to avoid information leakage. */
void koss_enable_audit_debug(KossInstance *inst, bool enable);

/* ── Synchronous audit callback ─────────────────────────────────────── */
/* Audit callback type: called when an operation matching the audit mask
   is about to be performed. Return true to allow, false to block.
   Args: target (e.g. "fs.readFileSync"), args array, arg count,
         current working directory, userdata pointer. */
typedef bool (*AuditCallback)(const char* target, const char** args, int argc, const char* pwd, void* userdata);

/* Register or clear the synchronous audit callback.
   Pass NULL for callback to clear the audit callback. */
KossResult koss_check_sandbox(KossInstance *inst, AuditCallback callback, void* userdata);

/* ── Code execution ─────────────────────────────────────────────────── */
KossResult koss_eval(KossInstance *inst, const char *code);
KossResult koss_run_file(KossInstance *inst, const char *path);
KossResult koss_run_module(KossInstance *inst, const char *path);
KossResult koss_run_string(KossInstance *inst, const char *code);
KossResult koss_run_module_string(KossInstance *inst, const char *code);

/* ── Async execution & event loop ───────────────────────────────────── */
KossResult koss_run_async(KossInstance *inst, const char *code, uint64_t timeout_ms);
KossResult koss_tick(KossInstance *inst);

/* ── Global injection (host → JS) ───────────────────────────────────── */
KossResult koss_set_global_string(KossInstance *inst, const char *name, const char *value);
KossResult koss_set_global_number(KossInstance *inst, const char *name, double value);
KossResult koss_set_global_bool(KossInstance *inst, const char *name, bool value);
KossResult koss_set_global_null(KossInstance *inst, const char *name);
KossResult koss_set_global_undefined(KossInstance *inst, const char *name);
KossResult koss_set_global_json(KossInstance *inst, const char *name, const char *json_str);

/* ── Native function / class registration ───────────────────────────── */
KossResult koss_register_function(KossInstance *inst, const char *name, KossNativeFn func);
KossResult koss_register_module_loader(KossInstance *inst, KossNativeFn callback);
KossResult koss_register_class(KossInstance *inst, const char *class_name,
                                const char *methods_json, KossNativeFn callback);

/* ── Worker thread pool ─────────────────────────────────────────────── */
KossResult koss_create_worker_pool(KossInstance *inst, int32_t size);
KossResult koss_worker_post_message(KossInstance *inst, int32_t worker_id, const char *data);
KossResult koss_worker_execute(KossInstance *inst, int32_t worker_id, const char *code);
KossResult koss_worker_try_recv(KossInstance *inst);
KossResult koss_worker_terminate(KossInstance *inst, int32_t worker_id);
KossResult koss_worker_shutdown(KossInstance *inst);

/* ── Internal bindings & fetch ──────────────────────────────────────── */
KossResult koss_get_binding(KossInstance *inst, const char *binding_name);
KossResult koss_fetch(KossInstance *inst, const char *url_json);

/* ── Memory management ──────────────────────────────────────────────── */
/* WARNING: koss_free_string MUST ONLY be called on pointers returned by
   KossResult.value or by native callbacks that KossJS allocated. Passing
   a pointer from malloc(), a string literal, or a previously-freed pointer
   will cause undefined behavior (heap corruption, crash, or double-free).
   Safe to call with NULL (no-op). */
void koss_free_string(char *s);
/* Free a KossResult and its value string. The struct is passed by value.
   Must only be called once per KossResult — the value is consumed. */
void koss_free_result(KossResult result);

/* ── Info ────────────────────────────────────────────────────────────── */
/* Returns a static string (do NOT attempt to free this pointer). */
const char *koss_version(void);

#ifdef __cplusplus
}
#endif

#endif /* KOSSJS_H */
