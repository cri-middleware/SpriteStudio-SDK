#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Whether a part's Skew (shear) inherits down to its children.
 * true == inherits down to children.
 * false == does not inherit.
 */
#define SKEW_PROPAGATES false

#define SEED_MAGIC 7573

#define LIFE_EXTEND_SCALE 8

#define LIFE_EXTEND_MIN 64

#define SHAPE_VERTICES_STRIDE 24

/**
 * Stride (number of `f32`s) per part in the `local_vertices` SoA buffer.
 * Layout: `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
 * in part-local space, with `coordinate_system` (Y-Up/Y-Down) flip already applied.
 */
#define LOCAL_VERTICES_STRIDE 10

/**
 * Runaway guard for [`emit_nines_geometry`]: repeats per axis are clamped to
 * this, and a centre whose repeat count exceeds [`NINES_MAX_TILE_QUADS`] falls
 * back to a single stretched quad. Authored content never reaches either — a
 * 4096 px part filled by a 16 px source slice needs 256 repeats — they only
 * stop degenerate data (a near-zero source slice) from exhausting memory.
 */
#define NINES_MAX_TILE_DIV 4096

/**
 * Companion to [`NINES_MAX_TILE_DIV`] for the two-axis centre region.
 */
#define NINES_MAX_TILE_QUADS (1 << 16)

/**
 * Severity passed to [`SsLogCallback`]. The runtime only ever emits `ERROR`
 * today; `WARN` and `INFO` are reserved so a host can switch on the level
 * without its arms going stale when the runtime starts using them.
 */
#define SS_RUNTIME_LOG_ERROR 1

/**
 * See [`SS_RUNTIME_LOG_ERROR`].
 */
#define SS_RUNTIME_LOG_WARN 2

/**
 * See [`SS_RUNTIME_LOG_ERROR`].
 */
#define SS_RUNTIME_LOG_INFO 3

/**
 * Sentinel returned by the handle-returning track functions on failure.
 */
#define SS_RUNTIME_INVALID_HANDLE UINT32_MAX

/**
 * Track blend operators, mirroring [`crate::TrackBlend`].
 */
#define SS_RUNTIME_TRACK_BLEND_MIX 0

#define SS_RUNTIME_TRACK_BLEND_ADDITIVE 1

/**
 * Where the animations laid on a track start, mirroring [`crate::TrackSync`].
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

void rust_eh_personality(void);

extern uint8_t *em_malloc(uintptr_t size);

extern void em_free(void *ptr);

extern int32_t em_posix_memalign(void **memptr, uintptr_t align, uintptr_t size);

extern uint8_t *ss_custom_alloc(uintptr_t size, uintptr_t align);

extern void ss_custom_free(void *ptr, uintptr_t size, uintptr_t align);

void ss_runtime_set_log_callback(SsLogCallback callback);

/**
 * Creates an AnimationResource by copying the provided data.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid memory block.
 */
void *ss_resource_create_copy(const unsigned char *ptr,
                              uintptr_t len);

/**
 * Creates an AnimationResource by borrowing the provided data.
 *
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid memory block of at least `len` bytes,
 * AND the memory must remain valid and unmodified until this resource is destroyed.
 */
void *ss_resource_create_borrow(const unsigned char *ptr,
                                uintptr_t len);

void ss_resource_destroy(void *resource);

void *ss_runtime_create(void);

void ss_runtime_destroy(void *context);

void ss_runtime_reset(void *context);

/**
 * Why the last call that tracks a reason failed, as the stable code from
 * [`crate::error::SSRuntimeError::as_code`]. `0` means the last such call
 * succeeded, and so does a null `context`.
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
 * The wording behind a code from [`ss_runtime_get_last_error`], as
 * NUL-terminated UTF-8.
 *
 * **Takes the code, not a context**, so it is a pure lookup: no lifetime to
 * respect, nothing to free, and the same answer forever. An unrecognised code
 * — including `0`, which is "no error" — gives an empty string, so the result
 * goes straight into a log call without a branch.
 *
 * The text carries no numbers. `InvalidVersion` reads "Invalid SSAB version";
 * a Player that wants to say *which* versions should compare
 * [`ss_runtime_ssab_schema_version`] against the pack's own `version` field,
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
 * Binds an AnimationResource to a Context.
 *
 * # Safety and Lifetimes
 * - To save memory, a single Resource can be bound to multiple Contexts.
 * - **Lifetime Safety:** This implementation uses `Arc` for internal reference counting.
 *   It is safe to destroy the `resource` (via `ss_resource_destroy`) even if it is still bound
 *   to contexts; the memory will be kept alive until all bound contexts are also destroyed or reset.
 */
bool ss_runtime_bind_resource(void *context,
                              void *resource);

/**
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid memory block.
 */
bool ss_runtime_load_ssab_copy(void *context,
                               const unsigned char *ptr,
                               uintptr_t len);

/**
 * # Security and Validation
 * To maximize performance and minimize binary size, `libssruntime` **does not validate** the input `.ssab` data.
 * Passing corrupted or maliciously crafted data will cause undefined behavior (e.g., out-of-bounds reads).
 * The caller MUST ensure the data is trusted or validated before passing it to this function.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid memory block.
 */
bool ss_runtime_load_ssab_borrow(void *context,
                                 const unsigned char *ptr,
                                 uintptr_t len);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_ssab(void *context, const unsigned char **out_data, uintptr_t *out_len);

const unsigned char *ss_runtime_get_ssab_buf(void *context);

uintptr_t ss_runtime_get_ssab_len(void *context);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid null-terminated string.
 */
bool ss_runtime_setup_animation(void *context, const char *name);

bool ss_runtime_setup_animation_by_hash(void *context, uint32_t name_hash);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_frame_data(void *context,
                               float frame_no,
                               const unsigned char **out_data,
                               uintptr_t *out_len);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_world_matrices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
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
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
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
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_local_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-part shape vertex positions for the current frame, written by
 * `ss_runtime_get_frame_data`. Layout: 12 vertices × `(x, y)` = 24 floats
 * per part, indexed by `part_index * 24`. Vertices are in local space,
 * post-pivot, post-coord-system, so a Player just multiplies them by the
 * world matrix. Only the first
 * `ss_runtime_get_shape_vertex_counts(...)[part_index]` vertices are
 * valid; remaining slots are zero-initialized. Non-shape parts carry zero
 * count and zero-filled vertex slots. When no part of the pack is a shape,
 * the buffer is empty, as is `ss_runtime_get_shape_vertex_box_coords`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_shape_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Per-vertex normalized bounding-box coordinates paired with
 * `ss_runtime_get_shape_vertices`. Layout: 12 vertices × `(fx, fy)` = 24
 * floats per part, indexed by `part_index * 24`. `(0, 0)` corresponds to
 * the LT corner of the shape's pre-pivot bounding box, `(1, 1)` to the RB
 * corner, regardless of pivot offset or coordinate system. Players use
 * this to interpolate the four part-color corners without re-deriving the
 * box layout.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_shape_vertex_box_coords(void *context,
                                            const float **out_data,
                                            uintptr_t *out_len);

/**
 * Per-part valid vertex count for `ss_runtime_get_shape_vertices`. Indexed
 * by `part_index`. Zero for parts that are not shape parts (or shape
 * parts whose size collapsed to zero).
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
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
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_mesh_vertices_x(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Mesh per-vertex Y coordinates. SoA companion to
 * `ss_runtime_get_mesh_vertices_x`; sliced identically via
 * `ss_runtime_get_mesh_vertex_offsets`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_mesh_vertices_y(void *context, const float **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) describing where each
 * part's mesh vertices live inside `ss_runtime_get_mesh_vertices_x`
 * / `_y`. Part `p` occupies indices `[offsets[p], offsets[p + 1])`; non-mesh
 * parts have `offsets[p] == offsets[p + 1]`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_mesh_vertex_offsets(void *context,
                                        const uint32_t **out_data,
                                        uintptr_t *out_len);

/**
 * Mesh per-vertex UV coordinates in cellmap texture-pixel space, interleaved
 * `[u, v, ...]`, concatenated across all parts. Static — resolved once from
 * each mesh part's frame-0 Cell. Sliced per-part by
 * `ss_runtime_get_mesh_vertex_offsets`, identically to the vertex buffers.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_mesh_uvs(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Mesh triangle indices (part-local, 0-based), concatenated across all parts.
 * Static. Slice part `p`'s range via `ss_runtime_get_mesh_index_offsets`:
 * `[offsets[p], offsets[p + 1])`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_mesh_indices(void *context, const int32_t **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
 * `ss_runtime_get_mesh_indices`. Part `p` occupies `[offsets[p], offsets[p + 1])`;
 * non-mesh parts have an empty range.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
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
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_nines_vertices(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Nine-Slice per-vertex UVs in cellmap texture-pixel space, interleaved
 * `[u, v, ...]`. Shares the vertex CSR of
 * `ss_runtime_get_nines_vertex_offsets`. Divide by the texture size to
 * normalize, the same as `ss_runtime_get_local_uvs`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
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
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_nines_box_coords(void *context, const float **out_data, uintptr_t *out_len);

/**
 * Nine-Slice triangle indices, concatenated across all parts and sliced by
 * `ss_runtime_get_nines_index_offsets`. Values are **part-local**
 * (`0..vertex_count`), so a Player passes them alongside the part's vertex
 * slice without rebasing. Each quad is wound `LT, RT, LB / LB, RT, RB`.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_nines_indices(void *context, const int32_t **out_data, uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) over the vertex buffers
 * `ss_runtime_get_nines_vertices` / `_uvs` / `_box_coords`. Part `p` occupies
 * vertices `[offsets[p], offsets[p + 1])` — multiply by 2 to index the
 * interleaved floats. Non-nines parts have an empty range.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_nines_vertex_offsets(void *context,
                                         const uint32_t **out_data,
                                         uintptr_t *out_len);

/**
 * CSR-style prefix sum (length = `num_parts + 1`) for the triangle indices in
 * `ss_runtime_get_nines_indices`. Part `p` occupies
 * `[offsets[p], offsets[p + 1])`; non-nines parts have an empty range.
 *
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers for output.
 * The caller must ensure `out_data` and `out_len` are valid if they are not null.
 */
void ss_runtime_get_nines_index_offsets(void *context,
                                        const uint32_t **out_data,
                                        uintptr_t *out_len);

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

bool ss_runtime_just_looped(void *context);

bool ss_runtime_is_playing(void *context);

bool ss_runtime_is_playing_forward(void *context);

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

bool ss_runtime_get_skip_frames(void *context);

int32_t ss_runtime_get_playback_direction(void *context);

int32_t ss_runtime_get_playback_style(void *context);

int32_t ss_runtime_get_loop_count(void *context);

int32_t ss_runtime_get_remaining_loops(void *context);

int32_t ss_runtime_get_start_frame(void *context);

int32_t ss_runtime_get_end_frame(void *context);

int32_t ss_runtime_get_frame_rate(void *context);

/**
 * The playback speed in effect, as a multiple of `ss_runtime_get_frame_rate`.
 *
 * Reads back what `ss_runtime_set_speed_scale` stored, which is not
 * always what it was handed: a negative rate clamps to `0.0`.
 */
float ss_runtime_get_speed_scale(void *context);

void ss_runtime_set_skip_frames(void *context, bool skip);

void ss_runtime_set_loop_count(void *context, int loop_count);

void ss_runtime_set_frame_rate(void *context, int frame_rate);

void ss_runtime_set_playback_direction(void *context, int direction, int style);

void ss_runtime_set_speed_scale(void *context, float speed_scale);

void ss_runtime_set_animation_section(void *context, int start_frame, int end_frame);

float ss_runtime_update(void *context, float delta);

void ss_runtime_play(void *context);

void ss_runtime_play_with_start_frame(void *context, float frame_no);

/**
 * Holds playback where it stands, keeping the current frame. With animation
 * mixing it holds every track and every fade, as `ss_runtime_stop` stops them.
 *
 * Idempotent: pausing an already-paused animation leaves it paused.
 * `ss_runtime_is_playing` stays true while held — a pause is a hold, not a
 * stop — so tell the two apart with `ss_runtime_is_pausing`. Lift the hold
 * with [`ss_runtime_resume`]; `ss_runtime_play` rewinds and cannot resume.
 */
void ss_runtime_pause(void *context);

/**
 * Lifts a [`ss_runtime_pause`], carrying on from the frame it stopped at.
 *
 * Idempotent, and a no-op on an animation that was never paused. It does not
 * start a stopped animation; use `ss_runtime_play` for that.
 */
void ss_runtime_resume(void *context);

void ss_runtime_stop(void *context);

void ss_runtime_set_frame_no(void *context, float frame);

void ss_runtime_next_frame(void *context);

void ss_runtime_prev_frame(void *context);

/**
 * # Safety
 * `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
 */
bool ss_runtime_set_track_animation(void *context, uint32_t track, const char *name);

bool ss_runtime_set_track_animation_by_hash(void *context, uint32_t track, uint32_t hash);

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

uint32_t ss_runtime_get_track_count(void *context);

bool ss_runtime_clear_track(void *context, uint32_t track);

/**
 * Fade `track` to `name` over `duration_ms` — the same time unit as
 * `ss_runtime_update`'s delta.
 *
 * # Safety
 * `name` must be a valid null-terminated UTF-8 (or lossy-decodable) string.
 */
bool ss_runtime_crossfade(void *context, uint32_t track, const char *name, float duration_ms);

bool ss_runtime_crossfade_by_hash(void *context, uint32_t track, uint32_t hash, float duration_ms);

bool ss_runtime_fade_track(void *context, uint32_t track, float target_alpha, float duration_ms);

/**
 * Progress in `[0, 1]` of the fade on `track`; `NaN` when it is not fading.
 */
float ss_runtime_get_track_fade_progress(void *context, uint32_t track);

bool ss_runtime_cancel_track_fade(void *context, uint32_t track);

bool ss_runtime_set_track_frame(void *context, uint32_t track, float frame);

/**
 * Returns `NaN` when the track has no animation on it.
 */
float ss_runtime_get_track_frame(void *context, uint32_t track);

bool ss_runtime_set_track_speed(void *context, uint32_t track, float speed);

/**
 * What `ss_runtime_set_track_speed` stored (a negative speed reads back as
 * `0.0`); `NaN` for a track that does not exist.
 */
float ss_runtime_get_track_speed(void *context, uint32_t track);

bool ss_runtime_set_track_frame_rate(void *context, uint32_t track, int frame_rate);

/**
 * The rate the track's animation plays at; `0` for a track with no animation
 * and no override, `-1` for a track that does not exist.
 */
int32_t ss_runtime_get_track_frame_rate(void *context, uint32_t track);

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

uint32_t ss_runtime_add_track_source_by_hash(void *context,
                                             uint32_t track,
                                             uint32_t hash,
                                             float weight);

bool ss_runtime_set_source_weight(void *context, uint32_t handle, float weight);

/**
 * Returns `NaN` for an invalid or released handle, so `weight == 0` stays
 * distinguishable from "no such source".
 */
float ss_runtime_get_source_weight(void *context, uint32_t handle);

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

float ss_runtime_get_source_frame(void *context, uint32_t handle);

bool ss_runtime_set_source_event_threshold(void *context, uint32_t handle, float threshold);

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
 * `y_flip = true` mirrors particle Y position and rotation sign, for Y-down
 * rendering of Y-up authored content (Godot, pixi.js). Pass `false` for
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

bool ss_instance_slot_is_independent(void *slot);

/**
 * Set the render mode. `mode` is interpreted as:
 *   0 = Direct       (pre-bake world matrices; default)
 *   1 = SceneGraph   (skip world matrix pre-computation)
 * Unknown values fall back to Direct.
 */
void ss_runtime_set_render_mode(void *context, uint8_t mode);

uint8_t ss_runtime_get_render_mode(void *context);

void ss_runtime_set_coordinate_system(void *context, int32_t coordinate_system);

int32_t ss_runtime_get_coordinate_system(void *context);

void ss_runtime_set_origin(void *context, int32_t origin);

int32_t ss_runtime_get_origin(void *context);

void ss_runtime_set_unit(void *context, int32_t unit);

int32_t ss_runtime_get_unit(void *context);

int32_t ss_runtime_get_passed_event_count(void *context);

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
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer.
 * The caller must ensure the pointer is valid and points to a valid null-terminated string.
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
 * `force`: `0` = clear/revert, `1` = force-hide, `2` = force-visible (reserved).
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
