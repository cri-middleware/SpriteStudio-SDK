> [日本語](./30_api_conventions.ja.md) | **English**

# Player API conventions

A Player's public API is written in its host's language. This page draws the line between what every Player unifies and what is left to each language's conventions.

**What is unified is the stem, the argument order and the set.** Whether a `get` prefix appears, whether something is a property or a method, how an event is delivered, and what a failed lookup returns follow each language's conventions. `findPartIndex` returning `Option<u32>` in Rust and `-1` in C# is correct. Being called `partIndexOf` is not.

---

## 1. Argument order

Override methods put **the value being set immediately after `part`, with modifiers trailing and defaulted.**

```
setPartVisibilityOverride(part, hide, cascade = false)
setPartColorOverride(part, color, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartColorOverrideCorners(part, corners, blendOp = Mix, priority = HoldUntilNextAnimation)
setPartCellOverride(part, cellmapName, cellName, priority = HoldUntilNextAnimation)
```

`blendOp` comes after `color`. Any other order forbids giving `blendOp` a default, which leaves every caller spelling out `Mix`. `Mix` is the zero value, and the color's own alpha carries the strength, so it means "plain".

In a language without default arguments (Rust), leaving every parameter required is fine. What is unified is the order; the default is what a language that supports it gets on top.

**`corners` is one argument.** Do not expand the four colors into positional parameters. The order is `lt, rt, lb, rb` (left-top, right-top, left-bottom, right-bottom), the same as the C-API's `rgba_ltrtlbrb`. Use whichever of array, tuple or struct is most natural in the language. Expanding it puts `blendOp` in a different place in each Player.

### 1-1. The order differs from the C-API

The C-API is `ss_runtime_set_part_color_override(context, part, blend_op, rgba, priority_mode)`, with `blend_op` first. Players are the layer responsible for host-facing ergonomics, so here they do not follow the C-API. **Copying the C-API's order as-is puts you outside this convention.**

---

## 2. Names and indices

**The unsuffixed form takes a part name. The form that takes an index is an optimization, not the default.**

What the user has is a part name, which comes from the editor. An index is only a cache obtained by looking it up with `findPartIndex`.

How the index form is offered is up to the language.

| Language | Index form |
|---|---|
| TypeScript | A `string \| number` union on the same method |
| C# | Overloads `(string, …)` / `(int, …)` |
| Dart | Sibling method `…ByIndex` |
| Rust | Sibling method `…_by_index` |
| GDScript | Sibling method `…_by_index` |
| Python | A single form that accepts either |

What `findPartIndex` returns when the part is not found is also up to the language: `Option<u32>` in Rust, and elsewhere `-1`, the same as the C-API.

---

## 3. Naming

**The form of a name follows what it denotes.** When in doubt, classify it with this table first, then name it.

| What it denotes | Form | Examples |
|---|---|---|
| An ongoing activity | Present progressive `is …ing` | `isPlaying` / `isPausing` / `isPlayingForward` |
| A completed state that persists | Past participle `is …ed` | `isFinished` |
| A pulse lasting one tick | Not `is …` | `justLooped` |
| A boolean setting | Noun phrase + `Enabled` | `frameSkipEnabled` / `subFrameEnabled` |
| A quantity | Noun phrase | `speedScale` / `loopCount` / `frameRate` |
| A lookup that can fail | `find…` | `findPartIndex` |

`isFinished` and `justFinished` are different things. The first is the standing state "the configured number of loops has played"; the second is a pulse that is true for exactly one tick, on that state's rising edge.

The third row is the one most easily gotten wrong. "The last update crossed a loop boundary" is a pulse that clears after one tick, so naming it `isLooped` makes it look like a state and invites the bug of reading it a tick late. For the same reason, `has…` is wrong for a pulse: it reads as "at some point, ever".

`justStarted` is true for exactly one tick: the first update that runs after `play` / `play(startFrame)`. `resume` does not raise it; lifting a pause carries playback on, it does not start it.

`isPausing` is used rather than `isPaused` to keep the same tense as its pair `isPlaying`; the Brain itself is `ss_runtime_is_pausing`.

### 3-1. Standard vocabulary

Use the same stem for the same concept. Casing and word separation follow each language's conventions.

| Concept | Stem |
|---|---|
| Whether playing (true while paused too) | `isPlaying` |
| Whether held by a pause | `isPausing` |
| Whether the configured loops have all played (state) | `isFinished` |
| Whether the last update was the first after `play` (pulse) | `justStarted` |
| Whether the last update crossed a loop (pulse) | `justLooped` |
| Whether the last update finished playback (pulse) | `justFinished` |
| Whether the actual direction of travel is forward | `isPlayingForward` |
| Frame skip on or off | `frameSkipEnabled` |
| Sub-frame on or off | `subFrameEnabled` |
| Playback speed multiplier | `speedScale` |
| Loop count (`-1` = infinite) | `loopCount` |
| Playback frame rate | `frameRate` |
| Set the playback section | `setAnimationSection` |
| Set the playback section by labels | `setAnimationSectionByLabel` |
| Playback direction / round-trip style | `playbackDirection` / `playbackStyle` |
| Look up an index from a part name | `findPartIndex` |
| List of animation names | `animationNames` |
| The animation currently set up | `currentAnimation` |
| List of cellmap names | `cellmapNames` |
| List of cell names | `cellNames` |
| Play from a given start frame | `play(startFrame)` |
| Playhead position (fractional) | `frameNo` |
| Move the playhead | `frameNo` (property form) / `seek(frameNo)` (verb form) |
| List of part names (in part-index order) | `partNames` |
| Total frame count of the animation | `totalFrames` |
| Hold / lift the hold | `pause` / `resume` |
| End playback where it stands | `stop` |
| Mirror the drawing without moving the host transform | `flipX` / `flipY` |
| A part's composed matrix for that frame | `partTransform` |
| Whether a part is hidden on that frame | `isPartHidden` |
| Whether a part is a skinned mesh | `isPartSkinnedMesh` |
| The front-most part a point is on | `hitTestPart` |
| The input for reducing motion | `reducedMotion` |
| The input for being selected (a toggle's on and off) | `selected` |

**`currentAnimation`, not `animationName`.** `animationName` is one character away from `animationNames`, and those two are a pair a host uses together (build a picker from the list, read back which one is selected). Godot also calls it `AnimationPlayer.current_animation`. Selecting one stays `setAnimation(name)`. In a host that expresses the selection as a property, that property is named `currentAnimation`.

**`frameNo`, not `frame`.** The value is fractional (with sub-frames, the playhead lands between frames). And `frame` is already the word for a rendered frame and for the FrameData a tick produces. Godot's `AnimatedSprite2D.frame` is an integer index, so borrowing that spelling promises a Godot reader a different type. The Brain's own name is also `frame_no`.

For moving the playhead, "property or method" is left to the host, as stated at the top of this page. A host that writes through a property assigns to the same stem; a host that prefers a verb uses `seek`. Do not create a third stem.

**`partNames` is required. A part count is not a substitute.** A count cannot turn the `partIndex` on an event back into something a human can read, and cannot build a part-list UI. The count is the length of the list.

Playback direction is expressed as two enums, `playbackDirection` and `playbackStyle`. Do not make it two booleans ("is it reversed", "is it ping-pong"): you could no longer set the pair together in one call.

**Use the Brain's enumerator names as-is** — `Forward` / `Backward` and `Normal` / `PingPong`. Casing and word separation follow the host, but the stem does not change. `Reverse` is not a spelling of `Backward`; it is a second word for the same concept, and a reader who has read another Player's docs will not find it. In a language that needs prefixes, pair `DIRECTION_*` with `STYLE_*`, not `PLAY_*` with `STYLE_*`.

Override priority also uses the Brain's enumerator names as-is: `OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` / `Permanent`.

The UI state words (`normal` / `hover` / `pressed` / `disabled` / `focused`) are in [UI Integration §2-1](../50_features/80_ui_integration.md#2-1-the-state-words-and-their-precedence).

### 3-2. Animation mixing

If a Player exposes the track stack, use the following stems. All of them are taken from the C-API.

A **track** layers onto the pose the tracks below it produced. A **source** is one animation blended within a track; the track's `main` source determines its structure ([Animation mixing](../50_features/10_animation_mixing.md)).

| Concept | Stem |
|---|---|
| Number of tracks | `trackCount` |
| Put an animation on a track | `setTrackAnimation` |
| The track's main source | `trackMainSource` |
| Empty a track | `clearTrack` |
| How strongly the track layers onto the pose below | `trackAlpha` |
| How it layers (`Mix` / `Additive`) | `trackBlend` |
| Where an animation laid on the track starts (`Free` / `Start`) | `trackSync` |
| The track's playhead | `trackFrame` |
| The track's speed | `trackSpeed` |
| The track's frame-rate override | `trackFrameRate` |
| The track's frame skip on or off | `trackFrameSkipEnabled` |
| The track's playback direction / round-trip style | `trackPlaybackDirection` / `trackPlaybackStyle` |
| The loop count of the track's animation | `trackLoopCount` |
| Whether the track's animation has played its loops (state) | `isTrackFinished` |
| Whether the last update wrapped the track's animation (pulse) | `trackJustLooped` |
| Fade while replacing the track's animation | `crossfade` |
| Fade the track's own alpha | `fadeTrack` |
| How far the fade in flight has got | `trackFadeProgress` |
| Stop the fade in flight | `cancelTrackFade` |
| Add another animation to a track | `addTrackSource` |
| A source's weight within its track | `sourceWeight` |
| A source's playhead | `sourceFrame` |
| The minimum weight at which a source fires events | `sourceEventThreshold` |
| The minimum share at which the outgoing side of a crossfade fires events | `trackOutgoingEventThreshold` |

**`crossfade` and `fadeTrack` are different things.** The first replaces what a track plays; the second changes how much the track takes effect.

The track forms of the playback settings take §3-1's stems with `track` in front, and track 0's keep §3-1's stems: `speedScale` is track 0's `trackSpeed`. On track 0, `crossfade` and `setTrackAnimation` change `currentAnimation` from the call on, as `setAnimation` does.

A read and its write share the stem, as in §3 (`trackAlpha` is used for both, and the form of the write follows the host). `setTrackAnimation` keeps its verb because there is no matching read. What comes back is a source handle, and you can read that with `trackMainSource`.

### 3-3. Terminology

**`cellmap` is one word.** Do not spell it `cellMap` or `CellMap`. The schema and this guide consistently use `cellmap`.

---

## 4. Events

A Player publishes six events. **How** they are delivered is left to the host's conventions; **what they are called** and **which ones exist** are not.

### 4-1. Stems

| What it reports | Stem |
|---|---|
| A **user data** key was crossed | `userData` |
| A **signal** key was crossed | `signal` |
| An **audio** key was crossed | `audio` |
| Playback started with `play` | `animationStarted` |
| Every configured loop has played | `animationFinished` |
| The playhead crossed a loop boundary | `animationLooped` |

The `on` prefix, the casing and the word separator follow the host: a Godot signal is `animation_finished`, a C# event `OnAnimationFinished`, a TypeScript subscription `onAnimationFinished`, a Python constructor argument `on_animation_finished`. What is unified is the **whole** stem. Shortening `animationFinished` to `finished` makes it a different name. On an object that also owns effects, instances, audio and textures, `finished` alone does not say what finished.

The last three are concepts already named in §3-1's vocabulary table (`justStarted` / `isFinished` / `justLooped`). **Keep paired names spelled the same.** Putting `onComplete` or `onAnimationEnd` next to `isFinished` calls one concept by two words.

### 4-2. The required set

The six above are required. If even one is missing, something observable in the other Players cannot be observed in that Player.

The following three exist in some Players but are not required.

| Not required | Where it is worth having |
|---|---|
| An animation-changed event | The host called `setAnimation` itself, so it already knows. Publish it only in a Player where the animation can be selected through **another entry point**, such as an inspector field or a declarative prop, and write the reason next to it |
| A per-frame hook | Meaningful only in a Player that has a scene graph. It is where part following hooks in, and it is called after the frame's matrices are final |
| A raw event feed | A way around the three decoded events. Useful where decoding is expensive or where the host wants the indices themselves. It is not a substitute for the decoded events |

### 4-3. Delivery is left to the host

Signals in Godot, `event` and `UnityEvent` in Unity, a subscribe call that returns an unsubscribe function in TypeScript, an assignable callback in Dart and React, a queue the host drains in Ren'Py, a slice polled in Rust. Each is the right form in its host. Do not unify them.

Do, however, follow these two rules.

**State how many listeners can be registered.** Whether it is one or many is left to the host (a Flutter widget callback and a React prop take one; a Godot signal and a C# event take many). A caller who assumed a second one could be added finds out only when the first one silently stops being called. Write down which you chose.

**If a Player has no callback mechanism, publish the pulses.** Where the host polls rather than subscribes, there is no other way to catch the completion edge. In that case, publish `justStarted` and `justFinished` alongside `justLooped`.

---

## 5. The tick unit follows the host's clock

A Player driven by the host receives the elapsed time. **Make its unit follow the host's clock**: seconds where the engine hands you seconds (`_process(delta)`, `Time.deltaTime`), milliseconds where it hands you milliseconds (`requestAnimationFrame`), a `Duration` where the host has one. Converting on the way in only adds one more place for a factor of 1000 to hide.

**Make the unit visible from the signature.** Use a parameter name such as `deltaMs` / `dt_ms` / `delta_secs`, or let the type carry the unit. A bare `delta` is fine only where the host's own clock fixes the unit (Godot's `delta` and Unity's `deltaTime` cannot be anything but seconds).

The verb is also left to the host. Either `update` or `advance` is fine.

---

## 6. Defaults

If the default for the same feature differs between Players, behavior changes silently when a project is ported. The following values are not the Player's to choose.

| Setting | Default | Why |
|---|---|---|
| Override `priority` | `HoldUntilNextAnimation` | The C-API's default |
| Visibility override `cascade` | `false` | A keyframed hide in SpriteStudio does not extend to descendants, so cascading happens only when requested explicitly |
| Color override `blendOp` | `Mix` | The zero value. The color's alpha carries the strength, so it means "plain" |
| Part following when the part is hidden | Keep following | A part hidden for one frame does not take its follower away |
| `isPlayingForward` before any animation is set up | `true` | Forward is the resting state. Returning false would let the host's audio be gated off on the first frame. Once an animation is set up, it reports the actual direction of travel, also while stopped (false after stopping on a ping-pong return leg, or with `Backward` set; [Playback control §5-3](../30_core/20_playback_control.md#5-3-playback-direction)) |
| The `loopCount` the host set | Kept across `setAnimation` and every switch of track 0 | The host sets it once. The Brain's setup resets it, so the Player sets it again (§8-1) |

The defaults of `subFrameEnabled` and `frameSkipEnabled` may follow the host engine's conventions.

Placement (`fit` / `align`) has defaults of its own, listed with the rest of its contract in [Placement: Fit and Align](../50_features/70_placement.md). They are not the Player's to choose either.

### 6-1. Part-following options

If your Player has part following (an attachment / tracker / anchor), offer the same options with the same defaults.

| Option | Default | Why |
|---|---|---|
| position | `true` | A follower that does not follow the position is not following |
| rotation | `true` | A weapon held in a hand turns with the hand |
| scale | **`false`** | A follower is **placed at** the part, not **sized by** it. A health bar above a head or a name plate usually should not stretch and shrink with the part's scale animation |
| When hidden | Keep following | A part hidden for one frame does not take its follower away |

**The prefix is left to the host's conventions.** What is unified is the set of three and their defaults, not the spelling. In a language where a bare `scale` could be misread as the host transform's own scale, a prefix such as `drive` or `update` is fine (Godot uses `update_position` and its siblings because it follows `RemoteTransform2D`).

---

## 7. Reporting failure

**Which mechanism reports a failure is left to the host's conventions. What it reports is not.**

`false`, `null`, an exception, a `Result`, an `Option` — each is the right form in its language. What is unified is that the failure is **legible**. Let the caller tell what they asked for and what was there instead — that is, what to write next.

### 7-1. A lookup may fail quietly. A command may not

| Form | On failure |
|---|---|
| `find…` — a lookup that can fail | Report through the return value only |
| `setAnimation` / `setAnimationSection` / `load…` / `set…Override` — a command that did nothing | Report through the return value **and** a report |

`findPartIndex` returning `-1` is a complete answer to "is this part here?". When `setAnimation` returns `false`, on the other hand, the caller issued the command believing it would take effect. The symptom is "keeps playing the previous animation" or "draws nothing". **A bool nobody has to read is not a report.**

### 7-2. What the report carries

**Only what the caller cannot get any other way is required.** All of it is information the failing call already holds.

| Call | What the report names |
|---|---|
| `setAnimation` | The pack, and the name it was given |
| Pack load | The URL / path / asset key it was given, and **the underlying cause, verbatim** |
| Texture resolution | The file it looked for, and the cellmap it belongs to |
| `setPartCellOverride` | The pack, and the cellmap name and cell name it was given |
| An override that takes a part name | The part name it was given |
| `setAnimationSection` | The start and end frames it was given |

The cause is on the load row because nothing else carries it. A version mismatch knows the expected version and the version that arrived; if the report drops those numbers, nobody can find them out.

**A list of what was there instead is not required.** `animationNames` / `partNames` / `cellmapNames` / `cellNames` are required vocabulary in §3-1, so a caller who wants the list can print it themselves. Requiring it would make every Player carry a truncation rule for the pack with 200 animations. Where it is short and clearly helpful, you may include one.

**Name what the caller wrote.** The caller wrote `"wakl"`, not `0x4f3a21bc`. A name hash is the Brain's identifier, not a name. Showing a part index in the report for a call that took a part name, and reporting only the cause of a load failure while dropping the path it was given, are the same mistake.

### 7-3. Once, and at one layer

**Report only once.** These calls sit on a path taken every frame. A state machine re-specifies its clip every frame, but a name that failed never becomes "set up", so the guard that does nothing for the same name (§8-2) does not protect it, and every frame takes the failure branch. Without limiting the count, one misspelling fills the log at frame-rate speed. Key the limit on the failure itself (such as the pair of kind and name), not on the call site. A second wrong name is then reported as a separate line.

**Report at one layer only.** In a Player that has an internal player under its public layer, there are two places that can write the same line. Report from the layer the host called. A report from the inner layer shows the name of a type the caller cannot touch.

### 7-4. Where it goes and how loudly are left to the host

Godot's Output panel, Unity's Console, the browser console, `FlutterError`, Ren'Py's log. Where the host has a structured error channel the caller explicitly connected (an `onError` property, a rejected `Future`, a `Result` that must be consumed), that channel is the main route, and the log is the backstop for the caller who connected neither.

The level follows the size of the impact. A pack that failed to load leaves the Player drawing nothing, so it is an error. A `setAnimation` that did not take effect only keeps the previous clip drawing, so it is a warning.

---

## 8. Selecting and playing an animation

The meaning of the C-API's playback control is in [Playback control](../30_core/20_playback_control.md). This section decides how to expose it in a Player's public API.

### 8-1. Separate selecting from playing

**Expose `setAnimation(name)` and `play()` as separate methods.** `setAnimation` calls `ss_runtime_setup_animation`; `play` calls `ss_runtime_play` (or `ss_runtime_play_with_start_frame`).

| API | What it does |
|---|---|
| `setAnimation(name)` | Selects the animation and waits on its first frame. **Does not start playback.** Allocates memory inside the Brain |
| `play()` / `play(startFrame)` | Plays the selected animation from the start (or from the given frame). Does not allocate memory |
| `pause()` / `resume()` | Holds where it stands / lifts the hold and continues playback |
| `stop()` | Ends playback where it stands |

There are three reasons to separate them.

- `ss_runtime_setup_animation` allocates memory. Calling it inside `play` makes an allocation run on every play.
- You can show the first frame's pose without playing.
- You can finish heavy preparation (setup, and loading the assets that animation uses) before `play`, so playback can start at once.

**Do not fold `resume` into `play`.** `play` always starts from the beginning, so using it to lift a pause sends the playhead back to the start. **Publish `isPausing` alongside `isPlaying`.** `isPlaying` stays true while paused.

`ss_runtime_setup_animation` resets the playback section to its default, so set it inside `setAnimation` or before `play`.

**The Player keeps the loop count the host set.** `ss_runtime_setup_animation` resets the loop count too, but in the Player's public API a `loopCount` the host set survives `setAnimation`. Hold the value the host last set, and set it again with `ss_runtime_set_loop_count` after `setAnimation` and after every call that switches track 0's animation (a `crossfade` or `setTrackAnimation` on track 0).

Speed, the frame-rate override, frame skip and playback direction are kept across setups, so there is no need to set them again ([Playback control §3](../30_core/20_playback_control.md#3-what-setup-resets)).

**A section named by labels is the same state.** `setAnimationSectionByLabel(startLabel, endLabel, startOffset = 0, endOffset = 0)` resolves each label to its frame in the animation's label table and sets the section the way `setAnimationSection` does; it does not play. The section runs from the start label's frame plus `startOffset` to the end label's frame plus `endOffset`, the rule an Instance part's start and end labels follow. Set it where any section is set: inside `setAnimation` or before `play`.

`setAnimationSectionByLabel` is a command (§7-1), so in these cases return `false`, report it once, and leave the section and the playhead as they were. An Instance part quietly drops a label it cannot find to the edge of the animation; this API does not.

- A label the animation does not have. The report names only the labels that are missing.
- Labels and offsets that land on a section `setAnimationSection` refuses (both ends inside the animation, the end before the start). The report names both the labels given and the frames they landed on.

`_start` and `_end` name the animation's own first and last frame even though the label table does not list them (not the ends of the current section). A real label of the same name, if there is one, is used instead.

### 8-2. Do nothing when the selected animation is specified again

**If `name` is the animation already set up, return from `setAnimation(name)` without calling the C-API.**

It is a "select" operation, so if there is nothing to reselect, it does nothing. `ss_runtime_setup_animation`, on the other hand, is not a call that does nothing. It returns playback to Stopped, rewinds the playhead, discards the playback section, and clears every override except `Permanent` ones. A state machine that specifies the current clip every frame is perfectly ordinary host code.

```cpp
// Without the guard, this animation never plays
player.setAnimation(state.clip);
```

The guard also prevents another bug. `ss_runtime_set_loop_count` resets the remaining loop count on every write, so a Player that sets the loop count on every setup also resets it every frame when setup runs every frame, and a finite loop never ends. Put the guard before that setting.

**Compare "already set up" against the state the Brain holds, and always invalidate it when a resource is bound.**

- Do not compare against the name the host last requested. A component that holds the animation name serialized has that value before the first setup, so the first `setAnimation` is skipped and nothing is set up.
- After `ss_runtime_bind_resource` (and the loading code that calls it), nothing is set up. Clear "already set up" on bind. Otherwise, with the next pack that has an animation of the same name as the previous pack, the first `setAnimation` is skipped and that pack cannot be played. In a set of characters exported from one project, overlapping names are normal.

To restart an animation from the beginning, use `play`; to discard overrides, use the clear-overrides API; to discard the playback section, set a section that covers the whole animation.

All seven official Players behave this way. The internal layer that wraps the C-API keeps the C-API's names and meanings as-is (`SpriteStudioInternalPlayer.SetupAnimation`, `SsInternalPlayer.setupAnimation`, and so on). The guard goes in the user-facing layer above it.

---

## 9. Reading a part

`partTransform`, `isPartHidden`, `hitTestPart` and every other read of a part answer from **the frame the last tick built**: the world matrices and the FrameData that tick's `ss_runtime_get_frame_data` returned, which stay valid until the next call ([Render Loop §1](../30_core/40_render_loop.md#1-soa-buffers)). **A read does not build a frame.**

| When the host reads | What it gets |
|---|---|
| After a tick | That tick's frame, the one being drawn |
| After setting or clearing an override, before the next tick | The last tick's frame. The override shows from the next tick |
| Again, with no tick in between | The same answer |
| After a tick that built no frame, such as one a Player skips for an off-screen character | The last frame built, so a part's pose and its hide flag stay a pair |
| Before the first tick | `isPartHidden` answers "not hidden" for a part that exists. "Hidden" would send the caller down its hidden branch before anything has been drawn |

A read that built a frame would cost a frame per call, and part following reads every tick, once per follower. It would also change what the next tick builds. An `OverwriteOnNextKeyframe` override skips the key on the frame it was set on only in the update in which it is first evaluated ([Part Overrides §1-3](../50_features/20_part_overrides.md#1-3-priority-modes)). After `play` or a seek, the next update passes the frame it starts on, so an override set there needs that update to evaluate it first. A read that built a frame before the update would take the first evaluation, and the update would hand the override back to the animation.

---

## 10. Departing from this document

Where the convention collides head-on with the host language's or engine's conventions, you may depart from it. When you do, **write the reason next to the code.** A departure with no stated reason is indistinguishable from a violation to whoever reads it next.

Unity passing Y-Up to the coordinate system while the other Players pass Y-Down is an example of a legitimate departure. It is because that is the engine's coordinate system, and the reason is written in the code.

---
[Back to contents](../README.md)
