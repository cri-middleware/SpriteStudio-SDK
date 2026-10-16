#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define ASSET_PATH_SEPARATOR '/'

enum CConverterError
#if defined(__cplusplus) || __STDC_VERSION__ >= 202311L
  : int32_t
#endif // defined(__cplusplus) || __STDC_VERSION__ >= 202311L
 {
  Success = 0,
  ErrorIo = -1,
  ErrorSspj = -2,
  ErrorConversion = -3,
  ErrorNotYetStarted = -4,
  ErrorAborted = -5,
  ErrorUnknown = -99,
};
#ifndef __cplusplus
#if __STDC_VERSION__ >= 202311L
typedef enum CConverterError CConverterError;
#else
typedef int32_t CConverterError;
#endif // __STDC_VERSION__ >= 202311L
#endif // __cplusplus

typedef struct Context Context;

typedef void (*LogCallback)(const char*);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * The converter's version string.
 *
 * **The caller owns the returned string** and must hand it to
 * [`ss_converter_version_free`]. It is the one function here that transfers
 * ownership; every other `*const c_char` this ABI returns is borrowed from a
 * context and must not be freed.
 */
const char *ss_converter_version(void);

/**
 * Release a string from [`ss_converter_version`]. Null is accepted and ignored.
 * Passing anything else — including a pointer from
 * [`ss_converter_scan_result_at`] or [`ss_converter_get_error`] — is undefined.
 */
void ss_converter_version_free(char *s);

uint32_t ss_converter_ssab_schema_version(void);

uint32_t ss_converter_ssqb_schema_version(void);

/**
 * A conversion context, or null if the allocation failed.
 *
 * **A context runs one job at a time** — a conversion
 * ([`ss_converter_convert`]) or a directory scan ([`ss_converter_scan_dir`]).
 * Starting a conversion drops any earlier scan, because `is_running`,
 * `is_finished` and `abort` all answer about whichever job is current. Use two
 * contexts to run a scan and a conversion at once.
 *
 * Release it with [`ss_converter_destroy`].
 */
struct Context *ss_converter_create(void);

/**
 * Release a context from [`ss_converter_create`]. Null is accepted and ignored.
 *
 * Every borrowed pointer taken from this context — the scan results, the error
 * text — dies with it.
 */
void ss_converter_destroy(struct Context *context);

/**
 * Start converting one `.sspj` into `output_dir`, **on a worker thread**.
 *
 * Returns immediately. Poll [`ss_converter_is_finished`], then call
 * [`ss_converter_get_result`] for the outcome and [`ss_converter_get_error`]
 * for the text when it is not `Success`.
 *
 * `callback` is invoked **on the worker thread** with a message that is only
 * valid for the duration of the call; copy anything you keep, and do not touch
 * a UI toolkit from it.
 *
 * Starting a conversion clears this context's previous result, error text and
 * scan — see [`ss_converter_create`] on one job at a time.
 */
void ss_converter_convert(struct Context *ctx,
                          const char *sspj,
                          const char *output_dir,
                          bool embedded,
                          LogCallback callback);

/**
 * Whether this context's current job is still going. The complement of
 * [`ss_converter_is_finished`] for a job that has started; both are `false` on
 * a context that has never been given one.
 */
bool ss_converter_is_running(struct Context *ctx);

/**
 * Whether this context's current job has run to completion (or was aborted).
 * `false` on a context that has never been given one, so this is "done", not
 * "idle".
 */
bool ss_converter_is_finished(struct Context *context);

/**
 * Ask this context's current job to stop early. Returns immediately — the
 * worker notices at its next checkpoint, so keep polling
 * [`ss_converter_is_finished`]. An aborted conversion reports `ErrorAborted`.
 */
void ss_converter_abort(struct Context *context);

/**
 * The outcome of the conversion started on this context.
 *
 * **Blocks until the worker finishes**, so call [`ss_converter_is_finished`]
 * first unless blocking the calling thread is acceptable — on an editor's UI
 * thread it is not.
 *
 * Asking twice is safe and gives the same answer. `ErrorNotYetStarted` means
 * exactly that: no conversion was ever started on this context.
 */
CConverterError ss_converter_get_result(struct Context *context);

/**
 * The failure text behind a non-`Success` [`ss_converter_get_result`], as
 * UTF-8 with its length. Either out pointer may be null.
 *
 * Writes a null pointer and a zero length when there is no error — including
 * before `ss_converter_get_result` has been called, which is what fills this
 * in. **The text is borrowed from the context**: do not free it, and do not
 * keep it past the next conversion on that context or the context's own
 * destruction.
 */
void ss_converter_get_error(struct Context *context, const char **out_data, uintptr_t *out_len);

/**
 * Start scanning `dir` for `.sspj` files, **on a worker thread**.
 *
 * Returns immediately; poll [`ss_converter_is_finished`]. The three budgets
 * bound the walk — `max_depth` directories deep, `max_entries` entries, and
 * `max_millis` of wall clock; [`ss_converter_scan_budget_exceeded`] reports
 * whether one of them cut the walk short, in which case the results are a
 * prefix rather than the whole tree.
 *
 * See [`ss_converter_create`] on one job at a time: a context scanning is not
 * a context converting.
 */
void ss_converter_scan_dir(struct Context *ctx,
                           const char *dir,
                           uint32_t max_depth,
                           uintptr_t max_entries,
                           uint64_t max_millis);

/**
 * Directory entries the scan has looked at so far — progress, not results.
 */
uintptr_t ss_converter_scan_visited(struct Context *ctx);

/**
 * `.sspj` files the scan has found so far. Safe to poll while it runs, unlike
 * [`ss_converter_scan_result_count`].
 */
uintptr_t ss_converter_scan_found(struct Context *ctx);

/**
 * Whether one of [`ss_converter_scan_dir`]'s budgets stopped the walk before
 * it reached the end of the tree. The results are still usable; they are just
 * not everything.
 */
bool ss_converter_scan_budget_exceeded(struct Context *ctx);

/**
 * How many `.sspj` paths the scan has, and **the call that publishes them**.
 *
 * Despite the name this rebuilds the context's result strings from the
 * scanner, which is what makes them readable by
 * [`ss_converter_scan_result_at`] — so call it before the first read, and
 * again whenever you want newer results.
 *
 * **It invalidates every pointer a previous [`ss_converter_scan_result_at`]
 * handed out.** Read the paths you want, or copy them, before calling this
 * again.
 */
uintptr_t ss_converter_scan_result_count(struct Context *ctx);

/**
 * One `.sspj` path from the scan, as UTF-8, or null when `index` is past the
 * count that [`ss_converter_scan_result_count`] last reported.
 *
 * **Borrowed from the context** — do not free it. It dies at the next
 * [`ss_converter_scan_result_count`], the next [`ss_converter_convert`] on
 * this context, or the context's own destruction.
 */
const char *ss_converter_scan_result_at(struct Context *ctx, uintptr_t index);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
