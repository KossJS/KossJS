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
void          koss_destroy(KossInstance *inst);

/* ── Code execution ─────────────────────────────────────────────────── */
KossResult koss_eval(KossInstance *inst, const char *code);
KossResult koss_run_file(KossInstance *inst, const char *path);

/* ── Global injection (host → JS) ───────────────────────────────────── */
KossResult koss_set_global_string(KossInstance *inst, const char *name, const char *value);
KossResult koss_set_global_number(KossInstance *inst, const char *name, double value);
KossResult koss_set_global_bool(KossInstance *inst, const char *name, bool value);

/* ── Native function registration ───────────────────────────────────── */
KossResult koss_register_function(KossInstance *inst, const char *name, KossNativeFn func);

/* ── Memory management ──────────────────────────────────────────────── */
void koss_free_string(char *s);
void koss_free_result(KossResult result);

/* ── Info ────────────────────────────────────────────────────────────── */
const char *koss_version(void);

#ifdef __cplusplus
}
#endif

#endif /* KOSSJS_H */
