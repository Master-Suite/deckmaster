//! One-way "embedded JSON" export: a single self-contained JSON file with
//! image bytes inlined as `data:` URLs. This is **not** a canonical
//! format -- see docs/DECKPKG_SPEC.md §2/§5. It exists purely as an
//! export convenience for pasting a whole deck into one LLM message or
//! sharing a single file when zip handling isn't available. Readers never
//! need to accept this shape back in.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use deckmaster_core::DeckPackage;
use serde_json::{json, Value};

use crate::Result;

pub struct EmbeddedJsonExporter;

impl EmbeddedJsonExporter {
    /// Render a package as a single JSON document where every
    /// `ImageElement` has had its `asset_id` replaced with an inline
    /// `src` data: URL. The output deliberately does not validate against
    /// the canonical schema -- it's a one-way convenience export.
    pub fn render(package: &DeckPackage) -> Result<String> {
        let mut document = serde_json::to_value(&package.presentation)?;

        if let Some(slides) = document.get_mut("slides").and_then(Value::as_array_mut) {
            for slide in slides {
                let Some(elements) = slide.get_mut("elements").and_then(Value::as_array_mut)
                else {
                    continue;
                };

                for element in elements {
                    inline_image_element(element, package);
                }
            }
        }

        // The embedded form carries no top-level assets[] -- bytes are
        // inline on each element now, so the separate asset registry
        // would just be dead weight.
        if let Some(object) = document.as_object_mut() {
            object.remove("assets");
        }

        Ok(serde_json::to_string_pretty(&document)?)
    }

    pub fn write(package: &DeckPackage, output: impl AsRef<std::path::Path>) -> Result<()> {
        let rendered = Self::render(package)?;
        std::fs::write(output, rendered)?;
        Ok(())
    }
}

fn inline_image_element(element: &mut Value, package: &DeckPackage) {
    let Some(object) = element.as_object_mut() else {
        return;
    };

    if object.get("type").and_then(Value::as_str) != Some("Image") {
        return;
    }

    let Some(asset_id_str) = object.get("asset_id").and_then(Value::as_str) else {
        return;
    };

    let Ok(asset_id) = uuid::Uuid::parse_str(asset_id_str) else {
        return;
    };

    let Some(asset) = package.presentation.find_asset(asset_id) else {
        return;
    };

    let Some(bytes) = package.asset_bytes.get(&asset_id) else {
        return;
    };

    let data_url = format!("data:{};base64,{}", asset.media_type, BASE64.encode(bytes));

    object.remove("asset_id");
    object.insert("src".to_string(), json!(data_url));
}
