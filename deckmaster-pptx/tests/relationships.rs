use deckmaster_pptx::{
    Package,
    Relationships,
};

#[test]
fn loads_presentation_relationships() {
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

    println!("{:#?}", rels);

    assert!(
        !rels.is_empty()
    );
}