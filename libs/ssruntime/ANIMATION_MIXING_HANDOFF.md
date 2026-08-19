# Animation Mixing — resume here

> **Transient.** Delete this file once both PRs below have merged. The durable record — the
> model, the measurements and the decisions — is `ANIMATION_MIXING_SPEC.md`, which outlives it.

Last updated 2026-08-19.

## Where it stands

The track stack is **implemented and merged into neither repo yet**. Both PRs are **open and
out of draft** (2026-08-19).
A headless geometric pass has been run over the whole downstream (`SPEC` §2.5) and the model
came out clean; it also found and fixed one real bug. What is left is review and merge — plus
`param_head_yaw`, which is the only way to judge whether additive and frame scrubbing are worth
what they cost.

| | |
|---|---|
| **SDK** | [#362](https://github.com/cri-middleware/SpriteStudio-SDK/pull/362) → `develop`, branch `feat/animation-mixing-tracks` |
| **wgpu** | [SSPlayerForWgpu#44](https://github.com/cri-middleware/SSPlayerForWgpu/pull/44) → `main`, branch `feat/track-api-passthrough` |

| Piece | State |
|---|---|
| Coverage rule (D2) | done, verified on 220 real-asset cases |
| Track stack, additive, crossfade, fades (D1) | done |
| Public API — Rust, C, WASM; old blend surface removed | done |
| Geometric pass, 552 `Ringo` pairs swept headless (`SPEC` §2.5) | done |
| `update()` stale-frame bug across a completing crossfade | found and fixed here |
| Player-facing docs, `20_limitations`, README, ROADMAP, CHANGELOG | done |
| `ssplayer-wgpu` pass-throughs + viewer bindings + seam tests | done |
| Visual pass on `ssplayer-wgpu` | done as far as Ringo allows; check 5 needs `param_head_yaw` |
| `param_head_yaw` axis, and judging the scrub with it | not done, needs authoring |
| Vtuber product spec rewrite | done 2026-08-19 — attachment replaced |
| Per-part masks | deferred by design (`SPEC` §7 item 9) |
| Shortest-path rotation, sign-crossing scale | measured, left open (`SPEC` §7 items 7-8) |

## Getting back to a working tree

```sh
# SDK
git switch feat/animation-mixing-tracks
cargo test --workspace --all-features        # 243 green

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

## The visual pass (what is left of it)

**Checks 1, 2 and 4 are settled** — the geometric pass swept all 552 `Ringo` pairs headless and
found no collapse, no shrink (worst 0.023%), no crossfade dip, no vanishing parts, and exact
identity at `alpha = 0` for both mix and additive (`SPEC` §2.5). Re-running them by eye is
optional. **Checks 3 and 5 are what still needs a person**, because "exaggerated is correct
here" and "does this feel like a performance" are not quantities.

**Two things you will see that are not bugs**, both measured and both looking exactly like the
dipping check 2 tells you to watch for — read `SPEC` §7 items 7-8 before reporting either:

- **Any pair involving `dead` squashes a part flat at `alpha = 0.5`.** That part is mirrored in
  one source and not the other (`scale_y` +1 vs −1), so a linear mix passes through zero.
  85 of 552 pairs do this.
- **Parts swing wide arcs on about half of all pairs.** Rotation is mixed as raw degrees, so a
  part at +131° in one source and −142° in the other travels 273° instead of 87°. 264 of 552
  pairs move some part ≥0.5× the character's own spread off the direct path.


```sh
cd ../SSPlayerForWgpu && cargo run -p ssplayer-wgpu-viewer
```

The viewer loads `assets/overall`, so **drop `assets/Ringo/Ringo.ssab` on the window**. Ringo
is the right subject: 24 animations on one rig (tracks require one SSAB and one rig), bone +
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

1 and 4 are the ones that settle correctness, and §2.5 has now settled them without an eye. 2
and 3 confirm the model behaves as documented. 5 only shows the mechanism works — whether
scrubbing *feels* like a performance is the question `param_head_yaw` exists to answer.

## What to do next, in order

1. ~~**Checks 3 and 5 by eye.**~~ Check 3 was walked in the viewer 2026-08-19 and additive was
   kept (`SPEC` §6.2 records the measurement behind that). **Check 5 cannot be settled on
   Ringo** — whether scrubbing reads as a performance is what item 2 exists to answer, so it was
   ruled undecidable without the asset rather than passed. **Both PRs are out of draft.**
   Merge order is SDK first; wgpu#44's submodule pointer still needs re-pointing at the merged
   SHA before it lands.
2. **Author `param_head_yaw`** — one axis, ~61 frames, sweeping a neck bone −30°→+30°, with the
   **centre frame matching the setup pose** (that is what makes the additive delta correct).
   Then redo check 5 and judge the feel. This is the evidence the Vtuber product spec's central
   bet — that frame scrubbing carries continuous control — actually needs.
3. ~~**Rewrite the Vtuber spec's §4-5 and §5-1**~~ **Done 2026-08-19.** The attachment on the
   Evernote note "Vtuber 検討" was replaced (verified byte-identical on round-trip). Nine places
   changed, not two: §4-5 rewritten with the measured tables, plus the 要約, §4-2, §4-4, §4-7,
   §4-10's tier table, §5-1 through §5-4, and open question #2. **The correction that mattered
   most**: the old text named additive as the fix for dilution, which is wrong — track `alpha`
   fixes that; additive fixes something else (holding an offset without damping the performance
   underneath). Every tier is now SDK-change-zero.

4. **Per-part masks**, when a densely-keyed existing asset needs layering without re-authoring.
   Not before: a track already reaches only as far as its animation keys.

## Re-running the measurements

Both harnesses were throwaway crates at `<scratchpad>/mixprobe` (§2.4, attribute-level) and
`<scratchpad>/mixgeom` (§2.5, geometric) — **session-scoped, so assume they are gone**. §2.5
records the traps that cost a wrong answer in the second one; they are a different pair from
the two below. `SPEC` §2.4 records what it
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
