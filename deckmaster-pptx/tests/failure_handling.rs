//! Phase 10C equivalent: "DeckMaster refuses to lie" checkpoint, adapted
//! to the asset_id model. The exporter must fail loudly, not silently
//! skip or produce a corrupt pptx, when a deck references assets it
//! can't actually back with bytes.

use deckmaster_core::{Asset, DeckPackage, Element, ImageElement, Presentation, Rect, Slide};
use deckmaster_pptx::PptxExporter;
use uuid::Uuid;

fn sample_png_bytes() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0xCD, 0x37, 0x42, 0x49, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn tempdir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("deckmaster-pptx-failtest-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn presentation_with_one_image(asset_id: Uuid) -> Presentation {
    let mut presentation = Presentation::new("Failure Cluster Deck");
    presentation.assets.push(Asset {
        id: asset_id,
        media_type: "image/png".to_string(),
        alt: None,
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 100.0,
        },
        asset_id,
        alt: None,
    }));
    presentation.slides.push(slide);

    presentation
}

#[test]
fn export_fails_when_asset_bytes_are_missing_from_the_package() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_one_image(asset_id);

    // Asset is declared in assets[] but never given bytes -- exactly the
    // shape of a .deckpkg whose assets/ folder lost a file.
    let package = DeckPackage::new(presentation);

    let dir = tempdir();
    let output = dir.join("should_not_exist.pptx");

    let result = PptxExporter::export(&package, &output);

    assert!(
        result.is_err(),
        "exporter must refuse to export when an asset's bytes are missing, not silently skip the image"
    );
    assert!(
        !output.exists(),
        "exporter must not leave a partial/broken pptx file behind on failure"
    );
}

#[test]
fn export_fails_when_image_references_an_undeclared_asset_id() {
    let declared_asset_id = Uuid::new_v4();
    let dangling_asset_id = Uuid::new_v4();

    let mut presentation = presentation_with_one_image(declared_asset_id);

    // Corrupt the element to reference an asset that was never declared.
    if let Element::Image(image) = &mut presentation.slides[0].elements[0] {
        image.asset_id = dangling_asset_id;
    }

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(declared_asset_id, sample_png_bytes());

    let dir = tempdir();
    let output = dir.join("should_not_exist.pptx");

    let result = PptxExporter::export(&package, &output);

    assert!(
        result.is_err(),
        "exporter must refuse to export an image element with a dangling asset_id"
    );
    assert!(!output.exists());
}

#[test]
fn export_fails_on_an_empty_presentation_with_no_slides() {
    let presentation = Presentation::new("No Slides Deck");
    let package = DeckPackage::new(presentation);

    let dir = tempdir();
    let output = dir.join("should_not_exist.pptx");

    let result = PptxExporter::export(&package, &output);

    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn export_succeeds_and_produces_a_valid_zip_when_everything_is_present() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_one_image(asset_id);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let dir = tempdir();
    let output = dir.join("should_exist.pptx");

    PptxExporter::export(&package, &output).expect("export should succeed");

    assert!(output.exists());

    // Sanity: the result is a well-formed zip, not just a file that
    // happens to exist.
    let file = std::fs::File::open(&output).expect("open output");
    zip::ZipArchive::new(file).expect("output must be a readable zip");
}
