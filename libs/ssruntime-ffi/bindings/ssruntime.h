/*
 * The C ABI — how every non-Rust Player talks to the runtime.
 *
 * A flat set of `extern "C"` functions mirroring the Rust API, plus the opaque handles
 * that carry state across the boundary. `cbindgen` generates `ssruntime.h` from this
 * module and opens it with this text, so it is written for the C or C++ integrator.
 *
 * # Handles
 *
 * Every handle is an opaque `void *`. The two that every Player holds:
 *
 * - **Resource** (`ss_resource_create_copy` / `_borrow` → `ss_resource_destroy`) — parsed
 *   `.ssab` data. Internally reference-counted, so it is safe to destroy a resource that
 *   contexts are still bound to; the buffer stays alive until the last one lets go.
 *   The `_borrow` variant does *not* copy, so the caller's buffer must outlive the
 *   resource and every context and effect slot holding it.
 * - **Context** (`ss_runtime_create` → `ss_runtime_destroy`) — the playback state of
 *   one animated object: a stack of tracks, each playing an animation, layered onto the
 *   setup pose. Bind a resource with `ss_runtime_bind_resource`; several contexts may
 *   share one.
 *
 * Effect and instance parts get their own handles (`ss_effect_slot_create`,
 * `ss_instance_slot_create`), stepped once per tick by the Player.
 *
 * # Calling convention
 *
 * Fallible calls return `bool` (or a null pointer, or `SS_RUNTIME_INVALID_HANDLE`) and
 * record the reason on the context. `ss_runtime_get_last_error` reads back a stable code,
 * and `ss_runtime_error_message` turns that code into the text to put in front of a
 * user — which is what a Player owes its caller.
 * **The message takes the code, not the context**, so it is a pure lookup with nothing
 * to own and nothing to outlive. The track and source getters that return a float use
 * `NaN` for a track or source that does not exist, which is distinguishable from a
 * legitimate `0.0`. The context-wide ones (`ss_runtime_get_frame_no`,
 * `ss_runtime_get_speed_scale` and `ss_runtime_update`) have no such case, and return
 * `0.0` for a null context.
 *
 * The two `ss_resource_create_*` constructors and `ss_effect_slot_create` are the
 * exception: they take no context, so a null return is the whole report. For a resource
 * that is enough, because there are only two ways to get one — a null `ptr`, which the
 * caller passed, and an `.ssab` whose schema version this runtime does not read. (Bytes
 * that are not an `.ssab` at all are not among them; nothing here validates the input.
 * See *Validation* below.)
 *
 * # Building a resource
 *
 * `ss_resource_create_*` then `ss_runtime_bind_resource` is the full form: it leaves
 * you holding the resource handle, which is what `ss_effect_slot_create` takes and
 * what lets one parsed pack serve several contexts. `ss_runtime_load_ssab_*` is the
 * one-liner for when you need neither — it creates, binds, and drops its own reference.
 *
 * **They are alternatives, not a sequence.** Calling both on the same bytes parses the
 * pack twice, and the parse is where every animation's metadata is built.
 *
 * Buffers handed out through `out_data` / `out_len` pairs point into runtime-owned
 * memory and stay valid only until the next call that rewrites them. Copy anything you
 * need to keep.
 *
 * # Safety
 *
 * Every entry point that takes a handle or a pointer relies on one contract, stated
 * here rather than on each of them (it is why they are `unsafe` to call from Rust):
 *
 * - A handle is null, or what its own create function returned and not yet destroyed:
 *   a context where a context is asked for, a slot where a slot is. A null handle is
 *   never dereferenced.
 * - A handle is not used from two threads at once. Different handles may be used from
 *   different threads at the same time, contexts that share one resource included.
 * - An `out_data` / `out_len` pointer is null or valid for a write.
 *
 * The ones that read a data or string pointer add their own `# Safety` section.
 *
 * # Validation
 *
 * Loading does not validate the `.ssab` (see "Limitations to design around" in the
 * `ssruntime` crate docs). Every entry point that takes a data pointer trusts it.
 */

#ifndef SSRUNTIME_H
#define SSRUNTIME_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Stride (number of `f32`s) per part in the `shape_vertices` and
 * `shape_vertex_box_coords` buffers (`ss_runtime_get_shape_vertices` and
 * `ss_runtime_get_shape_vertex_box_coords` in the C-API): 12 vertices of two
 * floats each, the most a shape part has (a star).
 */
#define SHAPE_VERTICES_STRIDE 24

/**
 * Stride (number of `f32`s) per part in the `local_vertices` SoA buffer.
 * Layout: `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
 * in part-local space, with `coordinate_system` (Y-Up/Y-Down) flip already applied.
 */
#define LOCAL_VERTICES_STRIDE 10

/**
 * Severity passed to `SsLogCallback`. The runtime only ever emits `ERROR`
 * today; `WARN` and `INFO` are reserved so a host can switch on the level
 * without its arms going stale when the runtime starts using them.
 */
#define SS_RUNTIME_LOG_ERROR 1

/**
 * See `SS_RUNTIME_LOG_ERROR`.
 */
#define SS_RUNTIME_LOG_WARN 2

/**
 * See `SS_RUNTIME_LOG_ERROR`.
 */
#define SS_RUNTIME_LOG_INFO 3

/**
 * Sentinel returned on failure by the functions that return a source handle,
 * and by `ss_runtime_get_track_blend` / `ss_runtime_get_track_sync` for a track
 * that does not exist. No live handle ever equals it.
 */
#define SS_RUNTIME_INVALID_HANDLE UINT32_MAX

/**
 * Track blend operators: what `ss_runtime_set_track_blend` takes and
 * `ss_runtime_get_track_blend` returns.
 */
#define SS_RUNTIME_TRACK_BLEND_MIX 0

#define SS_RUNTIME_TRACK_BLEND_ADDITIVE 1

/**
 * Where the animations laid on a track start: what `ss_runtime_set_track_sync`
 * takes and `ss_runtime_get_track_sync` returns.
 */
#define SS_RUNTIME_TRACK_SYNC_FREE 0

#define SS_RUNTIME_TRACK_SYNC_START 1

typedef struct SsMatrix4x4 SsMatrix4x4;

typedef void (*SsLogCallback)(int level, const char *message);

/**
 * Result of `EffectSlot::step` / `ss_effect_slot_step`.
 *
 * `draw_plan_buf` is a FlatBuffer-encoded `EffectDrawPlan` (see
 * `fbs/effect_draw_plan.fbs`). It is valid only when `visible == true`,
 * and only until the next `step` call on the same slot — callers must consume
 * it within the same frame and not retain pointers across calls.
 */
typedef struct ss_effect_step_result {
  bool visible;
  const uint8_t *draw_plan_buf;
  uintptr_t draw_plan_len;
} ss_effect_step_result;

/**
 * Resolved per-slot Effect event state, returned by
 * `ss_runtime_get_active_effect_event`.
 *
 * The runtime forward-scans the parent's `EventsPerFrame` and reports the
 * most recent `EventEffect` whose `frame_index <= current parent frame`
 * for `slot_index`. When no event is active, the runtime falls back to the
 * part's `InitialEvents.effect` (if present); if neither exists, returns
 * SS6 default semantics: `is_synthetic_default = true`, `start_time = 0`,
 * `speed = 1.0`, `independent = false`.
 */
typedef struct ss_effect_event_info {
  /**
   * Parent `frame_index` this `EventEffect` was emitted at; 0 for synthetic default
   * or `InitialEvents.effect` fallback.
   */
  int32_t event_frame;
  int32_t start_time;
  float speed;
  bool independent;
  /**
   * `true` = no real `EventEffect` matched (either `InitialEvents.effect` fallback
   * or full SS6 default), `false` = real `EventEffect` matched.
   */
  bool is_synthetic_default;
} ss_effect_event_info;

/**
 * Result of `InstanceSlot::step` / `ss_instance_slot_step`.
 *
 * `transitioned == true` indicates a new EventInstance edge — `step` has
 * configured the child for the new key without starting it, so the caller
 * plays the child on this tick. Frame stepping is already applied inside
 * `step`, so callers only consume `child_frame_no` to redraw.
 */
typedef struct ss_instance_step_result {
  bool visible;
  bool transitioned;
  bool independent;
  float child_frame_no;
  bool child_looped;
} ss_instance_step_result;

/**
 * Resolved per-slot Instance event state, returned by
 * `ss_runtime_get_active_event_instance`.
 *
 * The runtime forward-scans the parent's `EventsPerFrame` and reports the
 * most recent `EventInstance` whose `frame_index <= current parent frame`
 * for `slot_index`. When no event is active, the function returns SS6
 * default semantics: `is_synthetic_default = true`, `loop_num = 1`,
 * `speed = 1.0`, all label hashes 0 (unset), all flags false.
 *
 * `start_label_hash` / `end_label_hash` are returned raw. They are
 * already resolved internally by `ss_instance_slot_step` against the
 * child runtime context, which keeps late binding intact across
 * dynamic-swap of the child ssab.
 */
typedef struct ss_event_instance_info {
  /**
   * Parent `frame_index` this `EventInstance` was emitted at; 0 for synthetic default.
   */
  int32_t event_frame;
  /**
   * 0 = unset → the child animation's start frame.
   */
  uint32_t start_label_hash;
  int32_t start_offset;
  /**
   * 0 = unset → the child animation's end frame.
   */
  uint32_t end_label_hash;
  int32_t end_offset;
  /**
   * -1 = infinite, >=1 = finite count.
   */
  int32_t loop_num;
  float speed;
  bool independent;
  bool reverse;
  bool pingpong;
  /**
   * `true` = SS6 default (no real event active), `false` = real `EventInstance`.
   */
  bool is_synthetic_default;
} ss_event_instance_info;



#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Register the function that receives a message for every failure the runtime
 * records, for the whole process. It runs on the thread of the failing call, and
 * `message` is valid only during it. `level` is one of `SS_RUNTIME_LOG_*`.
 */
void ss_runtime_set_log_callback(SsLogCallback callback);

/**
 * Create a resource from a copy of the `len` bytes at `ptr`, which may be freed
 * on return. Release it with `ss_resource_destroy`. Returns null for a null `ptr`
 * or a pack of another schema version.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * `ptr` must point to `len` readable bytes.
 */
void *ss_resource_create_copy(const unsigned char *ptr,
                              uintptr_t len);

/**
 * Create a resource that reads the `len` bytes at `ptr` in place, with no copy.
 * Release it with `ss_resource_destroy`. Returns null for a null `ptr` or a pack
 * of another schema version.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * `ptr` must point to `len` readable bytes, which stay valid and unmodified until
 * this handle is destroyed and no context or effect slot holds the resource.
 */
void *ss_resource_create_borrow(const unsigned char *ptr,
                                uintptr_t len);

/**
 * Release this handle. Null is ignored. A context or effect slot bound to the
 * resource keeps its own share, so this is safe while they live.
 */
void ss_resource_destroy(void *resource);

/**
 * Create an empty context. Bind a resource and set up an animation before playing.
 * Release it with `ss_runtime_destroy`.
 */
void *ss_runtime_create(void);

/**
 * Release a context and its share of the bound resource. Null is ignored.
 */
void ss_runtime_destroy(void *context);

/**
 * Return a context to its just-created state: the resource is unbound, and the
 * last error, the overrides and the playback and coordinate settings are cleared.
 */
void ss_runtime_reset(void *context);

/**
 * Why the last call that tracks a reason failed, as a stable code: `1`
 * `EmptyData`, `2` `InvalidVersion`, `3` `AnimationNotFound`, `4` `NotLoaded`,
 * `5` `InvalidParameter`, `6` `MemoryError`, `7` `MalformedEffect`, `99` `Unknown`.
 * `0` means the last such call succeeded, and so does a null `context`.
 *
 * Every fallible entry point that has a context to write to clears this on
 * entry, so it always describes the most recent one rather than some earlier
 * failure. The exceptions are the constructors that take no context
 * (`ss_resource_create_borrow` / `ss_resource_create_copy` and
 * `ss_effect_slot_create`), which have nowhere to record: a null return from
 * those is all there is.
 *
 * Reading does not clear it. Read it immediately after the call that returned
 * `false`, before making another on the same context.
 */
int32_t ss_runtime_get_last_error(void *context);

/**
 * The wording behind a code from `ss_runtime_get_last_error`, as
 * NUL-terminated UTF-8.
 *
 * **Takes the code, not a context**, so it is a pure lookup: no lifetime to
 * respect, nothing to free, and the same answer forever. An unrecognised code
 * — including `0`, which is "no error" — gives an empty string, so the result
 * goes straight into a log call without a branch.
 *
 * The text carries no numbers. `InvalidVersion` reads "Invalid SSAB version";
 * a Player that wants to say *which* versions should compare
 * `ss_runtime_ssab_schema_version` against the pack's own `version` field,
 * which gives it two integers rather than a sentence to parse.
 */
const char *ss_runtime_error_message(int32_t code);

/**
 * The `.ssab` schema version this runtime reads.
 *
 * The counterpart of `ss_converter_ssab_schema_version`, and what turns an
 * `InvalidVersion` failure into something a Player can say: compare it against
 * the `version` field of the pack it just failed to load.
 */
uint32_t ss_runtime_ssab_schema_version(void);

/**
 * Bind a resource to a context, which takes its own share of it. One resource can
 * be bound to several contexts, and `ss_resource_destroy` is safe while they are:
 * the resource lives until the last of them is destroyed, reset or bound to another.
 * A `_borrow` resource's buffer is the caller's to keep alive for as long.
 *
 * Drops the context's tracks and overrides; `ss_runtime_setup_animation` comes next.
 * Returns `false` with `InvalidParameter` for a null `resource`.
 */
bool ss_runtime_bind_resource(void *context, void *resource);

/**
 * Create a resource from a copy of the `len` bytes at `ptr` and bind it to
 * `context`, in one call, for a caller that needs no resource handle. Returns
 * `false` with `InvalidParameter` for a null `ptr`, or `InvalidVersion` for a pack
 * of another schema version.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * `ptr` must point to `len` readable bytes.
 */
bool ss_runtime_load_ssab_copy(void *context,
                               const unsigned char *ptr,
                               uintptr_t len);

/**
 * Create a resource that reads the `len` bytes at `ptr` in place and bind it to
 * `context`, in one call, for a caller that needs no resource handle. Returns
 * `false` with `InvalidParameter` for a null `ptr`, or `InvalidVersion` for a pack
 * of another schema version.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * `ptr` must point to `len` readable bytes, which stay valid and unmodified until
 * `context` is destroyed, reset or bound to another resource.
 */
bool ss_runtime_load_ssab_borrow(void *context,
                                 const unsigned char *ptr,
                                 uintptr_t len);

/**
 * The bound resource's `.ssab` bytes. Returns `false` and writes nothing when
 * no resource is bound (`NotLoaded`).
 */
bool ss_runtime_get_ssab(void *context, const unsigned char **out_data, uintptr_t *out_len);

/**
 * The bound resource's `.ssab` bytes, or null with `NotLoaded` when none is bound.
 */
const unsigned char *ss_runtime_get_ssab_buf(void *context);

/**
 * The length of `ss_runtime_get_ssab_buf`, or `0` with `NotLoaded` when none is bound.
 */
uintptr_t ss_runtime_get_ssab_len(void *context);

/**
 * Select the animation `name` of the bound resource and move to its first frame.
 * This does not start it; `ss_runtime_play` does. It drops every track but 0, the
 * section and the loop count, and keeps the speed, the frame rate, the frame skip
 * and the direction. Returns `false` with `NotLoaded`, `AnimationNotFound`, or
 * `InvalidParameter` for a null `name`.
 *
 * # Safety
 *
 * `name` must be null or a valid null-terminated string.
 */
bool ss_runtime_setup_animation(void *context, const char *name);

/**
 * `ss_runtime_setup_animation` by the hash `ss_runtime_hash_string` gives for the name.
 */
bool ss_runtime_setup_animation_by_hash(void *context, uint32_t name_hash);

/**
 * Build the FrameData of `frame_no`, which is in `[first frame, last frame + 1)`
 * of the animation, and write where it is to `out_data` and `out_len`.
 *
 * Returns `false` and writes nothing when no frame can be built, with the
 * reason on `ss_runtime_get_last_error`: `NotLoaded` with no resource bound,
 * `AnimationNotFound` with no animation set up, `InvalidParameter` for a
 * `frame_no` out of range.
 *
 * The buffer belongs to the context, and stays valid until the next
 * `ss_runtime_get_frame_data` on it, or until it is rebound, reset or destroyed.
 */
bool ss_runtime_get_frame_data(void *context,
                               float frame_no,
                               const unsigned char **out_data,
                               uintptr_t *out_len);

/**
 * Per-part world matrices for the current frame, written by
 * `ss_runtime_get_frame_data`: 16 floats per part, indexed by `part_index * 16`, with
 * the translation at `[12]` and `[13]`. The SceneGraph render mode does not compute them.
 */
void ss_runtime_get_world_matrices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-part UVs for the current frame, written by `ss_runtime_get_frame_data`,
 * matching `ss_runtime_get_local_vertices`:
 * `[lt.u, lt.v, rt.u, rt.v, lb.u, lb.v, rb.u, rb.v, center.u, center.v]` = 10 floats,
 * indexed by `part_index * 10`, in the cellmap's texture-pixel space. Divide by the
 * texture size to normalize.
 */
void ss_runtime_get_local_uvs(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-part resolved cell metadata for the current frame, written by
 * `ss_runtime_get_frame_data`. Layout:
 * `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]` per part,
 * indexed by `part_index * 6`. `(rect_left, rect_top)` is the cell
 * rectangle origin in the cellmap's texture pixel space, letting a
 * Player build a complete source rect (origin + size) without its own
 * cellmap lookup. Parts without a current cell carry zeros.
 */
void ss_runtime_get_cell_meta(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-part local-space vertex positions for the current frame, written by
 * `ss_runtime_get_frame_data`. Layout per part:
 * `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
 * = 10 floats, indexed by `part_index * 10`. Coordinates are in part-local
 * space with `coordinate_system` (Y-Up/Y-Down) flip already applied, so a
 * Player just multiplies them by the world matrix. Includes pivot, image
 * flip, and per-frame vertex deformation. Parts without a current cell
 * carry zeros. Replaces `FrameData::vertices` for renderers that consume
 * pre-resolved SoA buffers.
 */
void ss_runtime_get_local_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-part shape vertex positions for the current frame, written by
 * `ss_runtime_get_frame_data`. Layout: 12 vertices × `(x, y)` = 24 floats
 * (`SHAPE_VERTICES_STRIDE`) per part, indexed by `part_index * 24`.
 * Vertices are in local space, post-pivot, post-coord-system, so a Player
 * just multiplies them by the world matrix. Only the first
 * `ss_runtime_get_shape_vertex_counts(...)[part_index]` vertices are
 * valid; remaining slots are zero-initialized. Non-shape parts carry zero
 * count and zero-filled vertex slots. When no part of the pack is a shape,
 * the buffer is empty, as is `ss_runtime_get_shape_vertex_box_coords`.
 */
void ss_runtime_get_shape_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-vertex normalized bounding-box coordinates paired with
 * `ss_runtime_get_shape_vertices`. Layout: 12 vertices × `(fx, fy)` = 24
 * floats (`SHAPE_VERTICES_STRIDE`) per part, indexed by `part_index * 24`.
 * `(0, 0)` corresponds to the LT corner of the shape's pre-pivot bounding
 * box, `(1, 1)` to the RB corner, regardless of pivot offset or coordinate
 * system. Players use this to interpolate the four part-color corners
 * without re-deriving the box layout.
 */
void ss_runtime_get_shape_vertex_box_coords(void *context,
                                            const float **out_data,
                                            uintptr_t *out_len);

/**
 * Per-part valid vertex count for `ss_runtime_get_shape_vertices`. Indexed
 * by `part_index`. Zero for parts that are not shape parts (or shape
 * parts whose size collapsed to zero).
 */
void ss_runtime_get_shape_vertex_counts(void *context,
                                        const int32_t **out_data,
                                        uintptr_t *out_len);

/**
 * Mesh per-vertex X coordinates in world space, concatenated across all
 * parts. Pair with `ss_runtime_get_mesh_vertex_offsets` to slice per-part
 * ranges: part `p`'s vertices live at indices `offsets[p]..offsets[p + 1]`.
 * Non-mesh parts contribute an empty range. SceneGraph render mode leaves the
 * buffer empty (length 0).
 */
void ss_runtime_get_mesh_vertices_x(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Mesh per-vertex Y coordinates. SoA companion to
 * `ss_runtime_get_mesh_vertices_x`; sliced identically via
 * `ss_runtime_get_mesh_vertex_offsets`.
 */
void ss_runtime_get_mesh_vertices_y(void *context, const float **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) describing where each
 * part's mesh vertices live inside `ss_runtime_get_mesh_vertices_x`
 * / `_y`. Part `p` occupies indices `[offsets[p], offsets[p + 1])`; non-mesh
 * parts have `offsets[p] == offsets[p + 1]`.
 */
void ss_runtime_get_mesh_vertex_offsets(void *context,
                                        const uint32_t **out_data,
                                        uintptr_t *out_len);

/**
 * Mesh per-vertex UV coordinates in cellmap texture-pixel space, interleaved
 * `[u, v, ...]`, concatenated across all parts. Static — resolved once from
 * each mesh part's frame-0 Cell. Sliced per-part by
 * `ss_runtime_get_mesh_vertex_offsets`, identically to the vertex buffers.
 */
void ss_runtime_get_mesh_uvs(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Mesh triangle indices (part-local, 0-based), concatenated across all parts.
 * Static. Slice part `p`'s range via `ss_runtime_get_mesh_index_offsets`:
 * `[offsets[p], offsets[p + 1])`.
 */
void ss_runtime_get_mesh_indices(void *context, const int32_t **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
 * `ss_runtime_get_mesh_indices`. Part `p` occupies `[offsets[p], offsets[p + 1])`;
 * non-mesh parts have an empty range.
 */
void ss_runtime_get_mesh_index_offsets(void *context,
                                       const uint32_t **out_data,
                                       uintptr_t *out_len);

/**
 * Nine-Slice per-vertex positions for the current frame, interleaved
 * `[x, y, ...]` and concatenated across all parts, written by
 * `ss_runtime_get_frame_data`. Slice per part with
 * `ss_runtime_get_nines_vertex_offsets`; parts drawn by another
 * `DrawBatchKind` occupy an empty range.
 *
 * Coordinates are part-local with pivot, image flip and the
 * `coordinate_system` (Y-Up/Y-Down) flip already applied — exactly like
 * `ss_runtime_get_local_vertices` — so a Player just multiplies them by the
 * part's world matrix. The grid is a 4x4 sheet for `nines_fill_mode == 0`
 * (stretch) and independent quads per repeat for `== 1` (tile).
 */
void ss_runtime_get_nines_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Nine-Slice per-vertex UVs in cellmap texture-pixel space, interleaved
 * `[u, v, ...]`. Shares the vertex CSR of
 * `ss_runtime_get_nines_vertex_offsets`. Divide by the texture size to
 * normalize, the same as `ss_runtime_get_local_uvs`.
 */
void ss_runtime_get_nines_uvs(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Nine-Slice per-vertex box coordinates, interleaved `[fx, fy, ...]` with
 * `0` at the part's left / top edge and `1` at its right / bottom edge.
 * Shares the vertex CSR of `ss_runtime_get_nines_vertex_offsets`.
 *
 * Use them to bilinearly interpolate the four `part_color` corners across the
 * grid — the same weighting `ss_runtime_get_shape_vertex_box_coords` provides
 * for Shape parts. A part flip is already folded in, so `fx` / `fy` always
 * describe the drawn box.
 */
void ss_runtime_get_nines_box_coords(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Nine-Slice triangle indices, concatenated across all parts and sliced by
 * `ss_runtime_get_nines_index_offsets`. Values are **part-local**
 * (`0..vertex_count`), so a Player passes them alongside the part's vertex
 * slice without rebasing. Each quad is wound `LT, RT, LB / LB, RT, RB`.
 */
void ss_runtime_get_nines_indices(void *context, const int32_t **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) over the vertex buffers
 * `ss_runtime_get_nines_vertices` / `_uvs` / `_box_coords`. Part `p` occupies
 * vertices `[offsets[p], offsets[p + 1])` — multiply by 2 to index the
 * interleaved floats. Non-nines parts have an empty range.
 */
void ss_runtime_get_nines_vertex_offsets(void *context,
                                         const uint32_t **out_data,
                                         uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
 * `ss_runtime_get_nines_indices`. Part `p` occupies
 * `[offsets[p], offsets[p + 1])`; non-nines parts have an empty range.
 */
void ss_runtime_get_nines_index_offsets(void *context,
                                        const uint32_t **out_data,
                                        uintptr_t *out_len);

/**
 * The playhead of track 0's main animation, with its fraction: what
 * `ss_runtime_update` last returned, or where a seek put it.
 */
float ss_runtime_get_frame_no(void *context);

/**
 * Returns `true` once the configured loops have completed, staying `true`
 * until the next `ss_runtime_play` / `ss_runtime_setup_animation`. Always
 * `false` under an infinite loop count, and never raised by
 * `ss_runtime_stop`.
 *
 * This is a *state*, not a pulse: to notify once, take its rising edge.
 */
bool ss_runtime_is_finished(void *context);

/**
 * `true` for the single `ss_runtime_update` that took playback back to the start of
 * its cycle; the next update clears it. Never raised on the last cycle of a finite
 * loop count, which finishes instead.
 */
bool ss_runtime_just_looped(void *context);

/**
 * `true` while the clock runs and track 0 has not finished. A paused character is still playing.
 */
bool ss_runtime_is_playing(void *context);

/**
 * Whether track 0's playhead is moving forward now. A ping-pong's return leg reads
 * `false`, as does a backward animation's outward leg.
 */
bool ss_runtime_is_playing_forward(void *context);

/**
 * `true` while a playing character is held by `ss_runtime_pause`. Once track 0 has
 * finished this reads `false`; `ss_runtime_is_clock_pausing` still reads the hold.
 */
bool ss_runtime_is_pausing(void *context);

/**
 * `true` from `ss_runtime_play` until `ss_runtime_stop` or the next setup:
 * the character's clock, which stays `true` after track 0 finishes while
 * the other tracks and the fades run on. `ss_runtime_is_playing` reads
 * track 0's state instead.
 */
bool ss_runtime_is_clock_playing(void *context);

/**
 * `true` while `ss_runtime_pause` holds a playing clock, after track 0 has
 * finished as well. Implies `ss_runtime_is_clock_playing`.
 */
bool ss_runtime_is_clock_pausing(void *context);

/**
 * `true` while the track stack would move the pose on a running clock for a
 * reason other than track 0's own animation stepping: a fade on any track, or
 * another source still playing on a track that is heard. Independent of the
 * clock; read it with `ss_runtime_is_clock_playing` and
 * `ss_runtime_is_clock_pausing`.
 */
bool ss_runtime_is_stack_in_motion(void *context);

/**
 * What `ss_runtime_set_skip_frames` stored.
 */
bool ss_runtime_get_skip_frames(void *context);

/**
 * 0 = forward, 1 = backward.
 */
int32_t ss_runtime_get_playback_direction(void *context);

/**
 * 0 = normal, 1 = ping-pong.
 */
int32_t ss_runtime_get_playback_style(void *context);

/**
 * `-1` for an animation that loops forever, otherwise the cycles it plays.
 */
int32_t ss_runtime_get_loop_count(void *context);

/**
 * The cycles left, `0` once finished; `-1` for an animation that loops forever.
 */
int32_t ss_runtime_get_remaining_loops(void *context);

/**
 * The first frame of the playback range: the section's when one is set, otherwise the animation's.
 */
int32_t ss_runtime_get_start_frame(void *context);

/**
 * The last frame of the playback range: the section's when one is set, otherwise the animation's.
 */
int32_t ss_runtime_get_end_frame(void *context);

/**
 * The rate track 0's animation plays at, in frames per second: the override when one
 * is set, otherwise the rate it was authored at.
 */
int32_t ss_runtime_get_frame_rate(void *context);

/**
 * The playback speed in effect, as a multiple of `ss_runtime_get_frame_rate`.
 *
 * Reads back what `ss_runtime_set_speed_scale` stored, which is not
 * always what it was handed: a negative rate clamps to `0.0`.
 */
float ss_runtime_get_speed_scale(void *context);

/**
 * `true` (the default) advances by the time elapsed, skipping frames to keep up.
 * `false` advances one frame at a time, at most one per update.
 */
void ss_runtime_set_skip_frames(void *context, bool skip);

/**
 * `n > 0` plays n cycles and then finishes; `0` or a negative count loops forever.
 * `ss_runtime_setup_animation` resets it to looping forever, so set it after.
 */
void ss_runtime_set_loop_count(void *context, int loop_count);

/**
 * Play at `frame_rate` frames per second instead of the authored rate; `0` or a
 * negative rate clears the override. It survives `ss_runtime_setup_animation`.
 */
void ss_runtime_set_frame_rate(void *context, int frame_rate);

/**
 * `direction`: 0 = forward, otherwise backward. `style`: 0 = normal, otherwise
 * ping-pong. Call it before `ss_runtime_play`. It survives `ss_runtime_setup_animation`.
 */
void ss_runtime_set_playback_direction(void *context, int direction, int style);

/**
 * Playback speed as a multiple of the frame rate. `0.0` stops the playhead while
 * `ss_runtime_is_playing` stays `true`, and a negative speed clamps to `0.0`; direction
 * is `ss_runtime_set_playback_direction`'s. It survives `ss_runtime_setup_animation`.
 */
void ss_runtime_set_speed_scale(void *context, float speed_scale);

/**
 * Limits playback to `[start_frame, end_frame]`, both ends included. An end
 * out of the animation's range (`-1`, say) is the animation's edge on that
 * side. Returns `false`, with `InvalidParameter` for
 * `ss_runtime_get_last_error`, when both ends are in range and the end is
 * before the start; the section and the playhead stay as they were.
 */
bool ss_runtime_set_animation_section(void *context, int start_frame, int end_frame);

/**
 * Advance by `delta` **milliseconds** and return the frame to pass to
 * `ss_runtime_get_frame_data`. With animation mixing it advances every track and fade;
 * the frame returned is track 0's.
 *
 * `delta` must not be negative. A negative value does not play backwards, and where it
 * leaves the playhead is unspecified. Clamp it to 0 first: a browser's
 * `requestAnimationFrame` timestamp can be earlier than the `performance.now()` read
 * before it.
 */
float ss_runtime_update(void *context, float delta);

/**
 * Start track 0 from the start of its range (the end when playing backward), with its
 * loop count re-armed, and start the clock the other tracks and fades run on.
 */
void ss_runtime_play(void *context);

/**
 * `ss_runtime_play`, starting from `frame_no`. A negative `frame_no` starts where `ss_runtime_play` does.
 */
void ss_runtime_play_with_start_frame(void *context,
                                      float frame_no);

/**
 * Holds playback where it stands, keeping the current frame. With animation
 * mixing it holds every track and every fade, as `ss_runtime_stop` stops them.
 *
 * Idempotent: pausing an already-paused animation leaves it paused.
 * `ss_runtime_is_playing` stays true while held — a pause is a hold, not a
 * stop — so tell the two apart with `ss_runtime_is_pausing`. Lift the hold
 * with `ss_runtime_resume`; `ss_runtime_play` rewinds and cannot resume.
 */
void ss_runtime_pause(void *context);

/**
 * Lifts a `ss_runtime_pause`, carrying on from the frame it stopped at.
 *
 * Idempotent, and a no-op on an animation that was never paused. It does not
 * start a stopped animation; use `ss_runtime_play` for that.
 */
void ss_runtime_resume(void *context);

/**
 * Stop every track and fade where they stand. Nothing rewinds; `ss_runtime_play` starts them again.
 */
void ss_runtime_stop(void *context);

/**
 * Seek track 0 to `frame`, clamped to the playback range. The events of the frames in between do not fire.
 */
void ss_runtime_set_frame_no(void *context,
                             float frame);

/**
 * Step track 0 one whole frame forward, whatever the direction and style. Does nothing on the range's last frame.
 */
void ss_runtime_next_frame(void *context);

/**
 * Step track 0 one whole frame back, whatever the direction and style. Does nothing on the range's first frame.
 */
void ss_runtime_prev_frame(void *context);

/**
 * Put `name` on `track`, replacing what it played and cancelling its fade. Tracks
 * grow on demand. Returns `false` with `NotLoaded`, `AnimationNotFound` (an unknown
 * name, or no `ss_runtime_setup_animation` yet), or `InvalidParameter` for a null
 * `name`.
 *
 * # Safety
 * `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
 */
bool ss_runtime_set_track_animation(void *context, uint32_t track, const char *name);

/**
 * `ss_runtime_set_track_animation` by the hash of the name.
 */
bool ss_runtime_set_track_animation_by_hash(void *context, uint32_t track, uint32_t hash);

/**
 * The strength of `track`: `0` contributes nothing, `1` all of it, and other values
 * extrapolate. Returns `false` with `InvalidParameter` for a `NaN` or a track that does
 * not exist.
 */
bool ss_runtime_set_track_alpha(void *context, uint32_t track, float alpha);

/**
 * Returns `NaN` for a track that does not exist, so callers can tell it apart
 * from a valid `alpha` of 0.
 */
float ss_runtime_get_track_alpha(void *context, uint32_t track);

/**
 * `blend` is one of `SS_RUNTIME_TRACK_BLEND_*`.
 */
bool ss_runtime_set_track_blend(void *context, uint32_t track, uint32_t blend);

/**
 * Returns `SS_RUNTIME_INVALID_HANDLE` for a track that does not exist.
 */
uint32_t ss_runtime_get_track_blend(void *context, uint32_t track);

/**
 * `sync` is one of `SS_RUNTIME_TRACK_SYNC_*`. Set it before laying an
 * animation on the track; a track with nothing on it yet is created for it.
 */
bool ss_runtime_set_track_sync(void *context, uint32_t track, uint32_t sync);

/**
 * Returns `SS_RUNTIME_INVALID_HANDLE` for a track that does not exist.
 */
uint32_t ss_runtime_get_track_sync(void *context, uint32_t track);

/**
 * How many tracks exist, silent ones included; at least 1.
 */
uint32_t ss_runtime_get_track_count(void *context);

/**
 * Silence `track`: its sources are released and its fade cancelled. Track 0 cannot be
 * cleared (`InvalidParameter`); fade it to alpha 0 instead.
 */
bool ss_runtime_clear_track(void *context, uint32_t track);

/**
 * Fade `track` to `name` over `duration_ms` — the same time unit as
 * `ss_runtime_update`'s delta. A `track` that does not exist yet is created.
 * Returns `false` with `InvalidParameter` for a `duration_ms` that is not positive,
 * a `NaN` or a null `name`, and with `AnimationNotFound` for a `name` the pack does
 * not have or before `ss_runtime_setup_animation`.
 *
 * # Safety
 * `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
 */
bool ss_runtime_crossfade(void *context, uint32_t track, const char *name, float duration_ms);

/**
 * `ss_runtime_crossfade` by the hash of the name.
 */
bool ss_runtime_crossfade_by_hash(void *context, uint32_t track, uint32_t hash, float duration_ms);

/**
 * Drive the alpha of `track` to `target_alpha` over `duration_ms`, the same time unit
 * as `ss_runtime_update`'s delta. Returns `false` with `InvalidParameter` for a
 * `duration_ms` that is not positive, a `NaN`, or a track that does not exist.
 */
bool ss_runtime_fade_track(void *context, uint32_t track, float target_alpha, float duration_ms);

/**
 * Progress in `[0, 1]` of the fade on `track`; `NaN` when it is not fading.
 */
float ss_runtime_get_track_fade_progress(void *context, uint32_t track);

/**
 * Cancel the fade on `track`, leaving its alpha and weights where they stand.
 */
bool ss_runtime_cancel_track_fade(void *context, uint32_t track);

/**
 * Seek the animation on `track`, which is how an axis authored as an animation is
 * driven. Returns `false` with `InvalidParameter` when the track has no animation.
 */
bool ss_runtime_set_track_frame(void *context, uint32_t track, float frame);

/**
 * Returns `NaN` when the track has no animation on it.
 */
float ss_runtime_get_track_frame(void *context, uint32_t track);

/**
 * The speed of `track`, as a multiple of each animation's frame rate. The track keeps
 * it across a crossfade or a new animation. Track 0's is `ss_runtime_set_speed_scale`.
 */
bool ss_runtime_set_track_speed(void *context, uint32_t track, float speed);

/**
 * What `ss_runtime_set_track_speed` stored (a negative speed reads back as
 * `0.0`); `NaN` for a track that does not exist.
 */
float ss_runtime_get_track_speed(void *context, uint32_t track);

/**
 * The rate in frames per second `track` plays its animations at; `0` or a negative
 * rate clears it. The track keeps it, as it does its speed.
 */
bool ss_runtime_set_track_frame_rate(void *context, uint32_t track, int frame_rate);

/**
 * The rate the track's animation plays at; `0` for a track with no animation
 * and no override, `-1` for a track that does not exist.
 */
int32_t ss_runtime_get_track_frame_rate(void *context, uint32_t track);

/**
 * `ss_runtime_set_skip_frames` for `track`. The track keeps it, as it does its speed.
 */
bool ss_runtime_set_track_skip_frames(void *context, uint32_t track, bool skip);

/**
 * `false` for a track that does not exist.
 */
bool ss_runtime_get_track_skip_frames(void *context, uint32_t track);

/**
 * `direction`: 0 = forward, otherwise backward. `style`: 0 = normal,
 * otherwise ping-pong; as `ss_runtime_set_playback_direction`.
 */
bool ss_runtime_set_track_playback_direction(void *context,
                                             uint32_t track,
                                             int direction,
                                             int style);

/**
 * 0 = forward, 1 = backward; `-1` for a track that does not exist.
 */
int32_t ss_runtime_get_track_playback_direction(void *context, uint32_t track);

/**
 * 0 = normal, 1 = ping-pong; `-1` for a track that does not exist.
 */
int32_t ss_runtime_get_track_playback_style(void *context, uint32_t track);

/**
 * How many times the track's animation plays before it holds its last frame;
 * `0` or a negative count loops forever. It belongs to the animation on the
 * track: laying another one there starts it looping forever again.
 */
bool ss_runtime_set_track_loop_count(void *context, uint32_t track, int loop_count);

/**
 * `-1` for a track that loops forever; `0` for a track with no animation.
 */
int32_t ss_runtime_get_track_loop_count(void *context, uint32_t track);

/**
 * `true` once the track's animation has played its loop count out and holds
 * its last frame, as `ss_runtime_is_finished` is for track 0. `false` for a
 * track with no animation.
 */
bool ss_runtime_is_track_finished(void *context, uint32_t track);

/**
 * `true` for the update in which the track's animation wrapped back to its
 * start, as `ss_runtime_just_looped` is for track 0.
 */
bool ss_runtime_track_just_looped(void *context, uint32_t track);

/**
 * How far into a crossfade on `track` the outgoing side keeps firing events:
 * it fires while its share of the track, `1 - t`, is at least `threshold`.
 * The default, `+inf`, keeps it silent from the moment the fade begins.
 */
bool ss_runtime_set_track_outgoing_event_threshold(void *context, uint32_t track, float threshold);

/**
 * `NaN` for a track that does not exist.
 */
float ss_runtime_get_track_outgoing_event_threshold(void *context, uint32_t track);

/**
 * Handle of the source carrying the track's animation, so it can be given a
 * weight once the track holds several. `SS_RUNTIME_INVALID_HANDLE` when the
 * track has no animation.
 */
uint32_t ss_runtime_get_track_main_source(void *context, uint32_t track);

/**
 * Add another weighted source to `track` (locomotion-style blending). Returns
 * `SS_RUNTIME_INVALID_HANDLE` on failure.
 *
 * # Safety
 * `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
 */
uint32_t ss_runtime_add_track_source(void *context, uint32_t track, const char *name, float weight);

/**
 * `ss_runtime_add_track_source` by the hash of the name.
 */
uint32_t ss_runtime_add_track_source_by_hash(void *context,
                                             uint32_t track,
                                             uint32_t hash,
                                             float weight);

/**
 * The proportion of a source within its track; `ss_runtime_set_track_alpha` is the
 * strength of the track as a whole. Returns `false` with `InvalidParameter` for an
 * invalid or released handle.
 */
bool ss_runtime_set_source_weight(void *context, uint32_t handle, float weight);

/**
 * Returns `NaN` for an invalid or released handle, so `weight == 0` stays
 * distinguishable from "no such source".
 */
float ss_runtime_get_source_weight(void *context, uint32_t handle);

/**
 * Seek one source. Returns `false` with `InvalidParameter` for an invalid or released handle.
 */
bool ss_runtime_set_source_frame(void *context, uint32_t handle, float frame);

/**
 * Which animation the source `handle` plays, as an index into the ssab's
 * `animations()`; `-1` for a handle that names no source. An event index
 * addresses the `events()` of the animation of the source that fired it
 * (`ss_runtime_get_passed_event_source_handle`), track 0's included: after a
 * crossfade on track 0 that is the incoming animation.
 */
int32_t ss_runtime_get_source_animation_index(void *context, uint32_t handle);

/**
 * Which animation track 0's main source plays, as an index into the ssab's
 * `animations()`: a crossfade or a track animation on track 0 changes it from
 * the call on. `-1` before anything is set up.
 */
int32_t ss_runtime_get_current_animation_index(void *context);

/**
 * The playhead of a source; `NaN` for an invalid or released handle.
 */
float ss_runtime_get_source_frame(void *context, uint32_t handle);

/**
 * A source fires events once its weight reaches `threshold`. The default, `+inf`,
 * keeps every source silent but track 0's main one, which always fires. Returns `false`
 * with `InvalidParameter` for an invalid or released handle.
 */
bool ss_runtime_set_source_event_threshold(void *context, uint32_t handle, float threshold);

/**
 * `NaN` for an invalid or released handle.
 */
float ss_runtime_get_source_event_threshold(void *context, uint32_t handle);

/**
 * Create a new EffectSlot bound to `resource`. Returns null on failure
 * (resource not loaded, effect_name_hash not found, malformed effect data).
 *
 * The slot holds its own reference to `resource`, so `ss_resource_destroy`
 * may be called while the slot is still alive, exactly as it may while a
 * Context is still bound.
 */
void *ss_effect_slot_create(const void *resource, uint32_t effect_name_hash, uint32_t seed);

/**
 * Release a slot and its share of the resource. Null is ignored.
 */
void ss_effect_slot_destroy(void *slot);

/**
 * Reset the slot's playback / "last started key" memory to its just-created
 * state without destroying it. Call on an explicit head-out (`ss_runtime_play`
 * / replay) so a re-played animation re-fires its effect from scratch instead
 * of debouncing the first key as the same one the slot last started. The
 * resource binding and emitter caches are preserved.
 */
void ss_effect_slot_reset(void *slot);

/**
 * Advance the slot one frame. The caller resolves the active effect event
 * (via `ss_runtime_get_active_effect_event`) and feeds it here together with
 * timing inputs. `parent_looped` re-arms transition detection across the
 * parent's loop edge.
 *
 * `y_flip = true` mirrors each particle's Y position, rotation sign and quad, for
 * Y-down rendering of Y-up authored content (Godot, pixi.js). Pass `false` for
 * Y-up engines (Unity 2D).
 *
 * `vert_stride` selects the vertex layout of the returned `EffectDrawPlan`:
 * `2` packs (x, y) pairs (Godot, pixi.js, wgpu); `3` packs (x, y, 0) triples
 * (Unity Mesh, Unreal procedural mesh). Other values fall back to `2`.
 *
 * `draw` is whether the part is in this frame's `draw_order`. Step a hidden
 * part's slot anyway, with `draw = false`: it keeps the effect's clock and
 * transitions moving and builds no plan, and the next drawn step builds the
 * plan an always-drawn slot would. A caller that steps before it has this
 * frame's `draw_order` passes `true`.
 *
 * Returns `EffectDrawPlan` FlatBuffer pointer + length when visible. Buffer
 * is valid only until the next call on the same slot.
 */
struct ss_effect_step_result ss_effect_slot_step(void *slot,
                                                 struct ss_effect_event_info ev,
                                                 float parent_frame,
                                                 float delta_seconds,
                                                 float fps,
                                                 bool parent_looped,
                                                 bool y_flip,
                                                 uint8_t vert_stride,
                                                 bool draw);

/**
 * The seed the slot was created with; `0` for a null slot.
 */
uint32_t ss_effect_slot_get_seed(void *slot);

/**
 * Whether the slot is currently running independently of the parent's frame.
 * Reflects the most recently seen `EventEffect.independent`. Callers use
 * this to decide whether to schedule continuous tick updates regardless of
 * parent frame change.
 */
bool ss_effect_slot_is_independent(void *slot);

/**
 * Create a fresh InstanceSlot. Slots are stateless at construction (no
 * resource binding) — `step` operates against a child runtime context
 * supplied per call.
 */
void *ss_instance_slot_create(void);

/**
 * Release a slot. Null is ignored.
 */
void ss_instance_slot_destroy(void *slot);

/**
 * Reset the slot's "last started key" memory to its just-created state without
 * destroying it. Call on an explicit head-out (`ss_runtime_play` / replay) so a
 * re-played animation re-fires the instance child from scratch instead of
 * debouncing the first key as the same one the slot last started.
 */
void ss_instance_slot_reset(void *slot);

/**
 * Drive the instance slot for one tick. The caller resolves the active
 * EventInstance (via `ss_runtime_get_active_event_instance` against the
 * *parent* runtime context) and passes the *child* runtime context here so
 * that label resolution and frame stepping happen against the right Context.
 * Returns `transitioned == true` on EventInstance edges, having configured the
 * child for the new key without starting it: the caller then calls
 * `ss_runtime_play` on the child (an independent child never advances
 * otherwise). Draw the child at `child_frame_no` on that tick too: the slot
 * already put it where `play` snaps it, and a synced child's value follows the
 * parent, which reading the frame back after `play` would lose.
 */
struct ss_instance_step_result ss_instance_slot_step(void *slot,
                                                     struct ss_event_instance_info ev,
                                                     void *child_ctx,
                                                     float parent_frame,
                                                     float delta_seconds,
                                                     bool parent_looped);

/**
 * Whether the slot's current key plays the child on its own time rather than the parent's frame.
 */
bool ss_instance_slot_is_independent(void *slot);

/**
 * The speed the most recent `ss_instance_slot_step` drove the child at, `1.0`
 * before the first or for a null slot. Usually the key's `speed`, but while
 * the slot ignores the implicit frame-0 key it keeps the last real key's, so
 * the event the host resolved says `1.0` when the child is not running at it.
 * Scale the elapsed time handed to the child's own instance and effect slots
 * by its absolute value, not by the resolved event's `speed`.
 */
float ss_instance_slot_get_speed(void *slot);

/**
 * Set the render mode. `mode` is interpreted as:
 *   0 = Direct       (pre-bake world matrices; default)
 *   1 = SceneGraph   (skip world matrix pre-computation)
 * Unknown values fall back to Direct.
 */
void ss_runtime_set_render_mode(void *context, uint8_t mode);

/**
 * 0 = Direct, 1 = SceneGraph.
 */
uint8_t ss_runtime_get_render_mode(void *context);

/**
 * 0 = Y-up (the default), 1 = Y-down, which flips Y coordinates and rotation. Any other value is Y-up.
 */
void ss_runtime_set_coordinate_system(void *context,
                                      int32_t coordinate_system);

/**
 * 0 = Y-up, 1 = Y-down.
 */
int32_t ss_runtime_get_coordinate_system(void *context);

/**
 * 0 = the canvas center (the default), 1 = its top-left corner, which offsets root
 * parts by half the canvas. Any other value is the center.
 */
void ss_runtime_set_origin(void *context, int32_t origin);

/**
 * 0 = the canvas center, 1 = its top-left corner.
 */
int32_t ss_runtime_get_origin(void *context);

/**
 * 0 = pixels (the default), 1 = normalized, which divides each part's position by the
 * canvas size. Sizes, local vertices and UVs stay in pixels. Any other value is pixels.
 */
void ss_runtime_set_unit(void *context, int32_t unit);

/**
 * 0 = pixels, 1 = normalized.
 */
int32_t ss_runtime_get_unit(void *context);

/**
 * How many events the last `ss_runtime_update` passed. They stay until the next one,
 * `ss_runtime_setup_animation`, `ss_runtime_bind_resource` or `ss_runtime_reset`.
 */
int32_t ss_runtime_get_passed_event_count(void *context);

/**
 * Collected event `index`, as an index into the `events()` of the animation of the
 * source that fired it (`ss_runtime_get_passed_event_source_handle`); `-1` out of range.
 */
int32_t ss_runtime_get_passed_event_index(void *context, int32_t index);

/**
 * Which source fired the collected event at `index` — parallel to
 * `ss_runtime_get_passed_event_index`, and read with the same index.
 *
 * With every source and every track's outgoing side at the default event
 * threshold (`+∞`) this is uniformly `ss_runtime_get_track_main_source` of
 * track 0, so a Player that does not use animation mixing can ignore it. It
 * diverges once a secondary source opts in to firing events via
 * `ss_runtime_set_source_event_threshold`, or a crossfade's outgoing side via
 * `ss_runtime_set_track_outgoing_event_threshold`, and that is when a host
 * needs it: without it, two blended animations' events arrive in one
 * undifferentiated list.
 *
 * Returns `SS_RUNTIME_INVALID_HANDLE` for a negative or out-of-range `index`,
 * which no live handle ever equals.
 */
uint32_t ss_runtime_get_passed_event_source_handle(void *context, int32_t index);

/**
 * Resolve the active `EventInstance` for `slot_index` on the parent's
 * currently selected animation, at the current frame. Returns SS6
 * default semantics (`is_synthetic_default = true`, `loop_num = 1`,
 * `speed = 1.0`) when no event is active, so callers don't need to
 * branch on the no-event case.
 *
 * `start_label_hash` / `end_label_hash` are returned raw. They are
 * already resolved internally by `ss_instance_slot_step` for the
 * child runtime — most Players should consume the slot helper rather
 * than resolving labels themselves.
 */
struct ss_event_instance_info ss_runtime_get_active_event_instance(void *context,
                                                                   uint32_t slot_index);

/**
 * Resolve the active `EventEffect` for `slot_index` on the parent's
 * currently selected animation, at the current frame. When no real event
 * has fired yet, falls back to `InitialEvents.effect` for the slot (with
 * `is_synthetic_default = true`); if neither exists, returns full SS6
 * defaults (`start_time = 0`, `speed = 1.0`, `independent = false`).
 */
struct ss_effect_event_info ss_runtime_get_active_effect_event(void *context, uint32_t slot_index);

/**
 * The FNV-1a hash the converter stores for a name, which the `_by_hash` calls
 * take. `0` for a null or non-UTF-8 string.
 *
 * # Safety
 *
 * `s` must be null or a valid null-terminated string.
 */
uint32_t ss_runtime_hash_string(const char *s);

/**
 * Resolve a part name to its `part_index` in the current primary animation.
 * Returns `-1` on null context / null or invalid name / not found.
 *
 * # Safety
 * `part_name` must be null or a valid null-terminated string.
 */
int32_t ss_runtime_find_part_index(void *context, const char *part_name);

/**
 * Set or clear a per-part visibility override.
 * `force`: `0` = clear/revert, `1` = force-hide. Anything else, `2` (reserved for
 * force-visible) included, returns `false` with `InvalidParameter`.
 * `cascade != 0` propagates a force-hide to descendants.
 */
bool ss_runtime_set_part_visibility_override(void *context,
                                             uint32_t part,
                                             unsigned char force,
                                             unsigned char cascade);

/**
 * Clear a part's visibility override (revert to animation).
 */
bool ss_runtime_clear_part_visibility_override(void *context, uint32_t part);

/**
 * Set a per-part color override from a single packed RGBA (`0xRRGGBBAA`, fills
 * all four corners). `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`. `priority_mode`:
 * `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=Permanent`. Normal
 * parts only.
 */
bool ss_runtime_set_part_color_override(void *context,
                                        uint32_t part,
                                        unsigned char blend_op,
                                        uint32_t rgba,
                                        unsigned char priority_mode);

/**
 * Set a per-part color override from four packed RGBA corners in ssruntime
 * order `lt, rt, lb, rb` (`rgba_ltrtlbrb` points to 4 `uint32` = `0xRRGGBBAA`).
 *
 * # Safety
 * `rgba_ltrtlbrb` must be null or point to 4 readable `uint32`.
 */
bool ss_runtime_set_part_color_override_corners(void *context,
                                                uint32_t part,
                                                unsigned char blend_op,
                                                const uint32_t *rgba_ltrtlbrb,
                                                unsigned char priority_mode);

/**
 * Clear a part's color override.
 */
bool ss_runtime_clear_part_color_override(void *context, uint32_t part);

/**
 * Set a per-part cell override, identified by FNV-1a hashes of the cellmap name
 * (without `.ssce`) and the cell name. `(0, 0)` reverts to the animation.
 * `priority_mode`: `0=OverwriteOnNextKeyframe 1=HoldUntilNextAnimation 2=
 * Permanent`. Normal + Mask parts only. Fails if the cell cannot be resolved.
 */
bool ss_runtime_set_part_cell_override(void *context,
                                       uint32_t part,
                                       uint32_t cellmap_name_hash,
                                       uint32_t cell_name_hash,
                                       unsigned char priority_mode);

/**
 * Clear a part's cell override.
 */
bool ss_runtime_clear_part_cell_override(void *context, uint32_t part);

/**
 * Clear every per-part override on the context.
 */
bool ss_runtime_clear_all_part_overrides(void *context);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SSRUNTIME_H */
