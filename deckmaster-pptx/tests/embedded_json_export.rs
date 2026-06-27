//! Tests for the embedded-JSON convenience export (see
//! docs/DECKPKG_SPEC.md sections 2/5/7). This is deliberately NOT a
//! round-trip test -- the format is one-way by design -- so these checks
//! focus on shape: does it inline real bytes as a correct data: URL, and
//! does it avoid carrying both an asset_id and a src on the same
//! element.

use deckmaster_core::{Asset, DeckPackage, Element, ImageElement, Presentation, Rect, Slide};
use deckmaster_pptx::EmbeddedJsonExporter;
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

fn presentation_with_image(asset_id: Uuid) -> Presentation {
    let mut presentation = Presentation::new("Embed Export Deck");
    presentation.assets.push(Asset {
        id: asset_id,
        media_type: "image/png".to_string(),
        alt: Some("alt text".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        },
        asset_id,
        alt: Some("alt text".to_string()),
    }));
    presentation.slides.push(slide);

    presentation
}

#[test]
fn embedded_json_inlines_a_correct_data_url() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let rendered = EmbeddedJsonExporter::render(&package).expect("render embedded json");
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");

    let image_element = &parsed["slides"][0]["elements"][0];

    assert!(
        image_element.get("asset_id").is_none(),
        "embedded export must not carry asset_id alongside src"
    );

    let src = image_element["src"]
        .as_str()
        .expect("image element must have a src string");

    assert!(src.starts_with("data:image/png;base64,"));

    let base64_part = src.trim_start_matches("data:image/png;base64,");
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_part,
    )
    .expect("src must be valid base64");

    assert_eq!(decoded, sample_png_bytes());
}

#[test]
fn embedded_json_drops_the_top_level_assets_array() {
    let asset_id = Uuid::new_v4();
    let presentation = presentation_with_image(asset_id);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let rendered = EmbeddedJsonExporter::render(&package).expect("render embedded json");
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");

    assert!(
        parsed.get("assets").is_none(),
        "embedded export carries bytes per-element now, so the separate \
         assets[] registry would just be redundant dead weight"
    );
}
