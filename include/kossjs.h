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

/* Result type returned by most APIs */
typedef struct {
    int code;       /* 0=ok, 1=js error, 2=bad argument */
    char *value;    /* heap string — free with koss_free_string */
} KossResult;

/* Native function callback type */
typedef char* (*KossNativeFn)(int argc, const char **argv);

/* ── Instance lifecycle ─────────────────────────────────────────────── */
KossInstance *koss_create(void);
KossInstance *koss_create_with_modules(const char *root_dir);
void          koss_destroy(KossInstance *inst);

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
void koss_free_string(char *s);
void koss_free_result(KossResult result);

/* ── Info ────────────────────────────────────────────────────────────── */
const char *koss_version(void);

#ifdef __cplusplus
}
#endif

#endif /* KOSSJS_H */
