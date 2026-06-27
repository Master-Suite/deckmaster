use std::fs::File;
use zip::ZipArchive;

#[test]
fn can_open_pptx_package() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sample.pptx"
    );

    let file =
        File::open(path).expect("failed to open sample pptx");

    let archive =
        ZipArchive::new(file).expect("failed to read zip archive");

    assert!(
        archive.len() > 0,
        "pptx package should contain files"
    );
}