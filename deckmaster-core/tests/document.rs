use deckmaster_core::{Document, Presentation};

#[test]
fn can_add_slide() {
    let path = std::env::temp_dir().join("deckmaster_test.deck.json");

    let presentation = Presentation::new("Test");

    let mut document =
        Document::create(&path, presentation).unwrap();

    document.add_slide("Results");

    assert_eq!(
        document.presentation().slides.len(),
        1
    );
}

#[test]
fn can_find_slide_by_id() {
    let path =
        std::env::temp_dir().join("deckmaster_find.deck.json");

    let presentation = Presentation::new("Test");

    let mut document =
        Document::create(&path, presentation).unwrap();

    document.add_slide("Results");

    let slide_id =
        document.presentation().slides[0].id;

    let slide =
        document.find_slide(slide_id);

    assert!(slide.is_some());

    assert_eq!(
        slide.unwrap().name.as_deref(),
        Some("Results")
    );
}