//! Phase 10D equivalent: smoke test against a real PPTX file (generated
//! by python-pptx, an independent implementation, not by our own
//! exporter) rather than a presentation we constructed ourselves. This
//! is the test that catches "our round-trip tests pass because both
//! sides agree with each other, but neither agrees with reality."
//!
//! Marked #[ignore] so a default `cargo test` doesn't depend on the
//! fixture file's presence -- run explicitly with:
//!   cargo test -p deckmaster-pptx --test real_world_smoke -- --ignored

use deckmaster_core::Element;
use deckmaster_pptx::{PptxExporter, PptxImporter};
use uuid::Uuid;

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real_world_fixture.pptx")
}

fn tempdir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("deckmaster-pptx-realworld-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
#[ignore]
fn real_pptx_imports_without_crashing() {
    let result = PptxImporter::import(fixture_path());
    assert!(result.is_ok(), "import failed: {:?}", result.err());
}

#[test]
#[ignore]
fn real_pptx_import_has_usable_elements() {
    let package = PptxImporter::import(fixture_path()).expect("import fixture");

    assert_eq!(
        package.presentation.slides.len(),
        2,
        "fixture was built with exactly 2 slides"
    );

    let total_elements: usize = package
        .presentation
        .slides
        .iter()
        .map(|slide| slide.elements.len())
        .sum();

    assert!(total_elements > 0, "expected at least some imported elements");

    let has_text = package.presentation.slides.iter().any(|slide| {
        slide
            .elements
            .iter()
            .any(|element| matches!(element, Element::Text(_)))
    });
    assert!(has_text, "expected at least one imported text element");
}

#[test]
#[ignore]
fn real_pptx_import_resolves_images_to_real_asset_bytes() {
    let package = PptxImporter::import(fixture_path()).expect("import fixture");

    let image_elements: Vec<_> = package
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .filter_map(|element| match element {
            Element::Image(image) => Some(image),
            _ => None,
        })
        .collect();

    assert!(
        !image_elements.is_empty(),
        "fixture was built with at least one picture shape"
    );

    for image in &image_elements {
        let asset = package
            .presentation
            .find_asset(image.asset_id)
            .expect("every imported image must resolve to a declared asset");

        let bytes = package
            .asset_bytes
            .get(&asset.id)
            .expect("every declared asset must have real bytes in the package");

        assert!(!bytes.is_empty(), "imported asset bytes must be non-empty");

        // Bounds from a real pptx should be plausible -- nonzero, and
        // not absurdly large (catches an EMU/pt conversion regression
        // that would otherwise produce a technically-valid but
        // nonsensical multi-million-point box).
        assert!(image.bounds.width > 0.0 && image.bounds.width < 5000.0);
        assert!(image.bounds.height > 0.0 && image.bounds.height < 5000.0);
    }
}

#[test]
#[ignore]
fn real_pptx_can_be_reexported_and_reimported_again() {
    let package = PptxImporter::import(fixture_path()).expect("import fixture");

    let dir = tempdir();
    let reexported_path = dir.join("reexported.pptx");

    PptxExporter::export(&package, &reexported_path).expect("reexport must succeed");

    let reimported =
        PptxImporter::import(&reexported_path).expect("reimporting the reexported pptx must succeed");

    assert_eq!(
        reimported.presentation.slides.len(),
        package.presentation.slides.len(),
        "slide count must survive a full import -> export -> import cycle"
    );

    let original_image_count: usize = package
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .filter(|element| matches!(element, Element::Image(_)))
        .count();

    let reimported_image_count: usize = reimported
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .filter(|element| matches!(element, Element::Image(_)))
        .count();

    assert_eq!(
        original_image_count, reimported_image_count,
        "image count must survive a full import -> export -> import cycle"
    );
}
