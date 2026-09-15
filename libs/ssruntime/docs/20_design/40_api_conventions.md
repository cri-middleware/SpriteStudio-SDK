> [日本語](./40_api_conventions.ja.md) | **English**

# Player API conventions

The Brain has one contract, but a Player's public API is written in its host's language. Without a line drawn between what is unified and what is left to the language, the same feature grows a different name and a different argument order in every Player. That is what happened. This document draws the line.

**What is unified is the stem, the argument order and the set.** Whether a `get` prefix appears, whether something is a property or a method, how an event is delivered, and what a failed lookup returns all follow the host language. `findPartIndex` returning `Option<u32>` in Rust and `-1` in C# is correct. Being called `partIndexOf` is not.

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
| The animation that is set up | `currentAnimation` |
| Names of the cellmaps | `cellmapNames` |
| Names of the cells in a cellmap | `cellNames` |
| Play from a given frame | `play(startFrame)` |
| Where the playhead is (fractional) | `frameNo` |
| Move the playhead | `frameNo` (property form) / `seek(frameNo)` (verb form) |
| Names of the parts, in part-index order | `partNames` |
| How many frames the animation has | `totalFrames` |
| Hold, and lift the hold | `pause` / `resume` |
| End playback where it stands | `stop` |
| Mirror the drawing, without moving the host transform | `flipX` / `flipY` |
| A part's composed matrix for this frame | `partTransform` |
| Whether a part is hidden on this frame | `isPartHidden` |
| Whether a part is a skinned mesh | `isPartSkinnedMesh` |

**`currentAnimation`, not `animationName`.** The latter sits one character from
`animationNames`, and those two are the pair a host uses together — list them to
build a picker, read back which is selected. Godot names this same thing
`AnimationPlayer.current_animation`; `AnimatedSprite2D.animation` is the selection
on a sprite that has no transport of its own.

Selecting one stays `setAnimation(name)`. Where a host models selection as a
property instead of a call, that property carries the read's stem.

**`frameNo`, not `frame`.** The value is fractional — sub-frame interpolation puts
the playhead between two authored frames — and `frame` is already the word for a
rendered frame and for the `FrameData` a tick produces. `AnimatedSprite2D.frame` in
Godot is an *integer* index into a `SpriteFrames` set, so borrowing that spelling
would promise a Godot reader the wrong type. The Brain calls it `frame_no`.

Moving the playhead follows §2's rule about properties and methods: where the host
writes through a property, it is the same stem written to; where the host prefers a
verb, it is `seek`. Both are listed because both are correct — what is not correct
is a third stem.

**`partNames` is required, and a part *count* is not a substitute.** A count cannot
turn the `partIndex` on an event back into something a human reads, and cannot fill
a part picker. The length of the list is the count.

Playback heading is two enums, `playbackDirection` and `playbackStyle`. Do not model it as two booleans ("is it reversed", "is it ping-pong") — that makes it impossible to set the pair atomically in one call.

**Both take the Brain's own enumerator names** — `Forward` / `Backward` and `Normal` / `PingPong` — the way override priority does below. Casing and word separation follow the host, the stem does not: `Reverse` is not a spelling of `Backward`, it is a second word for one concept, and a reader who has the family's docs open will not find it. Two enums also means two prefixes where a language needs them: pair `DIRECTION_*` with `STYLE_*`, not `PLAY_*` with `STYLE_*`.

Override priority takes the Brain's own enumerator names: `OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` / `Permanent`.

### Motion blending

The track stack is one implementation deep — `ssplayer-wgpu` — and registered here
**because** of that. The vocabulary for it is being written down before the second
Player writes its own, since that is the only moment it costs nothing: the stems
below come from the C ABI the Brain already publishes, and ForWgpu's Rust surface
already spells them this way.

A **track** folds onto the pose the tracks below it produced. A **source** is one
animation blended within a track; a track's `main` source is the one supplying its
structure.

| Concept | Stem |
|---|---|
| How many tracks exist | `trackCount` |
| Put an animation on a track | `setTrackAnimation` |
| The track's main source | `trackMainSource` |
| Empty a track | `clearTrack` |
| How strongly a track folds onto the pose below | `trackAlpha` |
| How it folds (`Mix` / `Additive`) | `trackBlend` |
| The track's playhead | `trackFrame` |
| The track's speed | `trackSpeed` |
| Fade a track from one animation to another | `crossfade` |
| Fade the track's own alpha | `fadeTrack` |
| How far the fade in flight has got | `trackFadeProgress` |
| Stop the fade in flight | `cancelTrackFade` |
| Blend another animation into a track | `trackAddSource` |
| A source's weight within its track | `sourceWeight` |
| A source's playhead | `sourceFrame` |
| The weight below which a source stops firing events | `sourceEventThreshold` |

**`crossfade` is not `fadeTrack`.** The first replaces what a track is playing; the
second changes how much of the track is heard at all. A Player that offers one and
names it for the other leaves the second unnameable.

A read and its write share the stem, per §3 — `trackAlpha` is both, spelled the way
the host spells a settable. `setTrackAnimation` keeps its verb because there is no
matching read: what comes back is a source handle, and that is `trackMainSource`.

### Terminology

**`cellmap` is one word.** Not `cellMap`, not `CellMap`. The schema and these docs are consistent on it.

---

## 4. Events

A Player publishes six events. **How** they are delivered is the host's business;
**what they are called** and **which ones exist** are not.

### Stems

| What it reports | Stem |
|---|---|
| A **User Data** key was crossed | `userData` |
| A **Signal** key was crossed | `signal` |
| An **Audio** key was crossed | `audio` |
| Playback headed out | `animationStarted` |
| Every configured loop has played | `animationFinished` |
| The playhead crossed a loop boundary | `animationLooped` |

The `on` prefix, the casing and the word separator follow the host: a Godot signal
is `animation_finished`, a C# event `OnAnimationFinished`, a TypeScript
subscription `onAnimationFinished`, a Python constructor kwarg
`on_animation_finished`. What is unified is the stem — **all of it.** Shortening
`animationFinished` to `finished` is not a spelling difference, it is a different
name, and `finished` alone does not say *what* finished on an object that also
owns effect slots, instance slots, audio and textures.

The last three name the same things §3's vocabulary table already names —
`isFinished`, `justLooped`. **Keep the pair spelled the same.** An event called
`onComplete` or `onAnimationEnd` sitting next to a state called `isFinished`
describes one concept with two words, and neither name finds the other.

### The set

Those six are the required set. Publishing fewer leaves a host unable to observe
something every sibling exposes; the three timeline events in particular are the
whole point of authoring keys.

Three more show up in some Players and are **not** required:

| Not required | Where it earns its place |
|---|---|
| an animation-changed event | The host called `setAnimation` itself, so it already knows. Publish one only where a Player has another route in — an inspector field, a declarative prop — and say so next to it |
| a per-frame hook | Only meaningful in a Player that owns a scene graph, where it is the seam part-following hangs off, after the frame's matrices are final |
| a raw, undecoded event feed | An escape hatch past the decoded three. Worth having where decoding is expensive or the host wants the indices; never a substitute for the decoded events |

### Delivery is the host's

Signals in Godot, `event` plus `UnityEvent` in Unity, a subscribe call returning an
unsubscribe in TypeScript, an assignable callback in Dart and React, a queue the
host drains in Ren'Py, a slice it polls in Rust. Every one of those is right in its
host. Do not unify them.

Two things do follow from that, though.

**Say how many listeners you take.** That is the host's call as well — a Flutter
widget callback and a React prop are single-slot by nature, a Godot signal and a
C# event are not — but it is not something a caller can discover safely. A host
that assumed it could add a second listener finds out when the first one silently
stops firing. Write which one you built.

**A Player with no callback mechanism at all publishes the pulses instead.** Where
the host polls rather than subscribes, there is nothing to take the completion edge
from, so `justFinished` joins `justLooped` in the read-outs. That is why
`justFinished` exists in exactly one Player: it is the compensation for having no
events, not a gap in the Players that have them.

---

## 5. The tick is the host's clock

A Player driven by the host takes a delta. **Its unit follows the host clock** —
seconds where the engine hands you seconds (`_process(delta)`, `Time.deltaTime`),
milliseconds where it hands you milliseconds (`requestAnimationFrame`), a
`Duration` where the host has one. Converting on the way in only adds a place for
a factor of 1000 to hide.

What is required is that **the signature says which**: a parameter named `deltaMs`
/ `dt_ms` / `delta_secs`, or a type that carries the unit itself. A bare `delta`
passes only where the host's own clock makes it unambiguous — Godot's `delta` and
Unity's `deltaTime` are seconds to every reader of that engine.

The verb is the host's too. `update` and `advance` are both in use and neither is
worth churning a quickstart over.

---

## 6. Defaults

A default that differs between Players changes behaviour silently when a project is ported. These are not the Player's to choose.

| Setting | Default | Why |
|---|---|---|
| Override `priority` | `HoldUntilNextAnimation` | The C ABI's own default |
| Visibility override `cascade` | `false` | A keyframed hide never cascades in SpriteStudio, so cascading is the opt-in |
| Colour override `blendOp` | `Mix` | The zero value; the colour's alpha carries the strength, so it means "plain" |
| Part following on a hidden part | Keep following | A part hidden for one frame should not take its follower with it |
| `isPlayingForward` before playback | `true` | Forward is the resting state; false would gate a host's audio off on its first frame |

`subFrameEnabled` and `frameSkipEnabled` may default to whatever suits the host engine. Those two are the Player's call.

### Part-following options

A Player that offers part following — an attachment, a tracker, an anchor — offers the same options with the same defaults.

| Option | Default | Why |
|---|---|---|
| position | `true` | A follower that does not follow the position is not following |
| rotation | `true` | A weapon in a hand turns with the hand |
| scale | **`false`** | A follower is placed **at** the part, not **sized by** it. A health bar over a head or a name plate rarely wants to stretch with the part's scale animation |
| on hidden | keep following | A part hidden for one frame should not take its follower with it |

**The prefix is the host's business.** What is unified is the set of three and their defaults, not the spelling. Where a bare `scale` would read as the host transform's own, a `drive` / `update` prefix is fine — ForGodot uses `update_position` and its siblings because it is modelled on `RemoteTransform2D`.

---

## 7. Reporting failure

**Which mechanism reports a failure is the host's. What the report says is not.**

A `false`, a `null`, an exception, a `Result`, an `Option` — each is right in its own language, the same way §2 leaves `findPartIndex`'s miss to the language. What is unified is that the failure is *legible*: the caller can tell what they asked for, what was there instead, and therefore what to write next.

### A lookup may fail quietly. A command may not.

| Shape | On failure |
|---|---|
| `find…` — a lookup that can fail | The return value is the whole report |
| `setAnimation` / `load…` / a `set…Override` — a command that did nothing | Return **and** report |

`findPartIndex` returning `-1` is a complete answer to "is this part here?" — §3 classifies it as a lookup precisely so that the caller is expected to branch on it. `setAnimation` returning `false` is not the same thing. It is a command the caller believed would take effect, and the symptom is a sprite still playing the previous clip, or drawing nothing at all. **A boolean nobody is obliged to read is not a report.**

### The report carries what only it knows

**Required is what the caller cannot get any other way** — the same test that
decides what the API exposes at all. That is two things, and they are cheap
because a failing call is already holding both:

| Call | The report names |
|---|---|
| `setAnimation` | the pack, and the name it was given |
| Pack load | the URL / path / asset key it was given, **and the underlying cause, verbatim** |
| Texture resolution | the file it looked for, and the cellmap it belongs to |
| `setPartCellOverride` | the pack, and the cellmap and cell it was given |
| The part-name form of any override | the part name it was given |

The cause on the load row is there because nothing else carries it: a version
mismatch knows which version was expected and which arrived, and if the report
drops those numbers they are gone.

**Listing what *would* have worked is not required.** It reads as the obvious
kindness, and it is where this rule was first written too broadly. `animationNames`,
`partNames`, `cellmapNames` and `cellNames` are all in §3's required vocabulary —
a caller who wants the list can already print it, and in a host with an inspector
it is on screen. Meanwhile a required list costs every Player a truncation policy
for the pack with two hundred animations. Include one where it is short and
obviously helpful; do not build one to satisfy this section.

**Name what the caller wrote.** They wrote `"wakl"`; they did not write `0x4f3a21bc`. A name hash is the Brain's identity for a name, not a name — reporting one asks the caller to run a hash by hand to find out which of their own strings is wrong. The same goes for a part index in a report about a call that took a part name, and for a load failure that reports the cause but drops the path it was handed.

### Report once, at one layer

**Once**, because these calls sit on the hot path and **a failing one is not covered by the guard that makes a repeating one cheap.** A state machine may assert its clip every frame; a Player skips that work by comparing against the animation it has set up — and the name that failed never became the one it has set up, so every frame takes the failing branch again. An unguarded log turns one wrong string into a frame-rate-speed flood that buries everything else in the console. Key the guard on the failure rather than on the call site, so that a second wrong name still gets its own line — ForRenPy warns once per `(kind, name)` pair.

**At one layer**, because a Player with an internal player under its public one has two places to write the same line. Report from the layer the host called. The inner one names a type the caller cannot act on.

### Where it goes, and how loudly, is the host's

Godot's Output panel, Unity's Console, the browser console, `FlutterError`, Ren'Py's log. Where the host has a structured error channel the caller opted into — an `onError` prop, a rejected `Future`, a `Result` that must be consumed — that channel is the route, and the log is the backstop for the caller who wired up neither.

The level follows the blast radius. A pack that did not load leaves the Player unable to draw anything: that is an error. A `setAnimation` that did not take leaves it drawing the previous clip: that is a warning.

---

## 8. The C ABI's own rules

Everything above governs a Player's public API. The C ABI is the other frozen
surface, and it is the one every Player reads while writing that API — so it
needs its own rules, written down, or the drift lands in seven repositories at
once.

**Every exported symbol is `ss_<object>_<verb>_<noun>`.** The object is
`runtime` for anything that takes a context, and `resource`, `effect_slot` or
`instance_slot` for the three handle types that own themselves.

**The verb comes first and is never omitted.** `get_` / `set_` / `clear_` /
`find_` / `add_` / `load_` / `bind_` / `cancel_` / `fade_`. Every reader is
`get_`, every writer is `set_`, and a pair spells its noun identically —
`ss_runtime_get_frame_rate` against `ss_runtime_set_frame_rate`, never `get_fps`
against `set_frame_rate`. A hash-keyed twin takes the `_by_hash` suffix and
nothing else changes.

**The noun is §3's stem.** The C ABI is not exempt from the vocabulary table; it
is where the stem is first spelled out, and a Player that copies it verbatim
should land inside this document rather than outside it. **Where the C ABI and
the table disagree, the C ABI is the one that is wrong.** The test is whether a
Player has to *translate* rather than merely re-spell: re-spelling
`ss_runtime_just_looped` as `justLooped`, `JustLooped()` or `just_looped()` is
the host's business and costs nothing, but every Player having to explain in a
comment why its name differs from the Brain's is the symptom of a bad stem.

**An out-parameter that points at runtime-owned memory is `const`.** The buffer
belongs to the context until the next call that rebuilds it; `const unsigned
char **` says so, and the compiler enforces it on the C and C++ hosts. The
hosts without `const` — C#, `ctypes`, `ffigen` — are unaffected either way, so
there is no cost to being right here.

**Failure crosses as a code, never as a string.** A call reports failure by
return value; the caller retrieves the reason with `ss_runtime_get_last_error`
and turns it into text with `ss_runtime_error_message`. The numbers are a
compatibility surface in their own right — see `libs/ssruntime/src/error.rs`.

The deliberate departures from the Player layer are listed where they arise:
argument order (§1) and the `get_` prefix itself, which §3 leaves to the host
and which the C ABI settles as "always".

### Why this section exists

Before the first release a rename costs a submodule bump the Players are taking
anyway, plus the call sites. After it, the same rename is a deprecation cycle
across every Player and a migration note for anyone who shipped. **Names are
therefore settled here, before publication, and the window does not reopen.**

---

## 9. Departing from this document

There are legitimate reasons to depart — chiefly a head-on collision with the host language's or engine's conventions. When you do, **write the reason next to the code.** A departure with no stated reason is indistinguishable from a mistake to whoever reads it next.

ForUnity passing Y-up to the coordinate system while every other Player passes Y-down is a legitimate departure: that is the engine's own axis, and the code says so.
