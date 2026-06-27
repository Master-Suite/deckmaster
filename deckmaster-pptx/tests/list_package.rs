use deckmaster_pptx::Package;

#[test]
fn list_package_files() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let mut package =
        Package::open(path).unwrap();

    let files =
        package.file_names();

    for file in files {
        println!("{file}");
    }
}