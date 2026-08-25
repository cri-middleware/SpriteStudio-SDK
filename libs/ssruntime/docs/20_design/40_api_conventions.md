> [日本語](./40_api_conventions.ja.md) | **English**

# Player API conventions

The Brain has one contract, but a Player's public API is written in its host's language. Without a line drawn between what is unified and what is left to the language, the same feature grows a different name and a different argument order in every Player. That is what happened. This document draws the line.

**What is unified is the stem and the argument order.** Whether a `get` prefix appears, whether something is a property or a method, and what a failed lookup returns all follow the host language. `findPartIndex` returning `Option<u32>` in Rust and `-1` in C# is correct. Being called `partIndexOf` is not.

---

## 1. Argument order

Override methods put **the value being set immediately after `part`, with modifiers trailing and defaulted.**

```
setPartVisibilityOverride(part, hide, cascade = false)
setPartColorOverride(part, color, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartColorOverrideCorners(part, corners, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartCellOverride(part, cellmapName, cellName, priority = HoldUntilNextAnimation)
```

`blendOp` comes after `color`. Any other order forbids giving `blendOp` a default, which leaves every caller spelling out `Mix`. `Mix` is the zero value, and the colour's own alpha carries the strength, so it means "plain".

In a language without default arguments (Rust), leaving every parameter required is fine. What is unified is the order; the default is what a language that supports it gets on top.

**`corners` is one argument.** Do not expand the four colours into positional parameters. The order is `lt, rt, lb, rb` (left-top, right-top, left-bottom, right-bottom), the same order as the C ABI's `rgba_ltrtlbrb`. Use whichever of array, tuple or struct is natural in the language. Expanding it is what made `blendOp` land in a different place in each Player.

### This deliberately differs from the C ABI

The C ABI is `ss_runtime_set_part_color_override(context, part, blend_op, rgba, priority_mode)` — `blend_op` first. Players are the host-facing ergonomic layer, so they do not mirror it. **Reading the C ABI and copying its order will put you outside this convention.**

---

## 2. Name resolution — by name and by index

**The unsuffixed form takes a part name. An index is the optimisation, not the default.**

A part name is what the user has: it comes from the editor. An index is a cache obtained through `findPartIndex`. An API whose default form requires you to have already done the lookup has it backwards.

How the index form is offered is up to the language.

| Language | Index form |
|---|---|
| TypeScript | A `string \| number` union on the same method |
| C# | Overloads `(string, …)` / `(int, …)` |
| Dart | Sibling method `…ByIndex` |
| Rust | Sibling method `…_by_index` |
| GDScript | Sibling method `…_by_index` |
| Python | A single form that accepts either |

What `findPartIndex` returns when the name is unknown is also up to the language: `Option<u32>` in Rust, `-1` elsewhere, matching the C ABI.

---

## 3. Naming

**The form of a name follows what the name denotes.** When in doubt, classify it here first, then name it.

| What it denotes | Form | Examples |
|---|---|---|
| An ongoing activity | Present progressive `is …ing` | `isPlaying` / `isPausing` / `isPlayingForward` |
| A completed state that persists | Past participle `is …ed` | `isFinished` |
| A pulse lasting one tick | Not `is …` | `justLooped` |
| A boolean setting or toggle | Noun phrase + `Enabled` | `frameSkipEnabled` / `subFrameEnabled` |
| A quantity | Noun phrase | `speedScale` / `loopCount` / `frameRate` |
| A lookup that can fail | `find…` | `findPartIndex` |

`isFinished` and `justFinished` are different things. The first is the standing state — every configured cycle has played; the second is its rising edge, true for exactly one tick. Read the second to fire a one-shot completion.

The third row is the one that gets missed. `isLooped` denotes "the last update crossed a loop boundary" — a pulse cleared every tick, not a state. `is` suggests something standing, and invites the exact bug of reading it a tick late. `has…` is wrong for a pulse for the same reason: it reads as "at some point, ever".

On `isPausing` versus `isPaused`: the latter is arguably better English. The former wins because it keeps the tense of `isPlaying`, and because the Brain itself is `ss_runtime_is_pausing`. Two paired predicates disagreeing about tense does more harm.

### Standard vocabulary

Use the same stem for the same concept. Casing and word separation follow the language.

| Concept | Stem |
|---|---|
| Transport running (true while paused) | `isPlaying` |
| Held by a pause | `isPausing` |
| Configured loops have all played (sticky) | `isFinished` |
| The last update crossed a loop boundary (pulse) | `justLooped` |
| The last update completed playback (pulse) | `justFinished` |
| The playhead is actually travelling forward | `isPlayingForward` |
| Frame skipping on or off | `frameSkipEnabled` |
| Sub-frame interpolation on or off | `subFrameEnabled` |
| Playback rate multiplier | `speedScale` |
| Loop count (`-1` is infinite) | `loopCount` |
| Playback frame rate | `frameRate` |
| Limit playback to a frame range | `setAnimationSection` |
| Heading and round-trip style | `playbackDirection` / `playbackStyle` |
| Resolve a part name to its index | `findPartIndex` |
| Names of the animations | `animationNames` |
| Names of the cellmaps | `cellmapNames` |
| Names of the cells in a cellmap | `cellNames` |
| Play from a given frame | `play(startFrame)` |

Playback heading is two enums, `playbackDirection` and `playbackStyle`. Do not model it as two booleans ("is it reversed", "is it ping-pong") — that makes it impossible to set the pair atomically in one call.

Override priority takes the Brain's own enumerator names: `OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` / `Permanent`.

### Terminology

**`cellmap` is one word.** Not `cellMap`, not `CellMap`. The schema and these docs are consistent on it.

---

## 4. Defaults

A default that differs between Players changes behaviour silently when a project is ported. These are not the Player's to choose.

| Setting | Default | Why |
|---|---|---|
| Override `priority` | `HoldUntilNextAnimation` | The C ABI's own default |
| Visibility override `cascade` | `false` | A keyframed hide never cascades in SS7, so cascading is the opt-in |
| Colour override `blendOp` | `Mix` | The zero value; the colour's alpha carries the strength, so it means "plain" |
| Part following on a hidden part | Keep following | A part hidden for one frame should not take its follower with it |
| `isPlayingForward` before playback | `true` | Forward is the resting state; false would gate a host's audio off on its first frame |

`subFrameEnabled` and `frameSkipEnabled` may default to whatever suits the host engine. Those two are the Player's call.

---

## 5. Departing from this document

There are legitimate reasons to depart — chiefly a head-on collision with the host language's or engine's conventions. When you do, **write the reason next to the code.** A departure with no stated reason is indistinguishable from a mistake to whoever reads it next.

ForUnity passing Y-up to the coordinate system while every other Player passes Y-down is a legitimate departure: that is the engine's own axis, and the code says so.
