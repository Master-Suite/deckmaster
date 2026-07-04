use deckmaster_core::{
    extension_for_media_type, validate, Asset, Color, DeckPackage, Element, ImageElement,
    MathElement, Presentation, Rect, Severity, Slide, TextElement,
};
use uuid::Uuid;

fn sample_png_bytes() -> Vec<u8> {
    // A real, minimal valid 1x1 PNG (not just arbitrary bytes), so tests
    // that care about "is this actually image data" have something
    // truthful to check, and so a human opening a fixture file in an
    // image viewer would see something rather than an error.
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0xCD, 0x37, 0x42, 0x49, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn presentation_with_text_only() -> Presentation {
    let mut presentation = Presentation::new("Text Only Deck");

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.add_text("Hello, DeckMaster", 100.0, 100.0, 500.0, 80.0);
    presentation.slides.push(slide);

    presentation
}

fn presentation_with_image(asset_id: Uuid) -> Presentation {
    let mut presentation = Presentation::new("Image Deck");

    presentation.assets.push(Asset {
        id: asset_id,
        media_type: "image/png".to_string(),
        alt: Some("a test image".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 50.0,
            y: 60.0,
            width: 200.0,
            height: 150.0,
        },
        asset_id,
        render_asset_id: None,
        alt: Some("a test image".to_string()),
    }));

    presentation.slides.push(slide);

    presentation
}

#[test]
fn text_only_presentation_round_trips_through_json() {
    let presentation = presentation_with_text_only();

    let json = deckmaster_core::to_json(&presentation).expect("serialize");
    let reparsed = deckmaster_core::from_json(&json).expect("deserialize");

    assert_eq!(presentation, reparsed);
}

#[test]
fn image_element_serializes_with_asset_id_not_src() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let json = deckmaster_core::to_json(&presentation).expect("serialize");

    assert!(
        json.contains("asset_id"),
        "serialized deck.json must reference images via asset_id"
    );
    assert!(
        !json.contains("\"src\""),
        "serialized deck.json must never carry a src field on image elements -- \
         per docs/DECKPKG_SPEC.md, canonical decks only ever use asset_id"
    );
    assert!(
        !json.contains("data:image"),
        "serialized deck.json must never embed a data: URL -- that's the \
         one-way embedded-json export's job, not the canonical format"
    );
}

#[test]
fn math_element_round_trips_through_json() {
    let mut presentation = Presentation::new("Math Deck");

    let mut slide = Slide::new(Some("Equation Slide".to_string()));
    slide.elements.push(Element::Math(MathElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 120.0,
            width: 500.0,
            height: 80.0,
        },
        tex: "\\frac{1}{1 + e^{-x}}".to_string(),
        font_size: 36.0,
        color: Color::hex("#111111"),
        render_asset_id: None,
    }));

    presentation.slides.push(slide);

    let json = deckmaster_core::to_json(&presentation).expect("serialize");
    assert!(json.contains(r#""type": "Math""#));
    assert!(json.contains(r#""tex": "\\frac{1}{1 + e^{-x}}""#));

    let reparsed = deckmaster_core::from_json(&json).expect("deserialize");

    assert_eq!(presentation, reparsed);
}

#[test]
fn pdf_assets_use_pdf_extension() {
    assert_eq!(extension_for_media_type("application/pdf"), "pdf");
    assert_eq!(
        deckmaster_core::media_type_for_extension("pdf"),
        "application/pdf"
    );
}

#[test]
fn svg_assets_use_svg_extension() {
    assert_eq!(extension_for_media_type("image/svg+xml"), "svg");
    assert_eq!(
        deckmaster_core::media_type_for_extension("svg"),
        "image/svg+xml"
    );
}

#[test]
fn deck_package_round_trips_through_a_real_zip_on_disk() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let mut package = DeckPackage::new(presentation.clone());
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let dir = tempdir();
    let path = dir.join("round_trip.deckpkg");

    package.save(&path).expect("save package");

    let reopened = DeckPackage::open(&path).expect("open package");

    assert_eq!(reopened.presentation, presentation);
    assert_eq!(
        reopened.asset_bytes.get(&asset_id).map(Vec::as_slice),
        Some(sample_png_bytes().as_slice())
    );
}

#[test]
fn deck_package_zip_has_deck_json_at_root_and_assets_under_assets_dir() {
    // This pins the actual on-disk layout from docs/DECKPKG_SPEC.md section
    // 3 -- not just "does our own reader understand our own writer" but
    // "is the zip laid out the way the spec says", since a future
    // independent reader (or a person unzipping by hand) relies on this.
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let dir = tempdir();
    let path = dir.join("layout_check.deckpkg");
    package.save(&path).expect("save package");

    let file = std::fs::File::open(&path).expect("open file");
    let mut archive = zip::ZipArchive::new(file).expect("read zip");

    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(names.contains(&"deck.json".to_string()));

    let expected_asset_path = format!(
        "assets/{}.{}",
        asset_id,
        extension_for_media_type("image/png")
    );
    assert!(names.contains(&expected_asset_path));
}

#[test]
fn validate_passes_on_a_clean_text_only_deck() {
    let presentation = presentation_with_text_only();
    let package = DeckPackage::new(presentation);

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(errors.is_empty(), "unexpected errors: {issues:?}");
}

#[test]
fn validate_passes_on_a_clean_image_deck_with_bytes_present() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(errors.is_empty(), "unexpected errors: {issues:?}");
}

#[test]
fn validate_flags_an_image_element_referencing_an_undeclared_asset_id() {
    let declared_asset_id = Uuid::new_v4();
    let dangling_asset_id = Uuid::new_v4();

    let mut presentation = presentation_with_image(declared_asset_id);

    // Corrupt the one image element to point at an asset id that was
    // never declared in assets[] -- the classic "someone hand-edited
    // deck.json and typo'd a uuid" failure mode.
    if let Element::Image(image) = &mut presentation.slides[0].elements[0] {
        image.asset_id = dangling_asset_id;
    }

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(declared_asset_id, sample_png_bytes());

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "expected validate() to flag the dangling asset_id reference"
    );
    assert!(errors
        .iter()
        .any(|issue| issue.message.contains(&dangling_asset_id.to_string())));
}

#[test]
fn validate_flags_a_declared_asset_missing_its_bytes() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    // Deliberately do NOT insert bytes for this asset -- simulates a
    // .deckpkg whose assets/ folder is missing a file that deck.json
    // still references.
    let package = DeckPackage::new(presentation);

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "expected validate() to flag the missing asset bytes"
    );
}

#[test]
fn validate_notes_but_does_not_error_on_an_unreferenced_asset() {
    let referenced_asset_id = Uuid::new_v4();
    let unused_asset_id = Uuid::new_v4();

    let mut presentation = presentation_with_image(referenced_asset_id);
    presentation.assets.push(Asset {
        id: unused_asset_id,
        media_type: "image/png".to_string(),
        alt: None,
    });

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(referenced_asset_id, sample_png_bytes());
    package
        .asset_bytes
        .insert(unused_asset_id, sample_png_bytes());

    let issues = validate(&package);

    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "an unused-but-present asset must not be a hard error: {issues:?}"
    );

    let infos: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Info)
        .collect();
    assert!(
        infos
            .iter()
            .any(|issue| issue.message.contains(&unused_asset_id.to_string())),
        "expected an informational note about the unused asset, got: {issues:?}"
    );
}

#[test]
fn validate_flags_a_presentation_with_no_slides() {
    let presentation = Presentation::new("Empty Deck");
    let package = DeckPackage::new(presentation);

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(!errors.is_empty());
}

#[test]
fn validate_flags_negative_element_bounds() {
    let mut presentation = presentation_with_text_only();

    presentation.slides[0]
        .elements
        .push(Element::Text(TextElement {
            id: Uuid::new_v4(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: -10.0,
                height: 20.0,
            },
            text: "broken bounds".to_string(),
            font_size: 18.0,
            color: Color::hex("#000000"),
        }));

    let package = DeckPackage::new(presentation);
    let issues = validate(&package);

    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(!errors.is_empty());
}

#[test]
fn validate_flags_empty_math_tex() {
    let mut presentation = Presentation::new("Broken Math Deck");

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Math(MathElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 500.0,
            height: 80.0,
        },
        tex: "   ".to_string(),
        font_size: 36.0,
        color: Color::hex("#111111"),
        render_asset_id: None,
    }));

    presentation.slides.push(slide);

    let package = DeckPackage::new(presentation);
    let issues = validate(&package);

    assert!(
        issues
            .iter()
            .any(|issue| issue.severity == Severity::Error && issue.message.contains("empty tex")),
        "expected empty math tex validation error, got: {issues:?}"
    );
}

#[test]
fn validate_accepts_pdf_image_asset_with_present_bytes() {
    let pdf_asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("PDF Image Deck");
    presentation.assets.push(Asset {
        id: pdf_asset_id,
        media_type: "application/pdf".to_string(),
        alt: Some("diagram".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 10.0,
            y: 10.0,
            width: 200.0,
            height: 120.0,
        },
        asset_id: pdf_asset_id,
        render_asset_id: None,
        alt: Some("diagram".to_string()),
    }));
    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(pdf_asset_id, b"%PDF-1.4\n".to_vec());

    let issues = validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    assert!(errors.is_empty(), "unexpected errors: {issues:?}");
}

#[test]
fn validate_requires_math_render_asset_to_be_raster() {
    let pdf_asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("Broken Math Fallback Deck");
    presentation.assets.push(Asset {
        id: pdf_asset_id,
        media_type: "application/pdf".to_string(),
        alt: None,
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Math(MathElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 500.0,
            height: 80.0,
        },
        tex: "x^2".to_string(),
        font_size: 36.0,
        color: Color::hex("#111111"),
        render_asset_id: Some(pdf_asset_id),
    }));
    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(pdf_asset_id, b"%PDF-1.4\n".to_vec());

    let issues = validate(&package);

    assert!(
        issues.iter().any(|issue| {
            issue.severity == Severity::Error
                && issue.message.contains("must point to a raster image asset")
        }),
        "expected raster fallback validation error, got: {issues:?}"
    );
}

#[test]
fn validate_rejects_svg_as_a_raster_render_fallback() {
    let svg_asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("Broken SVG Fallback Deck");
    presentation.assets.push(Asset {
        id: svg_asset_id,
        media_type: "image/svg+xml".to_string(),
        alt: None,
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Math(MathElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 500.0,
            height: 80.0,
        },
        tex: "x^2".to_string(),
        font_size: 36.0,
        color: Color::hex("#111111"),
        render_asset_id: Some(svg_asset_id),
    }));
    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(
        svg_asset_id,
        br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#.to_vec(),
    );

    let issues = validate(&package);

    assert!(
        issues.iter().any(|issue| {
            issue.severity == Severity::Error
                && issue.message.contains("must point to a raster image asset")
        }),
        "expected SVG render fallback validation error, got: {issues:?}"
    );
}

/// A fresh temp directory under the crate's own target dir, cleaned up by
/// the OS (or the next CI run) rather than via a Drop guard -- the test
/// binary's working directory churn isn't worth a dependency just for
/// this. Mirrors what the existing CLI smoke testing already does by
/// hand in /tmp.
fn tempdir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("deckmaster-core-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}
