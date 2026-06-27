//! Integration tests for the deckmaster-cli binary itself: argument
//! parsing, wiring into deckmaster-core/deckmaster-pptx, exit codes, and
//! user-facing messages. These spawn the actual compiled binary via
//! `env!("CARGO_BIN_EXE_deckmaster-cli")` rather than calling library
//! functions directly -- that's the whole point, since the gap this
//! suite closes is "does the CLI surface itself work," not "does the
//! underlying library work" (deckmaster-core and deckmaster-pptx already
//! have their own test suites for that).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use deckmaster_core::{DeckPackage, Element};

fn cli_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_deckmaster-cli"))
}

fn run(args: &[&str]) -> Output {
    Command::new(cli_path())
        .args(args)
        .output()
        .expect("failed to spawn deckmaster-cli")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn tempdir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("deckmaster-cli-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Pull the first slide's id and its first element's id out of a real
/// .deckpkg on disk, by reading the package directly rather than
/// scraping `inspect`'s text output -- scraping stdout for IDs would
/// make these tests brittle against harmless formatting changes to
/// `inspect`, which isn't what this suite is about.
fn first_slide_and_element_id(deckpkg: &Path) -> (String, String) {
    let package = DeckPackage::open(deckpkg).expect("open package for id lookup");
    let slide = &package.presentation.slides[0];
    let element_id = slide.elements[0].id();

    (slide.id.to_string(), element_id.to_string())
}

fn minimal_png_bytes() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0xCD, 0x37, 0x42, 0x49, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

// ---------------------------------------------------------------------------
// new / inspect / validate
// ---------------------------------------------------------------------------

#[test]
fn new_creates_an_openable_deckpkg_with_a_default_slide() {
    let dir = tempdir();
    let file = dir.join("hello.deckpkg");

    let output = run(&[
        "new",
        file.to_str().unwrap(),
        "My First Deck",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Presentation created"));
    assert!(file.exists());

    let package = DeckPackage::open(&file).expect("new package must be openable");
    assert_eq!(package.presentation.metadata.title, "My First Deck");
    assert_eq!(package.presentation.slides.len(), 1);

    let Element::Text(text) = &package.presentation.slides[0].elements[0] else {
        panic!("expected the default slide's element to be Text");
    };
    assert_eq!(text.text, "Welcome to DeckMaster");
}

#[test]
fn inspect_reports_title_slide_count_and_text_content() {
    let dir = tempdir();
    let file = dir.join("hello.deckpkg");
    run(&["new", file.to_str().unwrap(), "Inspect Me"]);

    let output = run(&["inspect", file.to_str().unwrap()]);
    let text = stdout(&output);

    assert!(output.status.success());
    assert!(text.contains("Title: Inspect Me"));
    assert!(text.contains("Slides: 1"));
    assert!(text.contains("Welcome to DeckMaster"));
}

#[test]
fn inspect_on_a_nonexistent_file_fails_cleanly_not_a_panic() {
    let output = run(&["inspect", "/no/such/file.deckpkg"]);

    assert!(!output.status.success());
    assert!(stderr(&output).starts_with("error:"));
}

#[test]
fn validate_reports_a_freshly_created_deck_as_valid() {
    let dir = tempdir();
    let file = dir.join("hello.deckpkg");
    run(&["new", file.to_str().unwrap(), "Valid Deck"]);

    let output = run(&["validate", file.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("valid"));
}

// ---------------------------------------------------------------------------
// add-slide / add-text
// ---------------------------------------------------------------------------

#[test]
fn add_slide_then_add_text_at_a_one_indexed_slide_number() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Multi Slide"]);

    let add_slide_output = run(&["add-slide", file.to_str().unwrap(), "Second Slide"]);
    assert!(add_slide_output.status.success());
    assert!(stdout(&add_slide_output).contains("Slide added"));

    // Slide numbers are 1-indexed at the CLI boundary -- "2" means the
    // second slide, not an array index.
    let add_text_output = run(&[
        "add-text",
        file.to_str().unwrap(),
        "2",
        "Some body text",
    ]);
    assert!(add_text_output.status.success());
    assert!(stdout(&add_text_output).contains("Text added"));

    let package = DeckPackage::open(&file).expect("open after adds");
    assert_eq!(package.presentation.slides.len(), 2);

    let second_slide = &package.presentation.slides[1];
    let texts: Vec<&str> = second_slide
        .elements
        .iter()
        .filter_map(|e| match e {
            Element::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(texts.contains(&"Some body text"));
}

#[test]
fn add_text_at_slide_zero_is_rejected_with_a_clear_message() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Deck"]);

    let output = run(&["add-text", file.to_str().unwrap(), "0", "irrelevant"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("slide numbers start at 1"));
}

#[test]
fn add_text_at_an_out_of_range_slide_is_rejected() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Deck"]); // exactly 1 slide

    let output = run(&["add-text", file.to_str().unwrap(), "5", "irrelevant"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("slide does not exist"));
}

// ---------------------------------------------------------------------------
// pack / unpack
// ---------------------------------------------------------------------------

fn write_minimal_deck_json(path: &Path) {
    let contents = r##"{
        "id": "11111111-1111-1111-1111-111111111111",
        "metadata": { "title": "Packed From Scratch", "author": null },
        "theme": {
            "name": "Default",
            "background": { "value": "#FFFFFF" },
            "foreground": { "value": "#111111" }
        },
        "assets": [],
        "slides": [
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "name": "Slide 1",
                "size": { "width": 960.0, "height": 540.0 },
                "elements": [
                    {
                        "type": "Text",
                        "id": "33333333-3333-3333-3333-333333333333",
                        "bounds": { "x": 100.0, "y": 100.0, "width": 600.0, "height": 80.0 },
                        "text": "Hand-authored text",
                        "font_size": 28.0,
                        "color": { "value": "#111111" }
                    }
                ]
            }
        ]
    }"##;

    fs::write(path, contents).expect("write deck.json");
}

#[test]
fn pack_a_hand_written_deck_json_with_no_images() {
    let dir = tempdir();
    let deck_json = dir.join("deck.json");
    write_minimal_deck_json(&deck_json);

    let output_pkg = dir.join("packed.deckpkg");
    let output = run(&[
        "pack",
        deck_json.to_str().unwrap(),
        output_pkg.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Packed"));

    let package = DeckPackage::open(&output_pkg).expect("open packed result");
    assert_eq!(package.presentation.metadata.title, "Packed From Scratch");
}

#[test]
fn pack_refuses_and_does_not_write_output_when_the_deck_has_no_slides() {
    let dir = tempdir();
    let deck_json = dir.join("empty.json");

    fs::write(
        &deck_json,
        r##"{
            "id": "11111111-1111-1111-1111-111111111111",
            "metadata": { "title": "Empty Deck", "author": null },
            "theme": {
                "name": "Default",
                "background": { "value": "#FFFFFF" },
                "foreground": { "value": "#111111" }
            },
            "assets": [],
            "slides": []
        }"##,
    )
    .expect("write empty deck.json");

    let output_pkg = dir.join("should_not_exist.deckpkg");
    let output = run(&[
        "pack",
        deck_json.to_str().unwrap(),
        output_pkg.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("refusing to pack"));
    assert!(
        !output_pkg.exists(),
        "pack must not write a .deckpkg when validation fails"
    );
}

#[test]
fn pack_fails_with_a_clear_message_when_a_declared_asset_file_is_missing() {
    let dir = tempdir();
    let deck_json = dir.join("deck.json");

    fs::write(
        &deck_json,
        r##"{
            "id": "11111111-1111-1111-1111-111111111111",
            "metadata": { "title": "Missing Asset Deck", "author": null },
            "theme": {
                "name": "Default",
                "background": { "value": "#FFFFFF" },
                "foreground": { "value": "#111111" }
            },
            "assets": [
                { "id": "99999999-9999-9999-9999-999999999999", "media_type": "image/png", "alt": null }
            ],
            "slides": [
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "name": "Slide 1",
                    "size": { "width": 960.0, "height": 540.0 },
                    "elements": []
                }
            ]
        }"##,
    )
    .expect("write deck.json");
    // Deliberately do NOT create an assets/ dir or the referenced file.

    let output_pkg = dir.join("should_not_exist.deckpkg");
    let output = run(&[
        "pack",
        deck_json.to_str().unwrap(),
        output_pkg.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found at"));
    assert!(!output_pkg.exists());
}

#[test]
fn pack_with_explicit_assets_dir_succeeds_when_the_real_file_is_present() {
    let dir = tempdir();
    let deck_json = dir.join("deck.json");
    let assets_dir = dir.join("my-assets");
    fs::create_dir_all(&assets_dir).expect("create assets dir");

    fs::write(
        &deck_json,
        r##"{
            "id": "11111111-1111-1111-1111-111111111111",
            "metadata": { "title": "With Image", "author": null },
            "theme": {
                "name": "Default",
                "background": { "value": "#FFFFFF" },
                "foreground": { "value": "#111111" }
            },
            "assets": [
                { "id": "99999999-9999-9999-9999-999999999999", "media_type": "image/png", "alt": null }
            ],
            "slides": [
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "name": "Slide 1",
                    "size": { "width": 960.0, "height": 540.0 },
                    "elements": [
                        {
                            "type": "Image",
                            "id": "33333333-3333-3333-3333-333333333333",
                            "bounds": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                            "asset_id": "99999999-9999-9999-9999-999999999999",
                            "alt": null
                        }
                    ]
                }
            ]
        }"##,
    )
    .expect("write deck.json");

    fs::write(
        assets_dir.join("99999999-9999-9999-9999-999999999999.png"),
        minimal_png_bytes(),
    )
    .expect("write asset file");

    let output_pkg = dir.join("with_image.deckpkg");
    let output = run(&[
        "pack",
        deck_json.to_str().unwrap(),
        output_pkg.to_str().unwrap(),
        "--assets-dir",
        assets_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let package = DeckPackage::open(&output_pkg).expect("open packed result");
    assert_eq!(package.asset_bytes.len(), 1);
}

#[test]
fn unpack_writes_deck_json_to_an_explicit_out_dir() {
    let dir = tempdir();
    let file = dir.join("hello.deckpkg");
    run(&["new", file.to_str().unwrap(), "Unpack Me"]);

    let out_dir = dir.join("unpacked");
    let output = run(&[
        "unpack",
        file.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Unpacked to"));

    let deck_json_path = out_dir.join("deck.json");
    assert!(deck_json_path.exists());

    let raw = fs::read_to_string(&deck_json_path).expect("read unpacked deck.json");
    assert!(raw.contains("Unpack Me"));
}

#[test]
fn pack_and_unpack_round_trip_through_each_other() {
    let dir = tempdir();
    let deck_json = dir.join("deck.json");
    write_minimal_deck_json(&deck_json);

    let packed = dir.join("packed.deckpkg");
    let pack_output = run(&[
        "pack",
        deck_json.to_str().unwrap(),
        packed.to_str().unwrap(),
    ]);
    assert!(pack_output.status.success());

    let unpacked_dir = dir.join("unpacked");
    let unpack_output = run(&[
        "unpack",
        packed.to_str().unwrap(),
        "--out-dir",
        unpacked_dir.to_str().unwrap(),
    ]);
    assert!(unpack_output.status.success());

    let repacked = dir.join("repacked.deckpkg");
    let repack_output = run(&[
        "pack",
        unpacked_dir.join("deck.json").to_str().unwrap(),
        repacked.to_str().unwrap(),
    ]);
    assert!(repack_output.status.success(), "stderr: {}", stderr(&repack_output));

    let original = DeckPackage::open(&packed).unwrap();
    let round_tripped = DeckPackage::open(&repacked).unwrap();
    assert_eq!(
        original.presentation.metadata.title,
        round_tripped.presentation.metadata.title
    );
    assert_eq!(
        original.presentation.slides.len(),
        round_tripped.presentation.slides.len()
    );
}

// ---------------------------------------------------------------------------
// import-pptx / export-pptx, round-tripped through the CLI only
// ---------------------------------------------------------------------------

#[test]
fn export_pptx_then_import_pptx_preserves_the_default_text() {
    let dir = tempdir();
    let deckpkg = dir.join("source.deckpkg");
    run(&["new", deckpkg.to_str().unwrap(), "Round Trip Deck"]);

    let pptx = dir.join("exported.pptx");
    let export_output = run(&[
        "export-pptx",
        deckpkg.to_str().unwrap(),
        pptx.to_str().unwrap(),
    ]);
    assert!(export_output.status.success(), "stderr: {}", stderr(&export_output));
    assert!(stdout(&export_output).contains("PPTX exported"));
    assert!(pptx.exists());

    let reimported = dir.join("reimported.deckpkg");
    let import_output = run(&[
        "import-pptx",
        pptx.to_str().unwrap(),
        reimported.to_str().unwrap(),
    ]);
    assert!(import_output.status.success(), "stderr: {}", stderr(&import_output));
    assert!(stdout(&import_output).contains("PPTX imported"));

    let package = DeckPackage::open(&reimported).expect("open reimported package");
    let has_welcome_text = package
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .any(|element| matches!(element, Element::Text(text) if text.text == "Welcome to DeckMaster"));

    assert!(
        has_welcome_text,
        "the original slide's text must survive a CLI-only export -> import round trip"
    );
}

#[test]
fn export_pptx_on_a_nonexistent_input_fails_cleanly() {
    let dir = tempdir();
    let output = run(&[
        "export-pptx",
        "/no/such/file.deckpkg",
        dir.join("out.pptx").to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).starts_with("error:"));
}

// ---------------------------------------------------------------------------
// export-embedded-json
// ---------------------------------------------------------------------------

#[test]
fn export_embedded_json_on_a_text_only_deck_drops_the_assets_array() {
    let dir = tempdir();
    let deckpkg = dir.join("deck.deckpkg");
    run(&["new", deckpkg.to_str().unwrap(), "Embed Me"]);

    let embedded = dir.join("embedded.json");
    let output = run(&[
        "export-embedded-json",
        deckpkg.to_str().unwrap(),
        embedded.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("not for re-import"));

    let raw = fs::read_to_string(&embedded).expect("read embedded json");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert!(parsed.get("assets").is_none());
}

#[test]
fn export_embedded_json_inlines_a_real_image_as_a_data_url() {
    let dir = tempdir();
    let deck_json = dir.join("deck.json");
    let assets_dir = dir.join("assets");
    fs::create_dir_all(&assets_dir).expect("create assets dir");

    fs::write(
        &deck_json,
        r##"{
            "id": "11111111-1111-1111-1111-111111111111",
            "metadata": { "title": "Image Deck", "author": null },
            "theme": {
                "name": "Default",
                "background": { "value": "#FFFFFF" },
                "foreground": { "value": "#111111" }
            },
            "assets": [
                { "id": "99999999-9999-9999-9999-999999999999", "media_type": "image/png", "alt": null }
            ],
            "slides": [
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "name": "Slide 1",
                    "size": { "width": 960.0, "height": 540.0 },
                    "elements": [
                        {
                            "type": "Image",
                            "id": "33333333-3333-3333-3333-333333333333",
                            "bounds": { "x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0 },
                            "asset_id": "99999999-9999-9999-9999-999999999999",
                            "alt": null
                        }
                    ]
                }
            ]
        }"##,
    )
    .expect("write deck.json");

    fs::write(
        assets_dir.join("99999999-9999-9999-9999-999999999999.png"),
        minimal_png_bytes(),
    )
    .expect("write asset");

    let packed = dir.join("packed.deckpkg");
    let pack_output = run(&["pack", deck_json.to_str().unwrap(), packed.to_str().unwrap()]);
    assert!(pack_output.status.success());

    let embedded = dir.join("embedded.json");
    let export_output = run(&[
        "export-embedded-json",
        packed.to_str().unwrap(),
        embedded.to_str().unwrap(),
    ]);
    assert!(export_output.status.success());

    let raw = fs::read_to_string(&embedded).expect("read embedded json");
    assert!(raw.contains("data:image/png;base64,"));
    assert!(!raw.contains("asset_id"));
}

// ---------------------------------------------------------------------------
// move-element / resize-element / update-text
// ---------------------------------------------------------------------------

#[test]
fn move_resize_and_update_text_all_persist_to_disk() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Editable Deck"]);

    let (slide_id, element_id) = first_slide_and_element_id(&file);

    let move_output = run(&[
        "move-element",
        file.to_str().unwrap(),
        &slide_id,
        &element_id,
        "300",
        "220",
    ]);
    assert!(move_output.status.success(), "stderr: {}", stderr(&move_output));
    assert!(stdout(&move_output).contains("Element moved"));

    let resize_output = run(&[
        "resize-element",
        file.to_str().unwrap(),
        &slide_id,
        &element_id,
        "640",
        "90",
    ]);
    assert!(resize_output.status.success());
    assert!(stdout(&resize_output).contains("Element resized"));

    let update_output = run(&[
        "update-text",
        file.to_str().unwrap(),
        &slide_id,
        &element_id,
        "Updated text",
    ]);
    assert!(update_output.status.success());
    assert!(stdout(&update_output).contains("Text updated"));

    let package = DeckPackage::open(&file).expect("open after edits");
    let Element::Text(text) = &package.presentation.slides[0].elements[0] else {
        panic!("expected a Text element");
    };

    assert_eq!(text.text, "Updated text");
    assert_eq!(text.bounds.x, 300.0);
    assert_eq!(text.bounds.y, 220.0);
    assert_eq!(text.bounds.width, 640.0);
    assert_eq!(text.bounds.height, 90.0);
}

#[test]
fn move_element_with_an_unknown_slide_id_fails_with_a_clear_message() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Deck"]);

    let bogus_slide_id = uuid::Uuid::new_v4().to_string();
    let (_, element_id) = first_slide_and_element_id(&file);

    let output = run(&[
        "move-element",
        file.to_str().unwrap(),
        &bogus_slide_id,
        &element_id,
        "0",
        "0",
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("slide not found"));
}

#[test]
fn update_text_with_an_unknown_element_id_fails_with_a_clear_message() {
    let dir = tempdir();
    let file = dir.join("deck.deckpkg");
    run(&["new", file.to_str().unwrap(), "Deck"]);

    let (slide_id, _) = first_slide_and_element_id(&file);
    let bogus_element_id = uuid::Uuid::new_v4().to_string();

    let output = run(&[
        "update-text",
        file.to_str().unwrap(),
        &slide_id,
        &bogus_element_id,
        "irrelevant",
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("element not found"));
}