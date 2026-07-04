use deckmaster_core::Rect;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::Result;

pub struct SlideParser;

#[derive(Debug, Clone)]
pub struct ParsedText {
    pub text: String,
    pub bounds: Rect,
    pub font_size: f32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct ParsedImage {
    pub relationship_id: String,
    pub bounds: Rect,
    pub alt: Option<String>,
}

impl SlideParser {
    pub fn extract_text(xml: &str) -> Result<Vec<String>> {
        Ok(Self::extract_text_elements(xml)?
            .into_iter()
            .map(|text| text.text)
            .collect())
    }

    pub fn extract_text_elements(xml: &str) -> Result<Vec<ParsedText>> {
        let mut reader = Reader::from_str(xml);

        let mut texts = Vec::new();

        let mut in_shape = false;

        let mut text_value = String::new();

        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;
        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;
        let mut font_size: Option<f32> = None;
        let mut color: Option<String> = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();

                    if is_tag(name.as_ref(), b"sp") {
                        in_shape = true;

                        text_value.clear();

                        x = None;
                        y = None;
                        width = None;
                        height = None;
                        font_size = None;
                        color = None;
                    }

                    if !in_shape {
                        continue;
                    }

                    if is_tag(name.as_ref(), b"off") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref())
                                .parse::<i64>()
                                .unwrap_or(0);

                            match key.as_ref() {
                                "x" => x = Some(emu_to_pt(value)),
                                "y" => y = Some(emu_to_pt(value)),
                                _ => {}
                            }
                        }
                    }

                    if is_tag(name.as_ref(), b"ext") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref())
                                .parse::<i64>()
                                .unwrap_or(0);

                            match key.as_ref() {
                                "cx" => width = Some(emu_to_pt(value)),
                                "cy" => height = Some(emu_to_pt(value)),
                                _ => {}
                            }
                        }
                    }

                    if is_tag(name.as_ref(), b"rPr") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            if key.as_ref() != "sz" {
                                continue;
                            }

                            let value = String::from_utf8_lossy(attr.value.as_ref())
                                .parse::<f32>()
                                .unwrap_or(1800.0);

                            font_size = Some(value / 100.0);
                        }
                    }

                    if is_tag(name.as_ref(), b"srgbClr") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            if key.as_ref() != "val" {
                                continue;
                            }

                            let value = String::from_utf8_lossy(attr.value.as_ref()).to_string();

                            color = Some(format!("#{value}"));
                        }
                    }

                    if is_tag(name.as_ref(), b"t") {
                        if let Ok(Event::Text(text)) = reader.read_event() {
                            let value = String::from_utf8_lossy(text.as_ref()).to_string();

                            if !text_value.is_empty() {
                                text_value.push('\n');
                            }

                            text_value.push_str(&value);
                        }
                    }
                }

                Ok(Event::End(ref e)) => {
                    if is_tag(e.name().as_ref(), b"sp") {
                        if !text_value.is_empty() {
                            texts.push(ParsedText {
                                text: text_value.clone(),
                                bounds: Rect {
                                    x: x.unwrap_or(0.0),
                                    y: y.unwrap_or(0.0),
                                    width: width.unwrap_or(100.0),
                                    height: height.unwrap_or(30.0),
                                },
                                font_size: font_size.unwrap_or(18.0),
                                color: color.clone().unwrap_or_else(|| "#000000".to_string()),
                            });
                        }

                        in_shape = false;
                    }
                }

                Ok(Event::Eof) => break,

                Err(e) => {
                    panic!("xml parse error: {e}");
                }

                _ => {}
            }
        }

        Ok(texts)
    }

    pub fn extract_images(xml: &str) -> Result<Vec<ParsedImage>> {
        let mut reader = Reader::from_str(xml);

        let mut images = Vec::new();

        let mut in_picture = false;

        let mut relationship_id: Option<String> = None;
        let mut alt: Option<String> = None;

        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;
        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();

                    if is_tag(name.as_ref(), b"pic") {
                        in_picture = true;

                        relationship_id = None;
                        alt = None;
                        x = None;
                        y = None;
                        width = None;
                        height = None;
                    }

                    if !in_picture {
                        continue;
                    }

                    if is_tag(name.as_ref(), b"cNvPr") {
                        let mut name_value: Option<String> = None;
                        let mut descr_value: Option<String> = None;

                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref()).to_string();

                            match key.as_ref() {
                                "name" => name_value = Some(value),
                                "descr" => descr_value = Some(value),
                                _ => {}
                            }
                        }

                        alt = descr_value.or(name_value);
                    }

                    if is_tag(name.as_ref(), b"blip") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            if key.ends_with("embed") {
                                relationship_id =
                                    Some(String::from_utf8_lossy(attr.value.as_ref()).to_string());
                            }
                        }
                    }

                    if is_tag(name.as_ref(), b"off") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref())
                                .parse::<i64>()
                                .unwrap_or(0);

                            match key.as_ref() {
                                "x" => x = Some(emu_to_pt(value)),
                                "y" => y = Some(emu_to_pt(value)),
                                _ => {}
                            }
                        }
                    }

                    if is_tag(name.as_ref(), b"ext") {
                        for attr in e.attributes() {
                            let attr = attr.unwrap();

                            let key = String::from_utf8_lossy(attr.key.as_ref());

                            let value = String::from_utf8_lossy(attr.value.as_ref())
                                .parse::<i64>()
                                .unwrap_or(0);

                            match key.as_ref() {
                                "cx" => width = Some(emu_to_pt(value)),
                                "cy" => height = Some(emu_to_pt(value)),
                                _ => {}
                            }
                        }
                    }
                }

                Ok(Event::End(ref e)) => {
                    if is_tag(e.name().as_ref(), b"pic") {
                        if let (
                            Some(relationship_id),
                            Some(x),
                            Some(y),
                            Some(width),
                            Some(height),
                        ) = (relationship_id.clone(), x, y, width, height)
                        {
                            images.push(ParsedImage {
                                relationship_id,
                                bounds: Rect {
                                    x,
                                    y,
                                    width,
                                    height,
                                },
                                alt: alt.clone(),
                            });
                        }

                        in_picture = false;
                    }
                }

                Ok(Event::Eof) => break,

                Err(e) => {
                    panic!("xml parse error: {e}");
                }

                _ => {}
            }
        }

        Ok(images)
    }
}

fn is_tag(name: &[u8], local_name: &[u8]) -> bool {
    if name == local_name {
        return true;
    }

    if name.len() <= local_name.len() {
        return false;
    }

    let prefix_end = name.len() - local_name.len() - 1;

    name[prefix_end] == b':' && name.ends_with(local_name)
}

fn emu_to_pt(emu: i64) -> f32 {
    emu as f32 / 12_700.0
}
