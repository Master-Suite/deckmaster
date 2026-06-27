use quick_xml::events::Event;
use quick_xml::Reader;

use crate::Result;

#[derive(Debug)]
pub struct SlideReference {
    pub relationship_id: String,
}

pub struct PresentationParser;

impl PresentationParser {
    pub fn slide_relationships(
        xml: &str,
    ) -> Result<Vec<SlideReference>> {
        let mut reader =
            Reader::from_str(xml);

        let mut refs = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e))
                | Ok(Event::Empty(ref e)) => {
                    let name = e.name();

                    if name.as_ref().ends_with(b"sldId")
                    {
                        for attr in e.attributes() {
                            let attr =
                                attr.unwrap();

                            let key =
                                String::from_utf8_lossy(
                                    attr.key.as_ref(),
                                );

                            if key == "r:id" {
                                let value =
                                    String::from_utf8_lossy(
                                        attr.value.as_ref(),
                                    )
                                    .to_string();

                                refs.push(
                                    SlideReference {
                                        relationship_id:
                                            value,
                                    },
                                );
                            }
                        }
                    }
                }

                Ok(Event::Eof) => break,

                Err(e) => {
                    panic!(
                        "xml parse error: {e}"
                    );
                }

                _ => {}
            }
        }

        Ok(refs)
    }
}
