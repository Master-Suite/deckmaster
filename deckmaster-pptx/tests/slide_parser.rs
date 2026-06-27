use deckmaster_pptx::{
    Package,
    Relationships,
    SlideParser,
    SlideXml,
};

#[test]
fn extracts_text_from_slide() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let mut package =
        Package::open(path).unwrap();

    let rels =
        Relationships::load_presentation_relationships(
            &mut package,
        )
        .unwrap();

    let slide_rel =
        rels.iter()
            .find(|r| {
                r.target.starts_with(
                    "slides/"
                )
            })
            .unwrap();

    let slide =
        SlideXml::load(
            &mut package,
            &slide_rel.target,
        )
        .unwrap();

    let texts =
        SlideParser::extract_text(
            slide.xml(),
        )
        .unwrap();

    println!("{:#?}", texts);

    assert!(
        !texts.is_empty(),
        "expected text in slide"
    );
}