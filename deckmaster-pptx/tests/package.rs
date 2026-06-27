use deckmaster_pptx::Package;

#[test]
fn package_lists_files() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let mut package =
        Package::open(path).unwrap();

    let files =
        package.file_names();

    assert!(
        !files.is_empty()
    );
}

#[test]
fn package_contains_content_types() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let mut package =
        Package::open(path).unwrap();

    assert!(
        package.contains(
            "[Content_Types].xml"
        )
    );
}