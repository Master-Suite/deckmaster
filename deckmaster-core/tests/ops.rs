use deckmaster_core::{
    move_element,
    resize_element,
    update_text,
    Element,
    Presentation,
    Slide,
};

#[test]
fn can_move_element() {
    let mut presentation = Presentation::new("Ops Test");

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.add_text(
        "Move me",
        100.0,
        200.0,
        500.0,
        80.0,
    );

    let slide_id = slide.id;

    let element_id = match &slide.elements[0] {
        Element::Text(text) => text.id,
        _ => panic!("expected text element"),
    };

    presentation.slides.push(slide);

    move_element(
        &mut presentation,
        slide_id,
        element_id,
        300.0,
        400.0,
    )
    .unwrap();

    let moved = match &presentation.slides[0].elements[0] {
        Element::Text(text) => text,
        _ => panic!("expected text element"),
    };

    assert_eq!(moved.bounds.x, 300.0);
    assert_eq!(moved.bounds.y, 400.0);
}

#[test]
fn can_resize_element() {
    let mut presentation = Presentation::new("Ops Test");

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.add_text(
        "Resize me",
        100.0,
        200.0,
        500.0,
        80.0,
    );

    let slide_id = slide.id;

    let element_id = match &slide.elements[0] {
        Element::Text(text) => text.id,
        _ => panic!("expected text element"),
    };

    presentation.slides.push(slide);

    resize_element(
        &mut presentation,
        slide_id,
        element_id,
        640.0,
        120.0,
    )
    .unwrap();

    let resized = match &presentation.slides[0].elements[0] {
        Element::Text(text) => text,
        _ => panic!("expected text element"),
    };

    assert_eq!(resized.bounds.width, 640.0);
    assert_eq!(resized.bounds.height, 120.0);
}

#[test]
fn can_update_text_element() {
    let mut presentation = Presentation::new("Ops Test");

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.add_text(
        "Old text",
        100.0,
        200.0,
        500.0,
        80.0,
    );

    let slide_id = slide.id;

    let element_id = match &slide.elements[0] {
        Element::Text(text) => text.id,
        _ => panic!("expected text element"),
    };

    presentation.slides.push(slide);

    update_text(
        &mut presentation,
        slide_id,
        element_id,
        "New text",
    )
    .unwrap();

    let updated = match &presentation.slides[0].elements[0] {
        Element::Text(text) => text,
        _ => panic!("expected text element"),
    };

    assert_eq!(updated.text, "New text");
}