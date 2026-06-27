use deckmaster_core::io::{from_json, to_json};
use deckmaster_core::*;

#[test]
fn presentation_round_trips_through_json() {
    let mut deck = Presentation::new("Demo");

    let mut slide = Slide::new(Some("Results".to_string()));
    slide.elements.push(Element::Text(TextElement {
        id: uuid::Uuid::new_v4(),
        bounds: Rect {
            x: 100.0,
            y: 200.0,
            width: 500.0,
            height: 80.0,
        },
        text: "Revenue increased 24%".to_string(),
        font_size: 32.0,
        color: Color::hex("#111111"),
    }));

    deck.slides.push(slide);

    let json = to_json(&deck).unwrap();
    let parsed = from_json(&json).unwrap();

    assert_eq!(deck, parsed);
}