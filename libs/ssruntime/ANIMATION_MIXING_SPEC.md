# Animation Mixing — Implementation Spec

> Status: **implemented and measured**. SDK-internal record for the Phase 3 ROADMAP item
> **Animation Mixing**. Not Player-facing docs — those are
> `docs/40_components/60_animation_mixing.{md,ja.md}`.
>
> Open work and how to pick it up: **`ANIMATION_MIXING_HANDOFF.md`**.
>
> Scope: how N animation sources resolve into one pose. **Runtime state + FFI only — no
> `.fbs` schema change.** The mix runs before emission, so `SCHEMA.md` is unaffected.
>
> The blend surface was replaced rather than extended: it had shipped to nobody (the repo is
> unreleased and no Player called it), which is a window that closes at the first release.

This document is the record of *why* the model is what it is — the survey it came out of, the
measurements that decided each question, and the things the data corrected. Line numbers are
anchors, not contracts; re-grep before editing.

---

## 0. TL;DR for the implementer

1. **Terminology**: the feature is **Mixing**; the unit of the stack is a **track**. "Layer" is
   taken twice already — the Override Layer feature and the internal `LayerState` (§1).
2. **The model**: tracks fold onto the **setup pose** in ascending order. Within a track,
   sources mix by *weight*; between tracks, by the track's *alpha*, either interpolative or
   additive (§3).
3. **`alpha` is the strength knob; `weight` is not.** A source that is the only one animating
   an attribute owns it outright whatever its weight, because there is nothing to be
   proportional against (§4).
4. **Coverage rule**: an attribute a source does not key is excluded from that attribute's
   weight sum. Presence is per animation — does the keyframe array exist — never per frame
   (§4).
5. **Track 0 is the structure**: discrete attributes and events come from it, `setup_animation`
   establishes it, and it cannot be cleared. There is no separate "primary" (§3.3).
6. **SpriteStudio has a setup pose**, baked by the converter into every animation's keyframe
   arrays and stored as `parts_initial_attributes`. It is the stack's floor and the additive
   reference — but it must be transformed into the instance's coordinate space first (§2.2).
7. **Discrete attributes are not blended.** Resolving them from the topmost covering track
   (§6.3) is specified but deferred: cell switching goes through the Override Layer.

---

## 1. Terminology

| Term | Meaning | Where it is used |
|---|---|---|
| **Mixing** | The N-sources → one-pose composition stage. A per-frame operation with no state of its own. | Feature name, spec name, pipeline prose |
| **Track** | One level of the stack: an ordered slot carrying `alpha`, an operator, and its sources. | The model's unit |
| **Source** / **weight** | One animation on a track, and its share among that track's sources. | `track_add_source`, `set_source_weight` |
| **Crossfade** | A controller on top of mixing that hands a track from one animation to another. | Transitions only |
| **Additive** | A track operator that adds an offset from the setup pose instead of lerping. | Operator, not a feature |

Two naming decisions worth not re-litigating:

- **"Blending" is not a feature name.** The machinery is the *mixer* — as in Godot's
  `AnimationMixer` and Unity's `AnimationMixerPlayable` — and `blend` is what an author asks
  for.
- **"Track", not "layer".** `layer` already means the Override Layer feature
  (`docs/40_components/65_override_layer.md`) and the internal `LayerState` (one source's
  extracted pose);
  three meanings for one word is one too many. *Track* is the generic term across Unity
  Timeline, three.js, Ogre and Spine alike — see §5.5 on why that is not a problem.

---

## 2. The code and the data it runs on

### 2.1 Where the model lives

| Concern | Where |
|---|---|
| `TrackBlend` (Mix / Additive) | `src/lib.rs:809` |
| `TrackState` (alpha, blend, main source, fade) | `src/lib.rs:821` |
| `TrackFade` (crossfade / alpha fade) | `src/lib.rs:847` |
| `SourceState` (animation_idx, controller, **track**, weight, event_threshold) | `src/lib.rs:767` |
| `LayerState` — one source's extracted pose, plus its coverage | `src/lib.rs:603` |
| `layer_coverage` — the coverage lookup, and its "no data means everything" rule | `src/lib.rs:964` |
| **`mix_tracks`** — the whole two-level fold | `src/lib.rs:1061` |
| `rebuild_setup_pose` — the floor, in instance space | `src/lib.rs:1906` |
| `advance_track_fades` | `src/lib.rs:2812` |
| Emission seam | `Context::get_frame_data`, `src/lib.rs:2017` |
| Per-animation coverage, computed once at load | `src/core/framedata.rs:339`, built at `:402` |
| The setup pose, transformed like `evaluate_part` does | `src/core/framedata.rs:597` |

### 2.2 The two data-model facts that drive this spec
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
fourth.

**Re-measured through the shipped track API**, asking for the same 0.3 both ways:

| bases under the overlay | overlay on the **same track** (weight) | overlay on **its own track** (alpha) |
|---|---|---|
| walk | 0.231 | **0.300** |
| walk + run | 0.130 | **0.300** |
| walk + run + wait1 | 0.091 | **0.300** |
| walk + run + wait1 + dash | 0.070 | **0.300** |

Both columns are correct: weight *is* a proportion inside one track, so it dilutes by
construction; alpha is measured against the pose below and does not. This is the whole reason
both knobs exist, and why the docs lead with which one to reach for.

*Harness note*: recover the strength per (part, attribute) and take the median, not one
least-squares fit over everything. Where the overlay animates nothing, `mixed == base`, and a
global fit reads those as "fully diluted" when they only mean "no opinion here" — that
artifact alone reported 0.213 instead of 0.300 on the last row.

### 2.5 Geometric pass (harness, 2026-08-19)

§2.4 measures the mix one attribute at a time. This one measures what comes out the far end:
world matrices, transformed quad corners and skinned mesh vertices — post-mix, post-hierarchy,
post-skinning, the same buffers the Player draws from. It exists because the checks in the
handoff are all statements about geometry ("collapses to the origin", "shrinks", "a dip
mid-transition"), and geometry can be measured instead of squinted at. Harness:
`scratchpad/mixgeom` (throwaway, path-dep on `ssruntime` with `features = ["std"]` to match the
Player's own build). Subject: `Ringo`, 24 animations, **552 ordered pairs, swept exhaustively**.

| Check | Result |
|---|---|
| `alpha = 0` on track 1 reproduces the base **exactly** | 0.000000px over 552 pairs |
| Additive at `alpha = 0` reproduces the base **exactly** (no jump) | 0.000000px |
| Additive is distinguishable from mix at `alpha = 1` | 337–343px apart |
| Any part shrinking below **both** endpoints mid-sweep | worst **0.023%** — nothing |
| Fading track 0 to 0 lands on the rest pose | spread 157→162 (103%), no collapse, no explosion |
| Crossfade dip, `wait1`→`attack1`, `dead`→`run`, `walk`→`attack1`, `wait1`→`dead` | none; no part vanishes; finite throughout |
| Frame scrubbing continuity | max step 4.44px vs mean 2.40px per 0.25 frame |

**Two things it found that are not faults of the model**, both now §7 items 7 and 8: the
long-way rotation (264/552 pairs, worst 1.16× the character's spread) and sign-crossing scale
(85/552 pairs pass a mirrored part through zero). Both are visible, and both look like the
dipping D2 exists to prevent — which is the main reason to have written them down.

**One thing it found that *was* a fault**, fixed in this branch: `update()` returned the frame
of whichever source was primary *before* the fades were advanced. On the tick a crossfade
completes the track changes hands, so the caller got the outgoing animation's frame while
`get_frame_data` validated it against the incoming one — `InvalidParameter` on
`wait1`(80 frames)→`attack1`(20) at frame 45. This is the documented
`get_frame_data(update(dt))` pairing failing on an ordinary transition, and it reached the C
API too (`ss_runtime_update` returns the same value). `ssplayer-wgpu` happened to be immune
because it reads `get_frame_no()` instead. Regression test:
`update_returns_a_frame_get_frame_data_accepts_across_a_crossfade`.

**Traps in this harness, same spirit as §2.4's.** Both cost a wrong answer before they were
caught:

- **Whole-pose averages hide everything.** A dip is a handful of parts pulled toward a default;
  60 well-behaved parts drown them out. The first version of this harness measured the point
  cloud's RMS spread and passed happily **with the coverage rule deliberately disabled**. Only
  a *per-part* series — each part's own world-basis lengths, against its own endpoints — has
  the resolution to see it.
- **Do not measure a part's radius from the pose centroid.** The centroid moves, so a joint
  that merely passes near the middle of the character reads as a 97% collapse. Compare a part
  against *its own* endpoints (distance from the segment joining them), never against a
  moving reference.

---

## 3. D1 — The track model (**implemented**)

> **Naming**: the stack's unit is a **track**, not a "layer". `layer` is already taken twice —
> the Override Layer feature (`docs/40_components/65_override_layer.md`) and the internal `LayerState` (one
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
>
> A second correction, found by the geometric pass (§2.5): **`update()` must read the frame
> back after `advance_track_fades`, not before.** A completing crossfade hands the track to its
> incoming source and so changes which source is primary; sampling the frame first returned the
> outgoing animation's, which `get_frame_data` then rejected whenever the incoming animation was
> the shorter of the two.


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
`clear_blend_sources`, `blend_source_count`, `set_primary`, `primary_handle`, and
`set_source_speed` (superseded by `set_track_speed`).

**`set_source_frame` stays**, across Rust, C and WASM. `set_track_frame` drives only the
track's *main* source, so a track carrying several weighted sources — the locomotion case
`track_add_source` exists for — has no other way to seek the others independently.

Three things fall out of the shape rather than needing to be designed:

- **`crossfade` stops being stateful for the host.** `crossfade(track, name, ms)` starts from
  whatever the track is already playing, so there are no `from` / `to` handles to mix up and
  no abandoned-source weights to clean up — the open question in §7 item 3 disappears.
- **The empty source is unnecessary.** "Fade out to the setup pose" is `fade_track(t, 0, ms)`,
  because the stack's floor *is* the setup pose. Its whole cluster of special cases goes away.
- **"Primary" is unnecessary.** Track 0 is the structure source for discrete attributes and
  events, by construction.

---

## 4. D2 — The coverage rule (**implemented**, always on)

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

Precompute a per-(animation × part) coverage bitmask at `setup_animation` / `track_add_source`
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

### 5.5 "Isn't this Spine?"

Worth answering once, because the question will come from outside too, and SpriteStudio and
Spine sell into the same market.

**The architecture is the field's, not Spine's.** Ordered levels with a per-level weight are
Unity's Animation Layers, Unreal, Godot's AnimationTree and Rive's layers; "track" is Unity
Timeline, three.js `KeyframeTrack` and Ogre `AnimationTrack`; additive blending is in every
one of the eight runtimes surveyed. Nothing in §3 is a thing only Spine does — solving this
problem carefully lands everyone in the same place.

**What is distinctly Spine, we did not take.** The `mixingFrom` chain of interrupted mixes,
`holdPrevious`, the `mixAttachmentThreshold` / `mixDrawOrderThreshold` / `eventThreshold`
family as named knobs, `MixBlend`'s four values with the `timelineMode` / `HOLD` / `HOLD_MIX`
machinery, and the `addAnimation` queue. Our dip avoidance is coverage at (part × attribute)
granularity, derived from a data model Spine does not have — a setup pose the converter bakes
into every animation.

**Do not adopt Spine's vocabulary.** `TrackEntry`, `mixDuration`, `mixTime`, `holdPrevious`,
the `*Threshold` names, `MixBlend`, `setEmptyAnimation`: those names are where an
implementation reads as a port. Generic terms — track, alpha, additive, crossfade, weight —
are the field's and are fine.

The strongest answer is this document: a survey of eight runtimes at their primary sources,
then measurements on 338 of our own animations that found the problems and sized them. That is
a derivation, not a copy.

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

Additive layers are **outside normalization** by construction. The shipped fold is
`cur[k] + (v - base[k]) * alpha` against `Mix`'s `cur[k] + (v - cur[k]) * alpha`
(`src/lib.rs:1214`), applied uniformly — `scale` is additive on `s − s_setup` (which is the
proposed `s − 1` wherever the setup pose is unscaled) and `alpha` participates on the same
terms. The open question in §7 item 2 was resolved this way by the implementation; it is
recorded here rather than left hanging.

**Why the operator earns its place — measured 2026-08-19, and keep this: it is the only
evidence additive has.** The case for additive is that an overlay *held at a value* should
displace an ongoing performance rather than replace it. Held `attack1` on track 1 over a
playing `walk` at `alpha = 0.5`, measuring how much of `walk`'s own motion survives on the 63
parts it actually moves (per-part stddev of world position over 40 frames, against `walk`
alone):

| | motion surviving |
|---|---|
| `walk` alone | 100.0% (reference) |
| **Mix**, alpha 0.5 | **49.6%** |
| **Additive**, alpha 0.5 | **106.0%** |

Mix damps the performance underneath by exactly its alpha — that is what a lerp toward a static
pose *is*, and no parameter choice avoids it. The worst-hit part goes 9.87px → 2.95px. Additive
leaves it at 9.79px; the 6% overshoot is a parent's offset widening its children's swing
through the hierarchy, not an error. **This behaviour is not expressible with `Mix` at any
alpha**, which is the whole argument for having a second operator.

**What this does *not* show.** No asset in the repo demonstrates additive being *useful*.
Ringo's overlays are whole-body performances, so adding one displaces the rig by 342.8px
(against mix's 11.0px) — mechanically correct, visually nonsense. The handoff's visual check 3
therefore confirms the mechanism and nothing more. Judging whether additive is worth authoring
for needs an axis-shaped animation — a small delta from the setup pose — which is what
`param_head_yaw` is for. **Decision (2026-08-19): additive stays.** Kept on the strength of the
measurement above plus standard practice (Unity additive layers, Spine, three.js
`makeClipAdditive`, Unreal), with its product value still unproven until that axis exists.

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
- **Later, with §6.3**: the two the Player docs still list under §9 Limitations —
  "discrete attributes come from track 0, so an overlay track cannot swap a cell" and
  "`parts_color` follows track 0".

---

## 7. Open decisions

1. **§6.3 threshold semantics** — one per-layer threshold, or Spine's split into
   attachment/draw-order/alpha thresholds? *Lean: one, until a case splits it.*
2. ~~**§6.2 additive and `scale`/`alpha`** — delta-on-value for everything, or per-attribute
   rules?~~ **Closed**: uniform delta against the setup pose, no per-attribute rules
   (`src/lib.rs:1214`). Documented in §6.2, along with the measurement that kept the operator.
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
   180°-crossing mix takes the long way. Cheap to add, needs a default. **Measured (§2.5):
   this is not a corner case.** On `Ringo`, **264 of 552** animation pairs swing some part
   **≥0.5× the character's own spread** off the straight path between its endpoints; the worst,
   `slipdown_wait`+`dance` part 17, travels **1.16×** because the part sits at **+131.08° in one
   source and −141.98° in the other** — an 87° gap that the raw-degree mix walks the other way
   round, 273°. Any mid-alpha hold on such a pair is visibly wrong, so this is the open item
   with the most evidence behind it.
8. **Sign-crossing scale** — a part one source mirrors and the other does not (`scale_y` +1 vs
   −1) is linearly mixed straight through **zero**: at `alpha = 0.5` the part is squashed to
   nothing, then reappears mirrored. **85 of 552** `Ringo` pairs mirror at least one part —
   every pair involving `dead` does. Same family as item 7, and the reason §2.5's dip check
   excludes mirrored parts: **it looks exactly like the dipping the coverage rule exists to
   prevent, and it is not that.** *Lean: leave it. A mirrored part is a different pose, not a
   pose with an in-between, and the alternatives (mixing the sign discretely, or on a
   threshold) all invent a rule the author did not ask for. Document it and, if it bites,
   reach for §6.3's topmost-covering-track resolution rather than a special case here.*
9. **Per-part masks** — deferred per §5.4. When added, they belong on the layer, alongside
   alpha and the operator.

---

## 8. File reference index

Duplicates §2.1 for the parts that live outside `mix_tracks`; §2.1 is the one to trust for the
model itself.

| Concern | Path |
|---|---|
| Pose evaluation + hardcoded defaults | `src/core/framedata.rs:530` (defaults at `:547`) |
| The setup pose, transformed to match it | `src/core/framedata.rs:597` — **keep the two in step** |
| Per-animation coverage | `src/core/framedata.rs:339`, computed at `:402` |
| Keyframe presence (`Option`) | `src/core/interpolation.rs:230` |
| Setup pose in the format | `parts_initial_attributes`, written by `ssconverter/src/model_t/animedata.rs:285` and merged into every animation at `:1384-1392` |
| Player-facing docs | `docs/40_components/60_animation_mixing.{md,ja.md}` |
| Adjacent feature | The Override Layer, the post-mix overwrite pass: `src/core/override_layer.rs`, documented in `docs/40_components/65_override_layer.md` |
| Open work, until both PRs merge | `ANIMATION_MIXING_HANDOFF.md` (transient) |
