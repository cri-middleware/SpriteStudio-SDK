# Maintaining the official Players — getting started

**Who this is for: whoever maintains the official Players.** It is a working agreement about our own repositories — what their READMEs answer, and what their bundled samples have to do — not a requirement on anyone porting SpriteStudio to a new engine. The porting guide states what a Player has to be; this states how we keep the seven we ship consistent with each other. If you are writing your own Player, the document you want is the [Player API conventions](./libs/ssruntime/docs/20_design/40_api_conventions.md), which is about the artifact rather than about our repositories.

The API conventions stop the same feature from growing a different name in every Player. But an aligned API loses most of its value **if the first five minutes are a different experience in every Player.** The shared vocabulary pays off for the reader moving to their second Player; the reader touching their first one meets the README.

This document fixes what the first screen answers. **What is shared is four questions and their order. How they are answered belongs to the host.**

---

## 1. The four questions the first screen answers

| # | Question | Example |
|---|---|---|
| 1 | What do I install? | One package-manager line, or "extract this and drop it in" |
| 2 | How does a `.sspj` become a `.ssab`? | Drag it onto the editor, or one conversion command |
| 3 | How do I get it on screen? | Place the node / component / element, assign the `.ssab`, play |
| 4 | Where do I read next? | Three links at most |

**The order is part of the convention.** Do not put concept explanations, architecture diagrams or feature lists before question 1. Whoever is reading there has not decided to use this Player yet.

When the distribution route is not the host's usual one, **put one sentence of reason right next to that line.** "Package Manager's Add package from URL does not work for this package, since the native binaries ship only in the tarball" is a good example. A detour with no reason beside it is indistinguishable from carelessness.

## 2. How they are answered belongs to the host

Players come in two shapes.

- **Editor-first** — install the add-on or package, drag the `.sspj` onto the editor, place a node or component and assign the `.ssab`. **Not one line of code.**
- **Code-first** — install the package, write the smallest thing that runs.

**Say on the first screen which shape this Player is.** A Hello World snippet in an editor-first Player is unnatural for that host. And "you start without writing a script" is a selling point, not something to leave implicit in a numbered list — it is the first thing a reader arriving from that engine wants to know.

## 3. A sample must run with nothing of the reader's own

**Opening the bundled sample must produce a first frame without the reader supplying a `.sspj`.** A Player that does not manage this is asking them to author before they can try.

### 3.1 The sample to show is `tests/Ringo`

The SDK's `tests/` holds five projects, and they are not the same kind of thing. **`Ringo` is the one to show; `overall` is the one to verify against.** Standardise the getting-started sample on `Ringo`. The same picture appearing in every Player is how someone working across several of them confirms they have arrived.

### 3.2 Keep the development project and the verification project apart

The file that records where converted assets came from — Godot's `.ssplayer_sources.cfg`, Unity's `SpriteStudioSourceMap.json` — **records everything that project ever converted.** Let one project serve as both the developer's scratch pad and the sample, and it fills with that developer's local absolute paths and stops being committable.

This is not held in place by discipline. It follows from whether the projects are separate.

| | SDK paths | local paths | committed |
|---|---|---|---|
| Godot `examples/Ringo` and the three other verification projects | 3–21 | 0 | yes |
| Godot `examples/dev_module` (development) | 0 | 51 | no |
| Unity `examples/UnityTestProject` (was both) | 0 | 95 | could not be |

A separated project's file stays clean on its own.

### 3.3 If the converted assets are not committed, opening the project must convert

Whether to commit the converted assets — the `.ssab` and its textures — is the host's call, and leaving them out keeps the repository light. But **if they are not committed, opening the project has to read that file and run the conversion.** Without it the reader ends up hunting down a `.sspj` and converting it themselves, which fails the rule above.

Godot's add-on reads the cfg and converts on open, so it commits no example binaries at all. A host without that machinery should commit one sample's worth and no more.

### 3.4 Committed assets need a way to regenerate them

A committed `.ssab` is a build output, and it goes stale when the SDK's `tests/` change. **Provide a `deploy-examples` script and run it at release time.** `ssconverter-cli` performs the conversion, so the Player's editor is not needed for it.

## 4. Departing from this document

The same as [Player API conventions §7](./libs/ssruntime/docs/20_design/40_api_conventions.md#7-departing-from-this-document). Depart when the host's conventions genuinely collide with this one. **Write the reason where the departure is.**
