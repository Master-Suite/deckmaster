//! Phase 10A equivalent: round-trip a presentation we build ourselves
//! (not a real-world PPTX) through export -> reimport, checking that
//! text and image content, bounds, and structure survive intact. See
//! docs/DECKPKG_SPEC.md and the Phase 10 roadmap entries this descends
//! from.

use deckmaster_core::{
    Asset, Color, DeckPackage, Element, ImageElement, MathElement, Presentation, Rect, Slide,
    TextElement,
};
use deckmaster_pptx::{PptxExporter, PptxImporter};
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
    dir.push(format!("deckmaster-pptx-test-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn mixed_text_and_image_slide_round_trips() {
    let asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("Mixed Slide Deck");
    presentation.assets.push(Asset {
        id: asset_id,
        media_type: "image/png".to_string(),
        alt: Some("a tiny test square".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.elements.push(Element::Text(TextElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 80.0,
            width: 600.0,
            height: 90.0,
        },
        text: "Mixed content slide".to_string(),
        font_size: 30.0,
        color: Color::hex("#222233"),
    }));

    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 220.0,
            width: 180.0,
            height: 180.0,
        },
        asset_id,
        render_asset_id: None,
        alt: Some("a tiny test square".to_string()),
    }));

    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id, sample_png_bytes());

    let dir = tempdir();
    let pptx_path = dir.join("mixed.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    assert_eq!(reimported.presentation.slides.len(), 1);

    let elements = &reimported.presentation.slides[0].elements;
    let texts: Vec<_> = elements
        .iter()
        .filter_map(|e| match e {
            Element::Text(t) => Some(t),
            _ => None,
        })
        .collect();
    let images: Vec<_> = elements
        .iter()
        .filter_map(|e| match e {
            Element::Image(i) => Some(i),
            _ => None,
        })
        .collect();

    assert_eq!(texts.len(), 1);
    assert_eq!(images.len(), 1);
    assert_eq!(texts[0].text, "Mixed content slide");

    // Bounds should survive the pt -> EMU -> pt round trip to within
    // floating point / rounding tolerance (EMU is 1/12700 pt, so any
    // drift should be sub-pixel).
    assert!((texts[0].bounds.x - 100.0).abs() < 0.1);
    assert!((texts[0].bounds.y - 80.0).abs() < 0.1);
    assert!((images[0].bounds.width - 180.0).abs() < 0.1);
    assert!((images[0].bounds.height - 180.0).abs() < 0.1);

    // The reimported image must resolve to a real declared asset with
    // real bytes -- this is the asset_id contract, not the old
    // src-as-data-url contract.
    let reimported_asset = reimported
        .presentation
        .find_asset(images[0].asset_id)
        .expect("reimported image must reference a declared asset");

    let bytes = reimported
        .asset_bytes
        .get(&reimported_asset.id)
        .expect("reimported asset must have bytes in the package");

    assert_eq!(bytes, &sample_png_bytes());
}

#[test]
fn multiple_images_on_one_slide_round_trip_with_distinct_assets() {
    let asset_id_a = Uuid::new_v4();
    let asset_id_b = Uuid::new_v4();

    let mut presentation = Presentation::new("Two Images Deck");
    presentation.assets.push(Asset {
        id: asset_id_a,
        media_type: "image/png".to_string(),
        alt: None,
    });
    presentation.assets.push(Asset {
        id: asset_id_b,
        media_type: "image/png".to_string(),
        alt: None,
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        },
        asset_id: asset_id_a,
        render_asset_id: None,
        alt: None,
    }));

    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 400.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        },
        asset_id: asset_id_b,
        render_asset_id: None,
        alt: None,
    }));

    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id_a, sample_png_bytes());
    package.asset_bytes.insert(asset_id_b, sample_png_bytes());

    let dir = tempdir();
    let pptx_path = dir.join("two_images.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    let images: Vec<_> = reimported.presentation.slides[0]
        .elements
        .iter()
        .filter_map(|e| match e {
            Element::Image(i) => Some(i),
            _ => None,
        })
        .collect();

    assert_eq!(images.len(), 2);

    // The two images must come back as two DISTINCT assets, not
    // collapsed into one -- each <p:pic> in the pptx is its own media
    // file and must round-trip to its own asset_id.
    assert_ne!(images[0].asset_id, images[1].asset_id);
    assert_eq!(reimported.presentation.assets.len(), 2);

    for image in &images {
        assert!(
            reimported.asset_bytes.contains_key(&image.asset_id),
            "every reimported image's asset_id must have bytes present"
        );
    }
}

#[test]
fn images_across_multiple_slides_round_trip() {
    let asset_id_1 = Uuid::new_v4();
    let asset_id_2 = Uuid::new_v4();

    let mut presentation = Presentation::new("Multi Slide Image Deck");
    presentation.assets.push(Asset {
        id: asset_id_1,
        media_type: "image/png".to_string(),
        alt: None,
    });
    presentation.assets.push(Asset {
        id: asset_id_2,
        media_type: "image/png".to_string(),
        alt: None,
    });

    let mut slide1 = Slide::new(Some("Slide 1".to_string()));
    slide1.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        asset_id: asset_id_1,
        render_asset_id: None,
        alt: None,
    }));

    let mut slide2 = Slide::new(Some("Slide 2".to_string()));
    slide2.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        asset_id: asset_id_2,
        render_asset_id: None,
        alt: None,
    }));

    presentation.slides.push(slide1);
    presentation.slides.push(slide2);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(asset_id_1, sample_png_bytes());
    package.asset_bytes.insert(asset_id_2, sample_png_bytes());

    let dir = tempdir();
    let pptx_path = dir.join("multi_slide.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    assert_eq!(reimported.presentation.slides.len(), 2);
    assert_eq!(reimported.presentation.assets.len(), 2);

    for slide in &reimported.presentation.slides {
        assert_eq!(slide.elements.len(), 1);
    }
}

#[test]
fn slide_size_is_preserved_through_export() {
    let mut presentation = Presentation::new("Custom Size Deck");
    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.size.width = 720.0;
    slide.size.height = 405.0;
    slide.add_text("16:9 at a non-default size", 50.0, 50.0, 400.0, 60.0);
    presentation.slides.push(slide);

    let package = DeckPackage::new(presentation);

    let dir = tempdir();
    let pptx_path = dir.join("custom_size.pptx");
    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let file = std::fs::File::open(&pptx_path).expect("open pptx");
    let mut archive = zip::ZipArchive::new(file).expect("read zip");
    let mut presentation_xml = String::new();
    {
        use std::io::Read;
        archive
            .by_name("ppt/presentation.xml")
            .expect("presentation.xml present")
            .read_to_string(&mut presentation_xml)
            .expect("read presentation.xml");
    }

    // 720pt * 12700 EMU/pt = 9144000; 405pt * 12700 = 5143500.
    assert!(presentation_xml.contains("cx=\"9144000\""));
    assert!(presentation_xml.contains("cy=\"5143500\""));
}

#[test]
fn math_without_render_fallback_exports_as_visible_tex_text() {
    let mut presentation = Presentation::new("Math Fallback Deck");

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Math(MathElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 500.0,
            height: 80.0,
        },
        tex: "\\alpha + \\beta".to_string(),
        font_size: 32.0,
        color: Color::hex("#111111"),
        render_asset_id: None,
    }));
    presentation.slides.push(slide);

    let package = DeckPackage::new(presentation);

    let dir = tempdir();
    let pptx_path = dir.join("math_fallback.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    let has_tex_text = reimported
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .any(|element| matches!(element, Element::Text(text) if text.text == "\\alpha + \\beta"));

    assert!(
        has_tex_text,
        "math without render fallback should remain visible as raw TeX text in PPTX"
    );
}

#[test]
fn pdf_image_with_raster_fallback_exports_as_pptx_image() {
    let pdf_asset_id = Uuid::new_v4();
    let fallback_asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("PDF Fallback Deck");
    presentation.assets.push(Asset {
        id: pdf_asset_id,
        media_type: "application/pdf".to_string(),
        alt: Some("source pdf".to_string()),
    });
    presentation.assets.push(Asset {
        id: fallback_asset_id,
        media_type: "image/png".to_string(),
        alt: Some("raster fallback".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 180.0,
            height: 120.0,
        },
        asset_id: pdf_asset_id,
        render_asset_id: Some(fallback_asset_id),
        alt: Some("source pdf".to_string()),
    }));
    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package
        .asset_bytes
        .insert(pdf_asset_id, b"%PDF-1.4\n".to_vec());
    package
        .asset_bytes
        .insert(fallback_asset_id, sample_png_bytes());

    let dir = tempdir();
    let pptx_path = dir.join("pdf_fallback.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    let image_count = reimported
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .filter(|element| matches!(element, Element::Image(_)))
        .count();

    assert_eq!(image_count, 1);
}

#[test]
fn svg_image_with_raster_fallback_exports_as_pptx_image() {
    let svg_asset_id = Uuid::new_v4();
    let fallback_asset_id = Uuid::new_v4();

    let mut presentation = Presentation::new("SVG Fallback Deck");
    presentation.assets.push(Asset {
        id: svg_asset_id,
        media_type: "image/svg+xml".to_string(),
        alt: Some("source svg".to_string()),
    });
    presentation.assets.push(Asset {
        id: fallback_asset_id,
        media_type: "image/png".to_string(),
        alt: Some("raster fallback".to_string()),
    });

    let mut slide = Slide::new(Some("Slide 1".to_string()));
    slide.elements.push(Element::Image(ImageElement {
        id: Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 100.0,
            width: 180.0,
            height: 120.0,
        },
        asset_id: svg_asset_id,
        render_asset_id: Some(fallback_asset_id),
        alt: Some("source svg".to_string()),
    }));
    presentation.slides.push(slide);

    let mut package = DeckPackage::new(presentation);
    package.asset_bytes.insert(
        svg_asset_id,
        br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#.to_vec(),
    );
    package
        .asset_bytes
        .insert(fallback_asset_id, sample_png_bytes());

    let dir = tempdir();
    let pptx_path = dir.join("svg_fallback.pptx");

    PptxExporter::export(&package, &pptx_path).expect("export to pptx");

    let reimported = PptxImporter::import(&pptx_path).expect("import from pptx");

    let image_count = reimported
        .presentation
        .slides
        .iter()
        .flat_map(|slide| &slide.elements)
        .filter(|element| matches!(element, Element::Image(_)))
        .count();

    assert_eq!(image_count, 1);
}