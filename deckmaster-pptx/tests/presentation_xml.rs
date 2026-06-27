use deckmaster_pptx::{
    Package,
    PresentationXml,
};

#[test]
fn can_load_presentation_xml() {
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

    assert!(
        !presentation.xml().is_empty()
    );
}

#[test]
fn presentation_xml_contains_slide_refs() {
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

    assert!(
        presentation
            .xml()
            .contains("sldId")
    );
}