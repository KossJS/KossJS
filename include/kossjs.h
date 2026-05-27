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
    KOSS_CAP_FS              = 1 << 0,  /* file system: read/write/delete/mkdir/readdir/link */
    KOSS_CAP_NET             = 1 << 1,  /* network: fetch() + raw sockets + DNS resolution  */
    KOSS_CAP_CRYPTO          = 1 << 2,  /* crypto: hash/hmac/pbkdf2/generatePrime/random     */
    KOSS_CAP_WORKER          = 1 << 3,  /* worker thread pool + Worker JS API                */
    KOSS_CAP_EXTERNAL_LOADER = 1 << 4,  /* external module loader (koss_register_module_loader) */
} KossCapability;

#define KOSS_CAP_SANDBOX 0
#define KOSS_CAP_ALL     (KOSS_CAP_FS | KOSS_CAP_NET | KOSS_CAP_CRYPTO \
                          | KOSS_CAP_WORKER | KOSS_CAP_EXTERNAL_LOADER)

/* ── Instance lifecycle ─────────────────────────────────────────────── */
KossInstance *koss_create(void);
KossInstance *koss_create_with_caps(uint32_t caps);
KossInstance *koss_create_with_modules(const char *root_dir);
KossInstance *koss_create_with_modules_and_caps(const char *root_dir, uint32_t caps);
void          koss_destroy(KossInstance *inst);

/* ── Capability query ───────────────────────────────────────────────── */
uint32_t koss_get_capabilities(KossInstance *inst);

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
