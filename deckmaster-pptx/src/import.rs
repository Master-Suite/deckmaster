use deckmaster_core::{
    Asset, Color, DeckPackage, Element, ImageElement, Presentation, Slide, TextElement,
};

use std::path::Path;

use crate::{
    Package, PresentationParser, PresentationXml, Relationships, Result, SlideParser, SlideXml,
};

pub struct PptxImporter;

impl PptxImporter {
    /// Import a `.pptx` file into a `DeckPackage`: a lean `deck.json`
    /// (image elements reference `asset_id`, never inline bytes) plus the
    /// raw image bytes pulled straight out of the pptx's media folder.
    /// Callers that want a `.deckpkg` on disk should call
    /// `DeckPackage::save` on the result.
    pub fn import(path: impl AsRef<Path>) -> Result<DeckPackage> {
        let mut package_zip = Package::open(path)?;

        let presentation_xml = PresentationXml::load(&mut package_zip)?;

        let slide_refs = PresentationParser::slide_relationships(presentation_xml.xml())?;

        let presentation_rels = Relationships::load_presentation_relationships(&mut package_zip)?;

        let mut presentation = Presentation::new("Imported Presentation");
        let mut asset_bytes = std::collections::BTreeMap::new();

        for slide_ref in slide_refs {
            let Some(slide_rel) = presentation_rels
                .iter()
                .find(|rel| rel.id == slide_ref.relationship_id)
            else {
                continue;
            };

            let slide_xml = SlideXml::load(&mut package_zip, &slide_rel.target)?;

            let parsed_texts = SlideParser::extract_text_elements(slide_xml.xml())?;
            let parsed_images = SlideParser::extract_images(slide_xml.xml())?;

            let slide_rels =
                Relationships::load_slide_relationships(&mut package_zip, &slide_rel.target)?;

            let mut slide = Slide::new(Some("Imported Slide".to_string()));

            for parsed_text in parsed_texts {
                slide.elements.push(Element::Text(TextElement {
                    id: uuid::Uuid::new_v4(),
                    bounds: parsed_text.bounds,
                    text: parsed_text.text,
                    font_size: parsed_text.font_size,
                    color: Color::hex(parsed_text.color),
                }));
            }

            for parsed_image in parsed_images {
                let Some(image_rel) = slide_rels
                    .iter()
                    .find(|rel| rel.id == parsed_image.relationship_id)
                else {
                    continue;
                };

                let media_path = resolve_relationship_target(&slide_rel.target, &image_rel.target);

                let image_bytes = package_zip.read_bytes(&media_path)?;

                let media_type = media_type_for_path(&media_path);

                let original_file_name = file_name(&media_path);

                let alt = parsed_image.alt.or_else(|| original_file_name.clone());

                let asset = Asset {
                    id: uuid::Uuid::new_v4(),
                    media_type: media_type.to_string(),
                    alt: alt.clone(),
                };

                asset_bytes.insert(asset.id, image_bytes);

                let asset_id = asset.id;
                presentation.assets.push(asset);

                slide.elements.push(Element::Image(ImageElement {
                    id: uuid::Uuid::new_v4(),
                    bounds: parsed_image.bounds,
                    asset_id,
                    render_asset_id: None,
                    alt,
                }));
            }

            presentation.slides.push(slide);
        }

        Ok(DeckPackage {
            presentation,
            asset_bytes,
        })
    }
}

fn resolve_relationship_target(source_part: &str, target: &str) -> String {
    if target.starts_with('/') {
        return normalize_package_path(target.trim_start_matches('/'));
    }

    let source = source_part.trim_start_matches('/');

    let source = if source.starts_with("ppt/") {
        source.to_string()
    } else {
        format!("ppt/{source}")
    };

    let source_dir = source.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("ppt");

    normalize_package_path(&format!("{source_dir}/{target}"))
}

fn normalize_package_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();

    for part in path.split('/') {
        match part {
            "" | "." => {}

            ".." => {
                parts.pop();
            }

            _ => {
                parts.push(part);
            }
        }
    }

    parts.join("/")
}

fn media_type_for_path(path: &str) -> &'static str {
    let lower = path.to_lowercase();

    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else {
        "application/octet-stream"
    }
}

fn file_name(path: &str) -> Option<String> {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
}
