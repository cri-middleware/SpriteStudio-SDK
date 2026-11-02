# Maintaining the official Players

**Who this is for: whoever maintains the official Players.** It is a working agreement about our own repositories — what their READMEs and the guides behind them answer, and what their bundled samples have to do — not a requirement on anyone porting SpriteStudio to a new engine. The porting guide states what a Player has to be; this states how we keep the seven we ship consistent with each other. If you are writing your own Player, the document you want is the [Player API Conventions](./libs/ssruntime/docs/20_design/30_api_conventions.md), which is about the artifact rather than about our repositories.

The API conventions stop the same feature from growing a different name in every Player. But an aligned API loses most of its value **if the first five minutes are a different experience in every Player.** The shared vocabulary pays off for the reader moving to their second Player; the reader touching their first one meets the README.

This document fixes what the first screen answers, and who it answers to. **What is shared is a short why, the four questions after it, their order, what each step names, one tree for placement, the two doors past them, a guide behind the first that keeps its reader, and a shelf behind the second. The rest of how they are answered belongs to the host.** §5 then names the Player the others are read against when their behaviour parts, and §6 fixes the shape of their roadmaps.

---

## 1. The four questions the first screen answers

| # | Question | Example |
|---|---|---|
| 1 | What do I install? | One package-manager line, or "extract this and drop it in" |
| 2 | How does a `.sspj` become a `.ssab`? | Drag it onto the editor, or one conversion command |
| 3 | How do I get it on screen? | Place the node / component / element, assign the `.ssab`, play |
| 4 | Where do I read next? | Getting started, basic playback, and what this Player cannot do |

**The order is part of the convention.** Whoever is reading above question 1 has not decided to use this Player yet, so the one thing written for them comes first: a short list of why someone in this host would animate with SpriteStudio. Each item is a gain they can recognise in their host's terms — a node in the scene, a widget, one more `show`, a crossfade instead of a cut — and how the Player works earns a clause only as the reason for a gain, as a SIMD core is why it stays light on mobile. Concept explanations, architecture diagrams and checklists of supported parts serve a reader who has already decided; they go behind the doors (§4), not ahead of question 1.

**Question 4 is three kinds of link, not a link budget.** Getting started, basic playback, and the Player's own Limitations & Scope — the last because a reader deciding whether to adopt asks what it cannot do before they ask anything else, and because every Player publishes that page. Past those three, a link earns its place by answering a question a *reader of the README* has: how do I troubleshoot this, how do I migrate from the old Player. **A list that reproduces the documentation site's navigation has moved the nav rather than summarised it**, and the reader who needed the nav was already going to the site.

**Past the first frame, a README links rather than restates**: using the Player is the documentation site's, and each repository's AGENTS.md table says so. This holds for every README a user meets — a crate's, an npm or pub.dev package's — each its own front door: what it is, how to install it, the smallest thing that runs, and a closing *Read next* line carrying question 4's links.

When the distribution route is not the host's usual one, **put one sentence of reason right next to that line.** "Package Manager's Add package from URL does not work for this package, since the native binaries ship only in the tarball" is a good example. A detour with no reason beside it is indistinguishable from carelessness.

## 2. How they are answered belongs to the host

Players come in two shapes.

- **Editor-first** — install the add-on or package, drag the `.sspj` onto the editor, place a node or component and assign the `.ssab`. **Not one line of code.**
- **Code-first** — install the package, write the smallest thing that runs.

**Say on the first screen which shape this Player is.** A Hello World snippet in an editor-first Player is unnatural for that host. And "you start without writing a script" is a selling point, not something to leave implicit in a numbered list — it is the first thing a reader arriving from that engine wants to know.

### 2.1 What each step names

**Each step names what the reader sees and types**: the archive's file names, menu and command labels as the host shows them (a Japanese README uses the labels the host shows in Japanese), and the host's most natural way to put a character on screen — Ren'Py's `show`, MZ's plugin command, a node or component placed in the editor. **Steps run in the order the reader runs them and use nothing before introducing it**; a manifest entry or a dev dependency the steps need is in the README, not only in the docs. Where the Player converts on its own — an importer, a test play, a launch — that is question 2's answer, and a converter the reader fetches is the alternative beside it.

### 2.2 Placement is one tree

**The Quick Start shows one tree**: where the Player itself goes, if the reader places files (a few real entries and `…`); where the SpriteStudio project goes, if the Player converts it (the `.sspj`, an image and `…`); and where the converted `.ssab` lands. The step that puts a character on screen uses that same path, and the tree and the steps use `Ringo` throughout (§3.1).

An example does not stand in for the tree: converted assets are not committed (§3.3), so no example on GitHub shows the placed layout. **The tree speaks for itself** — it carries no notes about what readers get wrong; it is how a reader checks their own layout.

## 3. A first frame with nothing of the reader's own

**A reader must reach a first frame without supplying a `.sspj` of their own.** A Player that does not manage this is asking them to author before they can try. **In either shape (§2), the README's own steps, run on SpriteStudio's [official sample data](https://www.webtech.co.jp/help/ja/spritestudio7/download/sample/), are that first frame**: they convert the sample by the route the Player gives a user — the host's import, a test play, a launch, a conversion command — and put it on screen as §2.1 describes. The README names that data beside the steps and sends no one to a sample project, which would put a clone behind the first door (§4.1).

The repository's `examples/` and any live demo still have their readers. A demo shows the Player working before anything is installed, and an example is how someone working on the Player checks it. Opening an example from a checkout is behind the second door — the README's developer section or CONTRIBUTING.md (§3.3).

### 3.1 One character, and the same one wherever a Player can manage it

The SDK's `tests/` holds six projects, and they are not the same kind of thing. **`Ringo` is the first frame; `overall` is what you verify against.** The same picture appearing in every Player is how someone working across several of them confirms they have arrived, so a Player that keeps a first-frame project separate from its verification project points it at `Ringo` — Godot's `examples/Ringo` and Unity's `examples/SampleProject` are it.

**A Player that ships one example project rather than two has it doing both jobs, and `overall` is the right content for that** — a feature tour with a single character is not a feature tour. What it owes the reader instead is that **opening it puts one character on screen straight away**, not a menu, a file picker or a blank canvas. That is the property `Ringo` was standing in for; where the project count makes the picture impossible, the property still holds.

`overall` is the verification project either way, and never the thing a first-time reader is sent to on its own.

### 3.2 Keep the development project and the verification project apart

The file that records where converted assets came from — Godot's `.ssplayer_sources.cfg`, Unity's `SpriteStudioSourceMap.json` — **records everything that project ever converted.** Let one project serve as both the developer's scratch pad and the sample, and it fills with that developer's local absolute paths and stops being committable.

This is not held in place by discipline. It follows from whether the projects are separate.

| | SDK paths | local paths | committed |
|---|---|---|---|
| Godot `examples/Ringo` and the three other verification projects | 3–21 | 0 | yes |
| Godot `examples/dev_module` (development) | 0 | 51 | no |
| Unity `examples/UnityTestProject` (was both) | 0 | 95 | could not be |

A separated project's file stays clean on its own.

### 3.3 Converted assets are not committed

**No Player commits its samples' converted assets** — the `.ssab` and the textures copied beside it. They are build outputs of the SDK's `tests/`, and a committed copy goes stale the moment those projects or the schema change. The reader still brings nothing of their own: the source is the `.sspj` in the SDK submodule, and what stands between a clone and the first frame is converting it.

- **The host's own import, where it has one** — Godot's SS Importer dock, Unity's SS Importer window — converts with the `libssconverter` the Player already carries, so it needs nothing the Player did not.
- **A `deploy-examples` script, in every Player,** converts every sample without the editor: for a test suite, for CI, for a reader without the host's editor at hand.

The steps that open a sample — in a README's developer section or in CONTRIBUTING.md, never in the Quick Start — name the route and what it needs. Where the sample's scenes hold references an import cannot restore — Unity's, by the GUID in a committed `.meta` — they also say when it has to run.

## 4. Two doors, and what may not be behind the first one

Past the four questions there are two readers, not three: someone **using** the Player, and someone
**working on** it. The boundary is a question the reader can answer about themselves — *can I do
this with what I installed?*

Events, per-part overrides, textures, attaching host content to a part: all of these are the first
reader, further in. They are not a third audience, and the docs already say so — six of the seven
Players shelve them next to basic playback rather than apart from it.

**Put the two doors at the top of the Quick Start, in two lines.** Not a table, and not before the
Quick Start: a reader who has to finish a routing table before reaching question 1 has been made to
read something in order to find out that it was not for them.

```
**Using it in your app?** Read on — install, convert, play. Everything after that
(events, per-part overrides, textures) is the same path further in: <link>.
**Working on the Player, or the runtime under it?** <link to CONTRIBUTING and the Build Guide>.
```

### 4.1 Nothing behind the first door may need a clone

**A clone, a submodule or a toolchain inside the "using it" path is the defect this section
exists to prevent.** The moment a reader who wanted a package meets `git clone --recursive`, they
have to decide whether they are still the intended audience — and the answer the page wanted to give
them was yes.

### 4.2 The host axis sits underneath this one

**Host** — Rust / C++ / browser — belongs inside the install step of a Player serving several, as
wgpu has it. It is useful, but it does not tell a reader whether the page is for them, which is what
the two doors are for.

A Player whose documentation genuinely has a third shelf — SSPlayerForWeb's `customize/` — names it
from the first door's line rather than growing a row for it.

### 4.3 Behind the first door, the same reader

**The guide a README's *Read next* lands on is written for the reader who came through the first
door**, so §2.1 holds past the first screen. Every label is the host's — argument names, options and
menus too — read from the host rather than recalled, with an internal id only in parentheses. Where
the Player converts on its own, the guide leads with that route and puts a converter the reader
fetches after it; a guide whose route differs from its README's gives question 2 two answers.

**The guide reaches the reader's path before any word about how the Player is built.** What the
reader can observe of it — that animations keep the host's draw order, pause with the game, need no
launch options — follows the path, in the host's terms. Renderer classes, registration hooks, bundle
formats and bindings are behind the second door, on the shelf §4.4 describes.

**A reason about how the Player is built moves behind the second door; it does not leave the
documentation.** What polling would cost, why a toast and not a dialog, why a mobile library is
embedded by hand, how an importer compares with a sibling's: each moves to the developer shelf
(§4.4) in the change that takes it off the guide, together with any pointer that sent a developer to
the guide for it. A reason that changes what the reader does stays beside the step, as §1 and §7
already ask.

**A step says what goes in, not what to keep out.** "It is the only download you need" does the work
of "do not also install the ZIP" without assuming the reader will get it wrong — the rule §2.2 sets
for the tree, carried into the prose.

**Text that ships inside the Player is the guide's twin**: a plugin's help, the readme in an
archive, a registry package's README. It changes in the same change as the guide.

### 4.4 The second door has a shelf

**Every Player keeps a developer shelf on its documentation site, in both languages, reached from
the README's developer section.** It is for whoever works on the Player or changes it for a project
of their own — the same reader (§4) — and it may ask more of them than the guide asks of a user: a
successor should be able to pick the Player up from it. It holds four things:

1. **How the Player is put together** — its layers and parts, and what each is responsible for:
   what drives the runtime, what draws, what converts, where the host hooks in.
2. **Using a changed Player** — how a checkout gets into an app or a project, since the first door
   gives only the released package.
3. **Checking against the real host** — how to run the host the Player plugs into, and where the
   host's labels are read from, so the guide's labels are checked rather than recalled (§4.3).
4. **Why it is built the way it is** — the reasons that span files.

**Every Player shelves them in the same place.** The documentation's `development/` folder holds
two pages: `build.md`, the Build Guide — building from a checkout, then parts 2 and 3 — and
`architecture.md`, How the Player Is Put Together — parts 1 and 4. They sit in a navigation section
named For Player & SDK Developers (Player・SDK 開発者向けガイド), in that order; a page only one
Player needs comes after them in the same folder.

**The shelf explains; it does not index.** Files and functions are renamed with every refactor, and
nothing checks a page that names them, so the shelf names a part by what it does. It names a file or
a function only where a step cannot be followed without it — a script, a command, a public type, a
setting the host shows, the host's own API — and a detail that changes with one function lives in
that function's comment, which changes with it. The shelf holds what no single file can: the route,
the procedure, the reason across files. A commit message is not a shelf — a successor reads the
Player, not its history. What every Player shares (playback, blending, audio, part tracking) is the
porting guide's, once, and the shelf links it.

## 5. The reference Player

**SSPlayerForWgpu is the Player the other six are read against.** When two Players disagree about how a `.ssab` draws or how playback behaves, it is the one you check. A feature the whole family will carry is settled there first and ported from there — the track stack behind motion blending is the precedent: wgpu surfaced it, and the other six port it against its `tests/tracks.rs`.

**The order of authority is the porting guide, then wgpu, then the rest.** The guide is the specification and wgpu its executable reading. Where the two disagree, one of them has a defect: fix whichever is wrong, and never let the code quietly overrule the page. Where the guide is silent, wgpu's behaviour decides, and the rule it followed goes into the guide in the same change, so the next Player ports a sentence rather than a diff.

It is this one because runtime changes reach it first — it links `ssruntime` and `ssab` as Rust crates through a submodule, so a change is exercised here before any C-API build picks it up — and because it holds no engine: adapters are separate crates, so what `ssplayer-wgpu` does is what the runtime and the guide say rather than what a host needed.

**It does not exercise the runtime's C ABI or WASM face.** Alone among the seven it calls `ssruntime` as a crate, so a defect in the C-API or WASM layer — an export that records no reason, a failure reported under the wrong code — never reaches it. The other six call through those layers and keep carrying their conformance. Reading it against the C-API is mechanical: its calls on a `Context` are the C functions without the `ss_runtime_` prefix (`setup_animation` is `ss_runtime_setup_animation`), apart from a handful shaped differently, such as a slice where C hands out one element at a time.

This is an arrangement among maintainers. User-facing documentation lists the Players as equals and does not call this one the reference.

## 6. The roadmaps

**Every official Player's `ROADMAP.md` has the same shape, so a reader moving between them finds each section where they expect it.** The title is `# SpriteStudio Player for <Engine> — Roadmap`, and the file holds unfinished work only. The SDK's own `ROADMAP.md` shares the title and the item syntax; its sections are its own.

The sections, in this order, each omitted when empty: `Playback & rendering`, `UI integration`, a section the host genuinely needs, `Distribution & verification`, `Needs SDK`, `Backlog`, `Deliberate non-goals`. **Status is the section, never a symbol.** `Needs SDK` is for work that waits on an SDK item and names it (`Waits for: …`); work blocked on anything else stays in its own section and ends `Blocked on: …`.

An item is `- [ ] **Title** — one or two sentences: what is missing and, if it is not obvious, why it is not done.` Non-goals are plain bullets. **A finished item is deleted, not ticked**: the page that documents the behaviour owns the description from then on. No dates, no history, no source line numbers, no measurement logs.

Link the SDK's roadmap as `https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/ROADMAP.md` — `HEAD` follows the default branch. A heading that other files link to keeps its anchor; search for it before renaming.

## 7. Departing from this document

The same as [Player API Conventions §10](./libs/ssruntime/docs/20_design/30_api_conventions.md#10-departing-from-this-document). Depart when the host's conventions genuinely collide with this one. **Write the reason where the departure is.**
