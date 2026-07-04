use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{Package, Result};

#[derive(Debug, Clone)]
pub struct Relationship {
    pub id: String,
    pub target: String,
}

pub struct Relationships;

impl Relationships {
    pub fn load_presentation_relationships(package: &mut Package) -> Result<Vec<Relationship>> {
        let xml = package.read_string("ppt/_rels/presentation.xml.rels")?;

        Self::parse(&xml)
    }

    pub fn load_slide_relationships(
        package: &mut Package,
        slide_target: &str,
    ) -> Result<Vec<Relationship>> {
        let rels_path = slide_relationships_path(slide_target);

        let xml = package.read_string(&rels_path)?;

        Self::parse(&xml)
    }

    pub fn parse(xml: &str) -> Result<Vec<Relationship>> {
        let mut reader = Reader::from_str(xml);

        let mut rels = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref().ends_with(b"Relationship") {
                        let mut id = None;
                        let mut target = None;

                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref()).to_string();

                            match key.as_ref() {
                                "Id" => id = Some(value),
                                "Target" => target = Some(value),
                                _ => {}
                            }
                        }

                        if let (Some(id), Some(target)) = (id, target) {
                            rels.push(Relationship { id, target });
                        }
                    }
                }

                Ok(Event::Eof) => break,

                Err(e) => {
                    panic!("xml parse error: {e}");
                }

                _ => {}
            }
        }

        Ok(rels)
    }
}

fn slide_relationships_path(slide_target: &str) -> String {
    let normalized = slide_target.trim_start_matches('/');

    let slide_path = normalized.strip_prefix("ppt/").unwrap_or(normalized);

    let Some((dir, file_name)) = slide_path.rsplit_once('/') else {
        return format!("ppt/_rels/{slide_path}.rels");
    };

    format!("ppt/{dir}/_rels/{file_name}.rels")
}
