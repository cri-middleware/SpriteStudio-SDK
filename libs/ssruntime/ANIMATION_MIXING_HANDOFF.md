# Animation Mixing — resume here

> **Transient.** Delete this file once both PRs below have merged. The durable record — the
> model, the measurements and the decisions — is `ANIMATION_MIXING_SPEC.md`, which outlives it.

Last updated 2026-08-18.

## Where it stands

The track stack is **implemented and merged into neither repo yet**. Two draft PRs are open,
and one thing blocks taking them out of draft: nobody has looked at it running.

| | |
|---|---|
| **SDK** | [#362](https://github.com/cri-middleware/SpriteStudio-SDK/pull/362) draft → `develop`, branch `feat/animation-mixing-tracks` |
| **wgpu** | [SSPlayerForWgpu#44](https://github.com/cri-middleware/SSPlayerForWgpu/pull/44) draft → `main`, branch `feat/track-api-passthrough` |

| Piece | State |
|---|---|
| Coverage rule (D2) | done, verified on 220 real-asset cases |
| Track stack, additive, crossfade, fades (D1) | done |
| Public API — Rust, C, WASM; old blend surface removed | done |
| Player-facing docs, `20_limitations`, README, ROADMAP, CHANGELOG | done |
| `ssplayer-wgpu` pass-throughs + viewer bindings + seam tests | done |
| **Visual pass on `ssplayer-wgpu`** | **not done — this is what blocks the PRs** |
| `param_head_yaw` axis, and judging the scrub with it | not done, needs authoring |
| Vtuber product spec §4-5 / §5-1 rewrite | not done |
| Per-part masks | deferred by design (`SPEC` §8 item 7) |

## Getting back to a working tree

```sh
# SDK
git switch feat/animation-mixing-tracks
cargo test --workspace --all-features        # 242 green

# wgpu — its submodule must point at the SDK branch
cd ../SSPlayerForWgpu
git switch feat/track-api-passthrough
git -C SpriteStudio-SDK fetch origin feat/animation-mixing-tracks
git -C SpriteStudio-SDK checkout FETCH_HEAD
cargo test --workspace
```

**Before merging wgpu#44**: its commit moves the submodule pointer onto the SDK's *branch*.
Once SDK#362 lands on `develop`, re-point the submodule at the merged SHA and amend. Merge
order is SDK first, always.

## The visual pass (the only thing in the way)

```sh
cd ../SSPlayerForWgpu && cargo run -p ssplayer-wgpu-viewer
```

The viewer loads `assets/overall`, so **drop `assets/Ringo/Ringo.ssab` on the window**. Ringo
is the right subject: 25 animations on one rig (tracks require one SSAB and one rig), bone +
mesh so the whole downstream — mix → hierarchy → skinning — is exercised, and a real rig rest
pose to fall back to. If something looks wrong and you cannot tell whether it is the mix or
the skinning, `assets/overall/Basic.ssab` is plain sprite parts and separates the two.

`Space` first: judging `alpha` is far easier when the animation is not also moving. The HUD
prints each live track's alpha, blend mode and scrub position, so you can always tell what you
are looking at.

| # | Do | Correct | Broken looks like |
|---|---|---|---|
| 1 | `0` — fade track 0 out | settles onto the rig's rest pose over 500ms; `0` again brings it back | parts collapse to the origin, the character squashes, or it flies off screen → the floor is not the transformed setup pose |
| 2 | `S`, then `1` / `2` | alpha 0 is the base animation, 1 is the overlay, and everything between is a plausible in-between | the character **shrinks or drifts to the origin** as alpha rises → dipping, i.e. the coverage rule is not doing its job |
| 3 | `A` (from 2) | the overlay is *added on top of* the motion underneath. **Exaggerated is correct here** — Ringo's overlays are whole-body performances, not axes | indistinguishable from mix, or the pose jumps at alpha 0 |
| 4 | `C` repeatedly | 400ms transitions with no sink or shrink mid-way — watch `wait1`→`attack1` and `dead`→`run`, where the poses differ most | a dip mid-transition. Mash the key too: a little stutter is by design (the running fade completes first), but parts vanishing or freezing is not |
| 5 | `S`, `A`, `1` to ~0.3, then `,` / `.` | the pose steps frame by frame; held down it reads as a continuous axis | — |

1 and 4 are the ones that settle correctness. 2 and 3 confirm the model behaves as documented.
5 only shows the mechanism works — whether scrubbing *feels* like a performance is the
question `param_head_yaw` exists to answer.

## What to do next, in order

1. **The visual pass above.** If it passes, both PRs come out of draft.
2. **Author `param_head_yaw`** — one axis, ~61 frames, sweeping a neck bone −30°→+30°, with the
   **centre frame matching the setup pose** (that is what makes the additive delta correct).
   Then redo check 5 and judge the feel. This is the evidence the Vtuber product spec's central
   bet — that frame scrubbing carries continuous control — actually needs.
3. **Rewrite the Vtuber spec's §4-5 and §5-1** (Evernote note "Vtuber 検討", attachment
   `vtuber_and_effect_tools_spec.md`). Both reasons it gives for avoiding multi-axis modulation
   are now obsolete: "軸を足すほど全部の効きが弱くなる" is what track alpha fixes, and
   "キーを持たないパーツには setup pose 値が混ざる" is what the coverage rule fixes. Its tier
   table can be redrawn — additive is no longer the price of admission to Tier 3.
4. **Per-part masks**, when a densely-keyed existing asset needs layering without re-authoring.
   Not before: a track already reaches only as far as its animation keys.

## Re-running the measurements

The harness was a throwaway crate at
`<scratchpad>/mixprobe` — **session-scoped, so assume it is gone**. `SPEC` §2.4 records what it
measured, how, and every number, which is enough to rebuild it: it depends on `ssruntime` and
`ssconverter` by path, converts every `.sspj` under `~/Desktop/ss` plus `tests/`, and drives
the real `Context` / `get_frame_data`.

Two traps it cost time to find, both worth reading before writing another one:

- **Do not hand-pick animation pairs.** Four hand-picked Ringo pairs showed *zero* drag, because
  the attributes where coverage differs are ones the animator keyed at their neutral value —
  which is exactly what the hardcoded default supplies. The exhaustive sweep found 41 in the
  same asset.
- **Estimate strength per (part, attribute) and take the median**, not one least-squares fit.
  Where an overlay animates nothing, `mixed == base`, and a global fit reads that as "fully
  diluted" rather than "no opinion". That artifact alone reported 0.213 where the answer was
  0.300.
