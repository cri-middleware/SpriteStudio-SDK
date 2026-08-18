# Animation Mixing — Implementation Spec (cold-start)

> Status: **partially implemented / redesign in progress**. SDK-internal implementation spec
> for the Phase 3 ROADMAP items **"Animation Mixing/Blending"** (shipped) and **"Per-part
> animation blending"** (this redesign supersedes it). Not Player-facing docs (do not move
> under `libs/ssruntime/docs/`, which is curated for Players).
>
> Scope: how N animation sources resolve into one pose. **Runtime state + FFI only — no
> `.fbs` schema change.** The mix runs before emission, so `SCHEMA.md` is unaffected.
>
> **The blend C-API is used by no Player** (confirmed 2026-07-17). Additions, signature
> changes and removals are all free until the first release that ships them. Do not grow
> `_ex` variants — redesign the existing signatures in place.

Grounded in the current code (`develop`, HEAD ~`2c1e5c0`). Line numbers are anchors, not
contracts — re-grep before editing.

---

## 0. TL;DR for the implementer

1. **Terminology**: the feature is **Mixing**. `blend` survives only as API vocabulary
   (`blend source`, `weight`). "Blending" is not a separate feature name (§1).
2. **Model (D1)**: **layered**. Within a layer, sources are combined by *normalized weighted
   sum*; between layers, an *ordered overwrite* `lerp(below, layer, alpha)`. Layer 0 with a
   single implicit layer reproduces today's behavior exactly (§3).
3. **Coverage (D2)**: an attribute a source does **not key** is excluded from that
   attribute's weight sum. Presence is decided **per animation** (does the keyframe array
   exist), never per frame (§4).
4. A layer carries **alpha + operator (interpolate | additive) + (later) a part mask**.
   Additive is in scope as the enabler for **SS-Studio Tier 3** modulation axes, and its
   reference is the **setup pose already carried in every `.ssab`** (§6.2) — no new parameter.
5. **Discrete attributes stay "primary wins" for now.** Resolving them from the topmost
   covering layer (§6.3) is specified but **deferred** — the driving product routes mouth /
   blink / expression through the already-shipped **Override Layer**, not through the mix.
6. **SpriteStudio has a setup pose**, and the converter bakes it into every animation's
   keyframe arrays; only attributes keyed by *neither* the animation nor the setup pose fall
   back to hardcoded defaults (§2.2). D2 acts on that genuinely-absent case, which real assets
   hit on **20–37% of (part × attribute) pairs** (§2.3, measured over 338 animations).

---

## 1. Terminology

| Term | Meaning | Where it is used |
|---|---|---|
| **Mixing** | The N-sources → one-pose composition stage. A pure per-frame operation with no state of its own. | Feature name, spec name, pipeline prose |
| **Blend source** / **weight** | An input to the mix and its share. | API vocabulary (`add_blend_source`, `set_source_weight`) — kept as-is |
| **Layer** | An ordered group of sources carrying `alpha`, an operator, and (later) a part mask. | The new level introduced by D1 |
| **Crossfade** | A controller *on top of* mixing that drives weights over time. | Transitions only |
| **Additive** | A mix operator that adds a delta instead of participating in normalization. | Operator, not a feature |

"Blending" is **not** used as an independent feature name. This matches Godot
(`AnimationMixer`) and Unity (`AnimationMixerPlayable`), where the machinery is the *mixer*
and `blend` is what the author asks for. Rename the Player-facing doc accordingly
(`docs/40_components/60_animation_blending.md` → Animation Mixing).

---

## 2. What the code does today

### 2.1 Current shape

| Concern | Where |
|---|---|
| `SourceState` (animation_idx, controller, weight, event_threshold) | `src/lib.rs:758` |
| `CrossfadeState` (from, to, elapsed, duration) | `src/lib.rs:792` |
| `LayerState` (the extracted pose of one source) | `src/lib.rs:597` |
| `mix_poses` — the whole composition stage | `src/lib.rs:880` |
| Per-part weight renormalization | `src/lib.rs:926-940` |
| `update_flag` OR-merge / write-back | `src/lib.rs:1012`, `src/lib.rs:1051` |
| Emission seam | `Context::get_frame_data`, `src/lib.rs:1700` |

Sources are a **flat array with one `primary_idx`**. Continuous attributes are a normalized
weighted sum over active sources; every discrete attribute is **copied verbatim from the
primary**; `update_flag` is OR-composed.

### 2.2 The two data-model facts that drive this spec

**(a) Poses are dense in value — and SpriteStudio *does* have a setup pose.** Each `.ssae`
carries an `isSetup` animation (one frame, the rest pose). The converter never emits it as a
playable animation; instead it (i) stores it as `parts_initial_attributes` on **every**
`AnimationData` (`ssconverter/src/model_t/animedata.rs:285`) and (ii) **seeds every
animation's keyframe arrays with it before the animation's own keys**
(`animedata.rs:1384-1392`). So an attribute the animator never touched still has a real
frame-0 key **if the setup pose keys it**.

Only where *neither* the animation nor the setup pose keys an attribute does `evaluate_part`
(`src/core/framedata.rs:530`, defaults at `:547`) fall back to a hardcoded default —
`position=0`, `scale=1`, `alpha=1`, `hide=true`.

Two consequences: the merge is **lossy** (at runtime "authored by this animation" and
"inherited from the setup pose" are indistinguishable — see §4), and `parts_initial_attributes`
is **present in every `.ssab` and currently unread by the runtime**, which is exactly the
reference pose additive blending needs (§6.2).

**(b) But presence is known.** `evaluate_float` (`src/core/interpolation.rs:230`) returns
`Option`, and `update_flag` records, **per part × per attribute**, whether a key existed.
Today this information reaches the output but is *not consulted by the mix*.

Consequence: without D2, a source that does not animate a part still votes for the hardcoded
default, dragging the mixed result toward `position=0` / `alpha=1`. That is the SS-shaped
version of Spine's "dipping", and (b) is exactly the signal needed to fix it.

### 2.3 Authoring density — measured, do not re-derive from intuition

**(i) How often does coverage actually differ between animations?** This is the population D2
acts on. Measured over **157 `.ssae` / 338 animations** (the official SpriteStudio sample
projects plus this repo's fixtures; every one of them has a setup animation). "PARTIAL" = the
(part × attribute) pair exists in the `.ssab` for some animations of that file but not others:

| Attribute | part×attr pairs | in every anim | **PARTIAL** |
|---|---|---|---|
| SCLX / SCLY | ~1900 | 67% | **33%** |
| POSX / POSY / ROTZ | ~1975 | 70% | **30%** |
| CELL | 1125 | 70% | **30%** |
| ALPH | 1896 | 77% | **23%** |
| PRIO | 1934 | 79% | **21%** |
| HIDE | 1990 | 86% | **14%** |
| PCOL | 299 | 24% | **76%** |

**D2 fires on 20–37% of pairs on real assets** — it is not a rounding error, and it is not a
no-op either.

**(ii) How much of that presence is inherited from the setup pose rather than authored?**
4.3% (HIDE) → 16.8% (CELL); 8–11% for the TRSA attributes. So the runtime's inability to tell
the two apart (§2.2) affects a minority of pairs — and see §4 for why it is benign anyway.

**(iii) Per-attribute density within one asset** (`tests/Ringo/Ringo.ssae`, 1646 part×animation
entries) — the number that decides whether coverage can stand in for a *mask*:

| Attribute | Keyed |
|---|---|
| HIDE | 90% |
| ROTZ | 87% |
| POSX / POSY | 81% |
| PRIO | 45% |
| CELL | 39% |
| SCLX / SCLY | 35% / 31% |
| ALPH | 13% |

Read this as: **coverage alone cannot substitute for an explicit mask on existing game
assets** — SS authors key position/rotation on nearly every part. It *is* decisive for
alpha/scale/cell, and it is decisive across the board for **newly authored Vtuber rigs**,
where a "key only what this layer touches" convention can be imposed (§5.4).

### 2.4 Measured effect of D1 and D2 (harness, 2026-08-18)

Both decisions were re-checked against real assets before implementing anything: **20 sspj
projects → 154 `.ssab`** (the official SpriteStudio sample set in `~/Desktop/ss` plus this
repo's fixtures), driven headless through the real `Context` / `get_frame_data` path. Harness:
`scratchpad/mixprobe` (throwaway crate, path-deps on `ssruntime` + `ssconverter`; re-run it
after each implementation step).

**D2 — how far can a non-covering source drag the result?** The mix is linear, so this is
exact rather than sampled: for every (part, attribute) keyed by some animations of an asset
and not others, `drag = weight × |value − default|`.

| | |
|---|---|
| (part × attr) keyed by at least one animation | 11,656 |
| coverage differs between animations | 2,796 (**24.0%**) |
| ...and the covering animation actually leaves the default → **real drag possible** | 1,758 (**15.1%**) |
| position drags > 50px / 5–50px / ≤5px | 471 / 170 / 44 |
| assets with any drag | **17 of 154** |

So D2 is not cosmetic — but the damage is **concentrated**, not uniform: `fire_sword.ssab`
(958 hits, worst 2,519px) and `00_spr_03_action_01_level3.ssab` (496 hits, worst 3,169px)
dominate, while most character rigs are clean. `Ringo.ssab` has 41 hits, worst 534px.

**Do not sample pairs by hand.** Four hand-picked Ringo pairs (`wait1+attack1`, `walk+run`,
`walk+attack1`, `wait1+dead`) showed **zero** drag — every coverage-differing attribute there
is one the animator keyed at its neutral value (`alpha=1`, `scale=1`), which is exactly what
the hardcoded default already supplies. The exhaustive sweep is what finds the 41.

**After implementing D2**, the harness re-runs the same worst cases as *real blends* — the
covering animation mixed 50/50 with one that does not cover the attribute — and measures how
far the value moves off the covering animation's own: **0.000000 across 220 sampled hits
spanning every affected asset** (was up to 1,584px at 50/50). Control: blending against a
partner that *does* cover the attribute still moves it (588.9px), so the check is live rather
than trivially zero.

**Cost.** D2 adds ~**22%** to the mix stage (65 parts, 2 sources: 9.9µs → 12.1µs per
`get_frame_data`). Single-animation playback never reaches `mix_poses` — `get_frame_data`
swaps the layer instead — and measured **unchanged** at 13.8µs, so nothing that ships today
pays for this. Two attempts to win the 22% back both measured no better and were dropped; see
the comment in `mix_poses` before trying a third.

**Floor — can the setup pose sit under track 0?** The track model puts the setup pose
(`parts_initial_attributes`, §2.2) under the stack instead of `evaluate_part`'s hardcoded
constants. Checked across every asset: of **217,350** (animation × part × attribute) entries,
**159,576 are unkeyed** — the only ones where the floor is observable at all — and on **every
one of them the setup pose already equals the hardcoded default**. Swapping the floor is a
**no-op for single-animation playback**, which is what makes the track model's base pose free
to adopt. (Where the setup pose carries a real value it is also baked into the animation's own
keyframe array, so the animation covers it and the floor never shows.)

**D1 — how much of an overlay survives?** Overlay asked for weight 0.3, measured back out of
the mixed pose (`Ringo`, `attack1` over locomotion):

| bases under the overlay | asked | **measured** |
|---|---|---|
| walk | 0.300 | **0.231** |
| walk + run | 0.300 | **0.130** |
| walk + run + wait1 | 0.300 | **0.091** |
| walk + run + wait1 + dash | 0.300 | **0.070** |

The overlay loses **more than half its strength** by the second base source and 77% by the
fourth. D1 holds it at 0.300 on every row. Confirmed as predicted (§3.1).

---

## 3. D1 — Track model (evaluation **implemented**, API pending)

> **Naming**: the stack's unit is a **track**, not a "layer". `layer` is already taken twice —
> the Override Layer feature (`OVERRIDE_LAYER_SPEC.md`) and the internal `LayerState` (one
> source's extracted pose) — and three meanings for one word is one too many. "Track" is the
> generic term across Unity Timeline, three.js, Ogre and Spine alike.
>
> Shipped 2026-08-18: `TrackState` (alpha + blend op), `SourceState::track`, and `mix_tracks`
> — the two-level fold with the setup pose as its floor. Tests:
> `track_alpha_lerps_against_the_track_below`,
> `track_alpha_is_independent_of_the_sources_below`,
> `additive_track_offsets_by_its_distance_from_setup`,
> `track_passes_through_attributes_it_does_not_animate`.
> **Public API shipped the same day** (§3.3): `set_track_animation` / `set_track_alpha` /
> `set_track_blend` / `crossfade(track, name, ms)` / `fade_track` / `clear_track` /
> `set_track_frame` / `set_track_speed` / `track_add_source`, across Rust, C and WASM, with the
> old blend surface removed. Player-facing docs rewritten as
> `docs/40_components/60_animation_mixing.{md,ja.md}`.
>
> One correction the implementation forced: the floor has to be the setup pose **transformed
> into the instance's coordinate space** (`core::framedata::setup_pose_part`, kept in step with
> `evaluate_part`), not the raw `parts_initial_attributes`. Fading track 0 to alpha 0 against an
> untransformed floor drifts toward a mis-placed rest pose. It is rebuilt on `setup_animation`
> and whenever a coordinate setting changes — never per frame.


### 3.1 Why not flat normalization

For **two** sources the normalized sum and an ordered `lerp` are algebraically identical
(`(w₁A + w₂B)/(w₁+w₂) == lerp(A, B, w₂/(w₁+w₂))`), so **crossfade is unaffected by this
decision**. They diverge at three or more sources:

```
walk 0.5, run 0.5, flinch 0.3
  flat normalized  → walk .385 / run .385 / flinch .231   (overlay weakens as the base grows)
  layered          → .7·(.5walk + .5run) + .3·flinch      (overlay is exactly .3)
```

An overlay whose strength depends on how many sources happen to sit under it is not
expressible as an intent. Every surveyed runtime solves this the same way — Rive
(`finalMix = stateMachineMix × animationMix`), Unity (layer × blend tree), DragonBones
(`layer` weight budget) — with two levels.

### 3.2 The model

```
resolved = ∅
for each layer L in ascending order:
    pose_L = Σ_{s ∈ L, s covers this attribute} w_s · pose_s  /  Σ_{s ∈ L, s covers} w_s
    if no source in L covers this attribute: skip L entirely for this attribute
    resolved = mix_op(L)(resolved, pose_L, alpha_L)
```

- **Within a layer**: normalized weighted sum. Order-independent; the existing per-part
  renormalization (`src/lib.rs:926-940`) applies unchanged.
- **Between layers**: ordered. `interpolate` → `lerp(resolved, pose_L, alpha_L)`;
  `additive` → `resolved + alpha_L · (pose_L − reference_L)` (§6.2).
- Layer 0 is the base. With one layer and `alpha = 1` this is bit-for-bit today's behavior,
  which is what makes the migration safe.
- Normalization is **per attribute × per part**, not global (D2 + the existing per-part rule).

### 3.3 API shape — replace the blend surface, do not extend it

The blend C-API has shipped to nobody (the repo is still unreleased, and no Player calls it),
so the right move is to **redesign it around the track**, not to bolt a track index onto the
existing handle-based calls. The used API — `setup_animation`, `update`, `get_frame_data`,
`set_frame_no`, overrides, events — is untouched; `setup_animation(name)` simply means "put
this on track 0".

```c
/* the whole common case */
bool ss_runtime_set_track_animation(ctx, track, name);
bool ss_runtime_set_track_alpha(ctx, track, alpha);        /* strength */
bool ss_runtime_set_track_blend(ctx, track, MIX | ADDITIVE);
bool ss_runtime_crossfade(ctx, track, name, ms);           /* fade the track TO an animation */
bool ss_runtime_fade_track(ctx, track, target_alpha, ms);
bool ss_runtime_clear_track(ctx, track);
bool ss_runtime_set_track_frame(ctx, track, frame);        /* modulation scrubbing */
bool ss_runtime_set_track_speed(ctx, track, speed);

/* opt-in: several weighted sources on one track (locomotion blending) */
uint32_t ss_runtime_track_add_source(ctx, track, name, weight);
bool     ss_runtime_set_source_weight(ctx, handle, weight);
```

**Removed**: `add_blend_source` / `_by_hash`, `add_empty_blend_source`, `remove_blend_source`,
`clear_blend_sources`, `blend_source_count`, `set_primary`, `primary_handle`, and the
`set_source_frame` / `set_source_speed` pair (superseded by the track forms).

Three things fall out of the shape rather than needing to be designed:

- **`crossfade` stops being stateful for the host.** `crossfade(track, name, ms)` starts from
  whatever the track is already playing, so there are no `from` / `to` handles to mix up and
  no abandoned-source weights to clean up — the open question in §7 item 3 disappears.
- **The empty source is unnecessary.** "Fade out to the setup pose" is `fade_track(t, 0, ms)`,
  because the stack's floor *is* the setup pose. Its whole cluster of special cases goes away.
- **"Primary" is unnecessary.** Track 0 is the structure source for discrete attributes and
  events, by construction.

---

## 4. D2 — Coverage rule (**implemented**, default ON)

> Shipped 2026-08-18. `CachedPartData::coverage` (`core/framedata.rs`) is computed once per
> animation × part at load; `LayerState::coverage_vec` carries it to the mix; `mix_poses`
> normalizes per attribute. Empty sources cover everything. Tests:
> `mix_poses_excludes_sources_that_do_not_key_the_attribute`,
> `mix_poses_uncovered_attribute_keeps_primary_value`,
> `mix_poses_coverage_normalizes_among_covering_sources_only`,
> `empty_source_covers_every_attribute`. Verified on real assets — see §2.4.


An attribute that a source does not key does **not** participate in that attribute's weighted
sum. If no active source in the layer keys it, the layer is skipped for that attribute; if no
layer keys it, the hardcoded default stands.

**Presence is decided per animation, not per frame.** Use "does the keyframe array exist for
this part × attribute" (`CachedPartData::*_array.is_some()`), *not* the per-frame
`update_flag` (which is "a key exists at or before this frame"). The per-frame signal flips
at the first key and would make a part pop in and out of the mix mid-playback. Spine makes
the same choice — its `timelineMode` is computed per track entry, not per frame.

`update_flag` keeps its current output role (OR-merge, `src/lib.rs:1012`) unchanged.

**Consequence to know: weight only means something among the sources that cover an
attribute.** When exactly one active source animates an attribute, normalization gives it 1.0
and its `weight` has no effect there — a 0.05-weight overlay still owns that attribute
outright. This is the correct reading of "nothing else has an opinion" (the alternative is
blending toward a hardcoded constant, which is what D2 removes), and it matches the per-part
rule that already shipped, but it does mean **`weight` is not a general strength knob**. The
knob that always means something is a layer's `alpha` (D1, §3.2), which lerps against the
pose below rather than voting inside a normalized sum.

Measured on `Ringo` (`wait1` + `walk` at various weights): for an attribute both animate,
weight behaves exactly as the normalized share predicts (0.05 → −8.57°, 0.3 → −41.54°,
0.9 → −85.26°, 3.0 → −135.0° against a −180° source). Ringo has no overlay-only attribute
whose value differs from the default, which is why its confinement numbers barely move — the
assets in §2.4 that do are where this shows.

**Setup-inherited keys count as coverage, and that is correct.** Because the converter seeds
every animation with the setup pose (§2.2), "this attribute has a keyframe array" means
"authored by the animator **or** by the setup pose", and the runtime cannot separate the two.
That is the behavior we want: a setup-pose value is a real rest pose, so letting it into the
weighted sum is meaningful (it is what Spine's `MixBlend.setup` achieves deliberately). D2
excludes only the genuinely absent case, where the alternative is a hardcoded constant that
means nothing. **No schema change is needed** — resist adding an "authored by this animation"
mask to the `.fbs` unless a measurement shows the distinction matters.

Precompute a per-(animation × part) coverage bitmask at `setup_animation` / `add_blend_source`
time so the mix loop reads a `u64` rather than re-deriving presence per frame.

---

## 5. Driving use case — SS-Studio (VTuber), as actually specified

Source of truth: the SS-Studio product spec (Evernote note **"Vtuber 検討"**, attachment
`vtuber_and_effect_tools_spec.md`, §4–§5). **Read it before assuming what this feature is
for** — its architecture is deliberately *not* the Live2D-shaped "many continuous layers
always live" model, and assuming otherwise leads to the wrong scope.

### 5.1 What that product actually does

Its drive model is inverted: **authored animation is primary, tracking is secondary**
(§4-2 there). Tracking *selects performances*; it does not drive the avatar.

| Concern | How SS-Studio solves it | What it asks of mixing |
|---|---|---|
| Performances (`idle`, `talk_loop_*`, `act_nod`, …) | one at a time, entered/left by **crossfade** | already shipped |
| Continuous modulation (head yaw/pitch) | **frame scrubbing** — the axis is authored as an animation and `set_frame_no(30.0 + yaw*30.0)` scrubs it (§4-5) | nothing |
| Lip sync / blink / expressions | **cell override** via the Override Layer (§4-6) | nothing |
| Secondary motion (hair, accessories) | spring-damper in the Player, or one modulation axis | nothing |
| Interruption / priority between performances | crossfade in v1; the ROADMAP State machine item later | nothing new |
| Stacking several modulation axes on top of a performance | **Additive blend** — and only here | **§6.2** |

Its own tier table puts the SDK dependency at **zero up to Tier 2, which is the product's
centre**; only **Tier 3 (multi-axis continuous follow)** needs additive blending. The spec
states the constraint plainly: *`ssruntime` gets nothing beyond what the ROADMAP already
lists.*

### 5.2 What this changes for this spec

1. **Do not scope for "5–6 always-live layers".** Two or three concurrent sources is the
   realistic ceiling for SS-Studio: a performance, a crossfade partner, and one or two
   modulation axes.
2. **Additive stays in scope, for a different reason** — Tier 3 modulation axes, not
   expressions. Its reference is the axis's **neutral (centre) frame**, which the product spec
   already fixes as a convention (*"`param_*` の中央フレームを setup pose と一致させる規約は、
   そのまま delta の定義になる"*). Hence the reference must be a **caller-specified frame**
   (§6.2), not frame 0.
3. **Discrete-attribute resolution drops out of the first scope** (§6.3). Mouth, blink and
   expression are cell overrides in that product, and the Override Layer already ships.
4. **Per-part masks stay deferred**, but for a weaker reason than first assumed: SS-Studio's
   modulation axes are few and coverage (§4) already isolates them.
5. **Batch FFI setters are not urgent** (§3.3) — the host is Rust, and there are one or two
   axes, not fifty parameters. Keep the shape, drop the priority.
6. **D1 + D2 retire two limitations the product spec had to design around.** It documents
   both as reasons to avoid multi-axis modulation: *"軸を足すほど全部の効きが弱くなる"* (the
   1/N dilution D1 fixes) and *"キーを持たないパーツには setup pose 値が混ざる"* (the default
   contamination D2 fixes). **Landing D1 + D2 makes Tier 3 substantially cheaper and may
   remove the need for per-part masks there entirely** — that is the strongest argument for
   doing them, and it should be reflected back into the product spec's §4-5 / §5-1.

### 5.3 ROADMAP alignment

Additive (E), per-part weight/mask (F) and the state machine (G) are all existing Phase 3
items. **The layered model (D1) and the coverage rule (D2) are not on the ROADMAP** — they
are a redesign of the shipped whole-animation blend, not a new feature line. Fold them into
the existing "Per-part animation blending" item's description rather than adding entries, and
say explicitly that the single-layer path is unchanged.

### 5.4 Prior art inside the project

An earlier design session (Evernote **"Motion Blending Architecture for ssruntime (Godot)"**)
agreed on a **pose-buffer + node-graph** architecture (`BlendNode` / `MaskNode`, a fast path
that skips the extra pose buffer for single-animation playback). What shipped instead is the
flat source list this spec now layers. **The ordered layer chain of §3 is the linear subset of
that graph** — enough for everything §5.1 asks for, and it leaves the graph reachable later
without another data-model change.

---

## 6. Consequences to specify

### 6.1 Continuous attributes
Unchanged list (TRSA, size, pivot, anchor, UV, bounding_radius, mask, skew,
`raw_vertex_deform`), now summed under D2's per-attribute coverage.

### 6.2 Additive operator
`resolved += alpha_L · (pose_L − reference_L)`.

**Reference pose**: the **setup pose**, read from `parts_initial_attributes` — already present
in every `.ssab` and currently unread by the runtime (§2.2). No new parameter, no schema
change, no per-source reference frame.

This is the right base rather than merely the convenient one: SS-Studio's own authoring
convention for a modulation axis is *"`param_*` の中央フレームを setup pose と一致させる"* —
the neutral frame **is** the setup pose. Reading the setup pose directly means the delta is
correct even when an author's centre frame drifts from it, and it removes the "which frame is
neutral?" question from the API entirely. (three.js needs `makeClipAdditive(clip, frame)`
because glTF has no setup pose to appeal to; SpriteStudio does.)

Additive layers are **outside normalization** by construction. Open: whether `scale` should be
additive on `(s − 1)` (proposed: yes, uniform treatment — additive of the delta) and whether
`alpha` participates at all.

### 6.3 Discrete attributes — specified, deferred
Today: verbatim from the primary (`src/lib.rs:884-903`). **Not in the first scope** — the
driving product does mouth / blink / expression as cell overrides (§5.1), so nothing needs
this yet. Specified here so the later change is not re-derived: **the topmost layer that
(a) keys the attribute for this part and (b) has `alpha ≥ threshold` wins**, with a
per-layer threshold defaulting so that a layer at full alpha always wins and a fading layer
does not. Directly modelled on Spine's `mixAttachmentThreshold` / `mixDrawOrderThreshold`
(both default 0 there).

Two attributes need care:
- **`priority`** — the z-order sort key. Switching it mid-fade re-sorts the draw order; keep
  the threshold conservative or pin it to layer 0.
- **`cell`** — a cell that crosses a texture-atlas boundary moves `DrawBatch` split points;
  the batch-merge condition must be evaluated **after** the mix (same constraint the Override
  Layer spec calls out).

### 6.4 Limitations these decisions retire
- **Now**: "a source that does not animate a part drags the result toward the defaults" — D2.
- **Now**: "an overlay's strength depends on how many sources sit under it" — D1. This is the
  one the SS-Studio spec explicitly designs around (§5.2 item 6).
- **Later, with §6.3**: "only animations with the same cell assignment can be blended"
  (docs §8), and "`parts_color` is taken from the primary".

---

## 7. Open decisions

1. **§6.3 threshold semantics** — one per-layer threshold, or Spine's split into
   attachment/draw-order/alpha thresholds? *Lean: one, until a case splits it.*
2. **§6.2 additive and `scale`/`alpha`** — delta-on-value for everything, or per-attribute
   rules? *Lean: uniform delta; document the scale case.*
3. **Interruption / stale weights** — `crossfade` currently discards a running fade and
   restarts from `(1, 0)`, leaving whatever weights the abandoned sources had. Spine keeps a
   `mixingFrom` chain so three animations can be mixing at once. *Lean: define a cleanup rule
   first, chain only if the Vtuber/state-machine work needs it.*
4. **Event threshold default** — ours is `1.0` (primary only); Spine's `eventThreshold` is `0`
   (no events while mixing out). Decide which way the default points once layers exist,
   since "primary" weakens as a concept under D1.
5. **Per-source playback control** — `play`/`pause`/`stop`/`loop`/`is_finished` are
   primary-only today; secondaries only have frame/speed/weight. Vtuber layers want
   independent loop control.
6. **Reserved transitions** (Spine `addAnimation` + `delay`, Unity exit time) — not needed by
   the Vtuber case; revisit with the state machine item.
7. **Shortest-path rotation** (Spine `shortestRotation`) — we mix raw degrees, so a
   180°-crossing mix takes the long way. Cheap to add, needs a default.
8. **Per-part masks** — deferred per §5.4. When added, they belong on the layer, alongside
   alpha and the operator.

---

## 8. Implementation order (cold-start)

1. ~~D2 coverage bitmask + per-attribute renormalization inside the existing flat mix.~~
   **Done (2026-08-18)** — §4, verified in §2.4.
2. Layer container: layer index on `SourceState`, `alpha`, ordered fold. Single-layer path
   must stay bit-identical to today (regression test on the existing `mix_poses_*` tests,
   `src/lib.rs:4379-4700`).
3. Additive operator + per-source reference frame (§6.2). This is the SS-Studio Tier 3
   enabler; steps 1–2 alone already make multi-axis modulation viable, so re-measure the need
   before building it.
4. FFI + WASM surface, doc rewrite (`docs/40_components/` → Animation Mixing), and the
   ROADMAP wording fix (§5.3).
5. Validate on `ssplayer-wgpu` with a real SS-Studio rig before freezing the C-API.
6. *Deferred*: discrete-attribute resolution by topmost covering layer (§6.3), including the
   post-mix batch-split constraint. Only when something needs cell switching through the mix
   rather than through the Override Layer.

---

## 9. File reference index

| Concern | Path |
|---|---|
| Mix stage | `src/lib.rs:880` (per-part renorm `:926`) |
| Source / crossfade state | `src/lib.rs:758`, `:792` |
| Extracted pose | `src/lib.rs:597` |
| Pose evaluation + defaults | `src/core/framedata.rs:530` (`:547`) |
| Keyframe presence (`Option`) | `src/core/interpolation.rs:230` |
| Emission seam | `src/lib.rs:1700` |
| Player-facing doc to rewrite | `docs/40_components/60_animation_blending.md` (+ `.ja.md`) |
| Adjacent spec | `OVERRIDE_LAYER_SPEC.md` (post-mix overwrite pass) |
