use deckmaster_pptx::{
    Package,
    Relationships,
    SlideXml,
};

#[test]
fn loads_first_slide_xml() {
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

    println!(
        "{}",
        &slide.xml()[..200.min(
            slide.xml().len()
        )]
    );

    assert!(
        !slide.xml().is_empty()
    );
}