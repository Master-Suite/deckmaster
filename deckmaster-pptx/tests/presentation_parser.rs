use deckmaster_pptx::{
    Package,
    PresentationParser,
    PresentationXml,
};

#[test]
fn finds_slide_relationships() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let mut package =
        Package::open(path).unwrap();

    let presentation =
        PresentationXml::load(
            &mut package
        )
        .unwrap();

    let refs =
        PresentationParser::slide_relationships(
            presentation.xml(),
        )
        .unwrap();

    println!("slide refs: {:#?}", refs);

    assert!(
        !refs.is_empty(),
        "expected at least one slide"
    );
}