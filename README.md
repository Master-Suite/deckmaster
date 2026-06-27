<!-- LOGO -->

<p align="center">
  <img height="280" alt="deckmaster-logo" src="https://github.com/user-attachments/assets/1037c197-df86-4b0c-a46e-4ecb53fd5fe7" />
</p>

<p align="center">
  <img alt="Local First" src="https://img.shields.io/badge/local--first-yes-2ea44f?style=flat-square">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-core--engine-f97316?style=flat-square">
  <img alt="PPTX" src="https://img.shields.io/badge/PPTX-import%20%2F%20export-0969da?style=flat-square">
  <img alt="AI Skill Cold Tested" src="https://img.shields.io/badge/AI%20skill-cold--tested-8250df?style=flat-square">
  <img alt="Ollama" src="https://img.shields.io/badge/Ollama-E2E%20passing-2ea44f?style=flat-square">
  <img alt="Qwen2.5 Coder" src="https://img.shields.io/badge/Qwen2.5--Coder-validated-0969da?style=flat-square">
</p>


**DeckMaster** is an open-source, local-first presentation engine written in Rust.

It gives slide decks a canonical object model that can move between PPTX, JSON, CLI operations, and AI agents without making the zipped PPTX package the source of truth.


<p align="center">
  <img width="1535" alt="Deck" src="https://github.com/user-attachments/assets/c00dc708-f523-4b46-a75d-9738e4912bc3" />
</p>

---

## Comparison With Existing Presentation Systems


**Strong** := the system is naturally good at the dimension.
**Partial** := the capability exists, but with tradeoffs or limits.
**Weak** := the capability is missing, indirect, proprietary, or structurally difficult.

| Dimension                | **DeckMaster**                                                                          | PowerPoint / PPTX                                                                     | Google Slides                                                                           | Beamer                                                                                                            | Slidev / Marp                                                                           | Reveal / HTML decks                                                                     | Canva / Figma                                                                           |
| ------------------------ | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| **Source of truth**      | **Canonical presentation model**                                                        | Package/XML                                                                           | Cloud object model                                                                      | TeX source                                                                                                        | Markdown source                                                                         | DOM/CSS/source                                                                          | Proprietary canvas                                                                      |
| **AI friendliness**      | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">     | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Strong for text and math" src="https://img.shields.io/badge/Strong_text_math-2ea44f?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> |
| **Visual editing**[^1]   | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">                                 | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   |
| **PPTX compatibility**   | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square"> | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">                                 | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> |
| **Git friendliness**     | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">     | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">                             | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       |
| **Round-trip potential** | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square">                           | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       |
| **Local-first**          | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square"> | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">                             | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       |
| **Canonical model**      | <img alt="Strong" src="https://img.shields.io/badge/Strong-2ea44f?style=flat-square">   | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">     | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square">                           | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Partial" src="https://img.shields.io/badge/Partial-d29922?style=flat-square"> | <img alt="Weak" src="https://img.shields.io/badge/Weak-6e7781?style=flat-square">       |

[^1]: A visual editor for DeckMaster exists, but it is a separate, closed-source product — **DeckMaster Pro** — and is not part of this repository. See [Editor](#editor) below.

DeckMaster is not trying to beat every presentation system at its own game.

It is worse than PowerPoint at full Office fidelity.
It is worse than Google Slides at live collaboration.
It is worse than Beamer at math-heavy academic PDFs.
It is worse than Canva at design polish.

DeckMaster is built around a different question:

> Can a CLI, an AI agent, a source file, and a PPTX adapter all edit the same presentation without one representation becoming the master?

DeckMaster's answer is:

```text
Everything goes through the model.
```


---

## AI-Conscious Presentation Engineering

DeckMaster is designed for an AI-conscious world.

An AI agent should not have to directly edit a zipped PPTX package full of XML relationships, EMU coordinates, theme parts, slide layouts, and content type declarations.

Instead of asking an AI to patch this:

```xml
<a:off x="1270000" y="2540000"/>
<a:ext cx="6350000" cy="1016000"/>
```

DeckMaster lets an AI or CLI operate on this:

```json
{
  "type": "Text",
  "bounds": {
    "x": 100.0,
    "y": 200.0,
    "width": 500.0,
    "height": 80.0
  },
  "text": "Revenue increased 24%",
  "font_size": 32.0,
  "color": {
    "value": "#111111"
  }
}
```

That is the core idea.

DeckMaster gives AI systems a small, typed, editable model of a presentation. It is:

* structured
* local-first
* diffable
* serializable
* testable
* round-trippable
* operation-friendly
* independent of any single editor or file format

---

## Current Status

DeckMaster is an early but working prototype.

It can now round-trip presentation decks with text and images through a canonical Rust model, using a single canonical package format: `deck.json` plus an `assets/` folder, where images are referenced by `asset_id` and never stored inline. Images are never embedded as base64 inside `deck.json` — a separate one-way export can inline them as `data:image/...` URLs for pasting a whole deck into a chat, but that form is never read back in.

Implemented:

* Canonical presentation model, with a top-level `assets[]` registry
* `ImageElement.asset_id` (not a path, not a data URL)
* JSON serialization and deserialization
* JSON round-trip tests
* A real on-disk package format: `deck.json` + `assets/<asset_id>.<ext>`, zipped
* `validate()` — checks for dangling asset references, missing asset bytes, malformed bounds, empty decks
* PPTX package reader
* PPTX relationship parsing
* PPTX slide discovery
* PPTX text extraction
* PPTX image extraction (resolved directly into the asset registry)
* PPTX → Presentation import
* Presentation → PPTX export (refuses to write a partial file if validation fails)
* Multi-slide PPTX export
* Text bounds import/export
* Text font size import/export
* Text color import/export
* Image bounds import/export
* Image binary import/export
* One-way embedded-JSON export (images inlined as data URLs, for pasting into a chat — not a canonical form)
* CLI import/export/pack/unpack/validate
* CLI model inspection
* CLI element editing operations
* Core model editing operations

Validated:

```text
PPTX
 ↓
DeckMaster Model
 ↓
PPTX
```

And for the canonical package format:

```text
PPTX
 ↓
deck.json + assets/
 ↓
PPTX
 ↓
DeckMaster Model
```

The test suite now covers:

* generated text/image round-trips
* multiple images on one slide
* images across multiple slides
* a real on-disk package round-trip (not just an in-memory model)
* invalid/dangling/missing asset handling — export refuses to silently produce a broken file
* real-world PPTX smoke tests against an independently-generated fixture

Exported presentations open successfully in Google Slides.

---

## Canonical Model

```text
Presentation
├── Metadata
├── Theme
├── Assets
└── Slides
    └── Elements
        ├── Text
        ├── Image
        ├── Shape
        ├── Table
        └── Chart
```

The model is intentionally simple, stable, and extensible.

Current text model includes:

```text
TextElement
├── id
├── bounds
│   ├── x
│   ├── y
│   ├── width
│   └── height
├── text
├── font_size
└── color
```

Canonical slide geometry uses points.

A widescreen slide is represented as:

```json
{
  "width": 960.0,
  "height": 540.0
}
```

PPTX-specific units such as EMUs are handled inside the PPTX adapter, not exposed as the canonical model.

---

## CLI Examples

All commands below run through `cargo run -p deckmaster-cli --`. If you've
built a release binary (`cargo build --release -p deckmaster-cli`), you
can call `target/release/deckmaster-cli` directly instead.

Most commands — `inspect`, `validate`, `new`, `unpack`, `add-slide`,
`add-text`, `move-element`, `resize-element`, `update-text`,
`export-pptx`, `export-embedded-json` — operate directly on a `.deckpkg`
file. `pack` is the one command that goes the other way: it takes a
loose `deck.json` (the form you'd hand-write or get from an AI agent)
and produces a `.deckpkg`.

Create a new, empty deck:

```bash
cargo run -p deckmaster-cli -- new hello.deckpkg "My First Deck"
```

Inspect a deck:

```bash
cargo run -p deckmaster-cli -- inspect hello.deckpkg
```

Validate a deck (checks for dangling asset references, missing asset bytes, malformed bounds, empty decks):

```bash
cargo run -p deckmaster-cli -- validate hello.deckpkg
```

Add a slide, and add text to a slide (slide numbers are 1-indexed):

```bash
cargo run -p deckmaster-cli -- add-slide hello.deckpkg "Second Slide"
cargo run -p deckmaster-cli -- add-text hello.deckpkg 2 "Some body text"
```

Move or resize an element, or update its text (these need a slide ID and element ID, both visible via `inspect`):

```bash
cargo run -p deckmaster-cli -- move-element hello.deckpkg <SLIDE_ID> <ELEMENT_ID> 300 220
cargo run -p deckmaster-cli -- resize-element hello.deckpkg <SLIDE_ID> <ELEMENT_ID> 640 90
cargo run -p deckmaster-cli -- update-text hello.deckpkg <SLIDE_ID> <ELEMENT_ID> "Updated text"
```

Pack a hand-written (or AI-authored) `deck.json` into a `.deckpkg`. By
default, image assets are read from a sibling `assets/` directory next
to `deck.json`; pass `--assets-dir` to use a different location.
`pack` runs validation first and refuses to write a broken package:

```bash
cargo run -p deckmaster-cli -- pack deck.json hello.deckpkg
cargo run -p deckmaster-cli -- pack deck.json hello.deckpkg --assets-dir ./my-assets
```

Unpack a `.deckpkg` back into a loose `deck.json` + `assets/` directory
(defaults to a folder named after the package):

```bash
cargo run -p deckmaster-cli -- unpack hello.deckpkg
cargo run -p deckmaster-cli -- unpack hello.deckpkg --out-dir ./out
```

Import a real PPTX into a DeckMaster package:

```bash
cargo run -p deckmaster-cli -- import-pptx input.pptx output.deckpkg
```

Export a DeckMaster package to PPTX:

```bash
cargo run -p deckmaster-cli -- export-pptx hello.deckpkg hello.pptx
```

Export to a single JSON file with images inlined as `data:` URLs — a one-way convenience for pasting a whole deck into a chat message, not meant to be read back in:

```bash
cargo run -p deckmaster-cli -- export-embedded-json hello.deckpkg hello.embedded.json
```

---

## Editor

A visual editor for DeckMaster exists and is actively developed, but it is a separate, closed-source product — **DeckMaster Pro** — and its code is not part of this repository.

The engine, CLI, and AI skill in this repo are fully usable on their own without the editor: you can author a `deck.json` by hand or with an AI agent using the [skill](#ai-skill), then import, inspect, edit, and export it entirely through `deckmaster-cli`. The editor is a convenience layer on top of the same canonical model, not a requirement for using DeckMaster.

The editor does not own the document. It edits the DeckMaster model, the same model the CLI and AI skill operate on.

---

## Development Commands

Run Rust tests:

```bash
cargo test
```

---

## AI Skill

DeckMaster includes an AI-facing authoring skill at:

```text
skills/deckmaster-deck/
````

The skill teaches a model how to produce valid DeckMaster `deck.json` files directly. It includes examples, evals, a standalone validator, and an Ollama cold-test harness.

See [`skills/deckmaster-deck/README.md`](skills/deckmaster-deck/README.md) for usage and testing.

---

## Current Target

DeckMaster's current target is a simple real-deck loop:

* import a simple PPTX with text and images
* edit it via the CLI or an AI agent using the skill
* move text and images
* resize text and images
* edit text content
* add new text
* add new slides
* save back to PPTX
* reopen successfully in Google Slides or PowerPoint
* preserve the canonical model as the source of truth

That is the real v1 target.

Not full Office compatibility.
Not animations.
Not collaboration.

Just simple real decks round-tripping through the model.

---

## License

Licensed under the Apache License, Version 2.0.
See [LICENSE](LICENSE) for details.
