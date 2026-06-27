# DeckMaster Package Format (`.deckpkg`) — v1

Status: **Canonical**. This document is the source of truth for the
DeckMaster data model. `deckmaster-core`, the editor, the CLI, and any LLM
authoring skill must all conform to this spec. Where code and this doc
disagree, the doc wins until the doc is updated.

---

## 1. Goals

1. One canonical on-disk representation for a presentation: a package, not
   a single bloated JSON file.
2. `deck.json` stays small and human/LLM-readable — it never embeds image
   bytes.
3. Images are real files in `assets/`, referenced by id.
4. The package is just a zip. No custom binary format, no surprises.
5. Unzip it and you get a plain folder you can `cat`, `grep`, diff in git,
   or hand to an LLM.

## 2. What changes from the old model

The old model allowed `ImageElement.src` to be either a `data:` URL or a
relative path, decided ad hoc per import mode. That is **deprecated as a
canonical form** as of this spec.

| | Old (deprecated as canonical) | New (canonical) |
|---|---|---|
| Image storage | inline base64 in `src` | real file in `assets/` |
| Image reference | `data:image/png;base64,...` or loose relative path | `asset_id` (UUID), resolved via `assets[]` |
| Top-level artifact | a single `.json` file | a `.deckpkg` zip containing `deck.json` + `assets/` |
| Embedded data URLs | the default | a one-way **export convenience**, never read back in as canonical |

Embedded-data-URL JSON is not deleted from the system. It becomes an
**export target** (`deckmaster export embedded-json`) for cases like
pasting a whole deck into a single LLM message. It is never again the
thing `deckmaster-core` treats as the source of truth, and the importer
does not need to accept it going forward.

## 3. Package layout

```
mydeck.deckpkg                 (a zip file, this exact layout inside)
├── deck.json                  required, canonical document
└── assets/
    ├── 3f9a1c20.png
    ├── 7bb44e10.jpeg
    └── ...
```

Rules:

- `deck.json` MUST be at the zip root, named exactly `deck.json`.
- All asset files MUST live directly under `assets/` (no subfolders in v1).
- Asset file names MUST be `{asset_id}.{ext}` where `asset_id` matches the
  `id` field of the corresponding entry in `deck.json`'s `assets[]` array,
  and `ext` is one of: `png`, `jpeg`, `gif`, `webp`, `bmp`.
- A `.deckpkg` with no `assets/` directory is valid (text-only deck).
- Extra files in the zip (e.g. a future `thumbnail.png`) MUST be ignored
  by readers, not rejected — this keeps the format forward-extensible.

File extension: `.deckpkg`. MIME type (for future hosted use):
`application/vnd.deckmaster.package+zip`.

## 4. `deck.json` schema

This is additive to the existing `deckmaster-core` model, with one
breaking change: `ImageElement.src` → `ImageElement.asset_id`.

```jsonc
{
  "id": "uuid",
  "metadata": {
    "title": "string",
    "author": "string | null"
  },
  "theme": {
    "name": "string",
    "background": { "value": "#RRGGBB" },
    "foreground": { "value": "#RRGGBB" }
  },
  "assets": [
    {
      "id": "uuid",            // matches assets/{id}.{ext} in the package
      "media_type": "image/png", // canonical MIME type
      "alt": "string | null"     // default alt text for this asset
    }
  ],
  "slides": [
    {
      "id": "uuid",
      "name": "string | null",
      "size": { "width": 960.0, "height": 540.0 },   // points, 96pt = 1in
      "elements": [
        {
          "type": "Text",
          "id": "uuid",
          "bounds": { "x": 0.0, "y": 0.0, "width": 0.0, "height": 0.0 },
          "text": "string",
          "font_size": 24.0,
          "color": { "value": "#RRGGBB" }
        },
        {
          "type": "Image",
          "id": "uuid",
          "bounds": { "x": 0.0, "y": 0.0, "width": 0.0, "height": 0.0 },
          "asset_id": "uuid",       // REQUIRED, must exist in top-level assets[]
          "alt": "string | null"    // overrides assets[].alt if present
        },
        { "type": "Shape", "...": "unchanged" },
        { "type": "Table", "...": "unchanged" },
        { "type": "Chart", "...": "unchanged" }
      ]
    }
  ]
}
```

### Field notes

- **Units**: all `bounds`, `size`, `font_size` stay in points (pt), as
  today. 1pt = 1/72in. Canonical widescreen slide = 960×540pt (13.333×7.5in).
- **Colors**: `#RRGGBB` hex string, always 6 digits, uppercase preferred
  but readers must accept lowercase.
- **`asset_id` referential integrity**: every `ImageElement.asset_id` MUST
  correspond to an entry in the top-level `assets[]` array, which in turn
  MUST correspond to a file in `assets/` inside the package. A `deck.json`
  that references a missing asset is invalid (see §6 Validation).
- **No `data:` URLs in `asset_id`.** If a reader sees something that looks
  like a data URL where `asset_id` is expected, that's a malformed/legacy
  document, not a variant to support.
- **Unused assets** (declared in `assets[]`, file present, but no element
  references them) are legal — decks may keep spare/library assets. Not
  an error, just worth surfacing in `validate` output as a note.

## 5. Conversions

```
                 deckmaster pack / unpack
   deck.json + assets/  <───────────────>  mydeck.deckpkg

                 deckmaster import-pptx
   some.pptx  ───────────────────────────>  mydeck.deckpkg

                 deckmaster export-pptx
   mydeck.deckpkg  ───────────────────────> output.pptx

                 deckmaster export embedded-json   (convenience, one-way)
   mydeck.deckpkg  ───────────────────────> fat-deck.json (data: URLs inline)
```

`embedded-json` export is **lossy in the reverse direction only** in the
sense that it's not re-imported — it exists purely so a whole deck can be
pasted into a single LLM chat message or stored as one file when zip
handling isn't available. The CLI does not accept it as import input.

## 6. Validation

`deckmaster validate <path.deckpkg>` checks, in order:

1. Zip opens and contains `deck.json` at root.
2. `deck.json` parses as valid JSON matching this schema.
3. Every slide has a valid `size` (width > 0, height > 0).
4. Every element has non-negative `bounds.width` / `bounds.height`.
5. Every `ImageElement.asset_id` exists in `assets[]`.
6. Every `assets[]` entry has a corresponding file at
   `assets/{id}.{ext}` inside the package, where `ext` matches
   `media_type`.
7. (Informational, not a failure) any `assets[]` entry with zero
   referencing elements is reported as unused.

This is intentionally a flat, fast checklist — not a general-purpose
schema validator framework. Extend the list before reaching for a
dependency.

## 7. Why this is the right base for LLM authoring

An LLM authoring a deck only ever needs to:

1. Write `deck.json` (pure text, fits in context).
2. Either supply asset files it has, or write a deck with zero images.
3. Hand both to `deckmaster pack` to get a `.deckpkg`.

It never needs to reason about base64, data URL prefixes, or where binary
bytes live inside a JSON string. That separation is the entire point of
this spec.

## 8. Versioning

This is v1. If the schema needs a breaking change later, add a top-level
`"deckpkg_version": 1` field (absent = 1, for backward compatibility with
decks written under this spec) and bump on actual breakage — not for
additive fields like new element types.
