//! Reading and writing `.deckpkg` files: a zip containing `deck.json` at
//! the root and image assets under `assets/`. See docs/DECKPKG_SPEC.md.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::io::{DeckMasterError, Result};
use crate::model::Presentation;

/// An in-memory `.deckpkg`: the parsed presentation plus the raw bytes of
/// every asset it references. This is the unit the editor, CLI, and PPTX
/// exporter all pass around — nobody downstream should need to touch the
/// zip layer directly.
#[derive(Debug, Clone)]
pub struct DeckPackage {
    pub presentation: Presentation,
    /// asset id -> raw file bytes, as read from assets/{id}.{ext}
    pub asset_bytes: BTreeMap<Uuid, Vec<u8>>,
}

impl DeckPackage {
    pub fn new(presentation: Presentation) -> Self {
        Self {
            presentation,
            asset_bytes: BTreeMap::new(),
        }
    }

    /// Read a `.deckpkg` zip from disk.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let deck_json = read_zip_entry_to_string(&mut archive, "deck.json")
            .ok_or_else(|| {
                DeckMasterError::Unsupported(
                    "package is missing deck.json at its root".to_string(),
                )
            })?;

        let presentation: Presentation = serde_json::from_str(&deck_json)?;

        let mut asset_bytes = BTreeMap::new();

        for asset in &presentation.assets {
            let entry_name = format!("assets/{}", asset.file_name());

            let Some(bytes) = read_zip_entry_to_bytes(&mut archive, &entry_name) else {
                // Missing asset files are a validation concern (see
                // ops::validate), not a hard read failure -- a package
                // that's missing one image should still be openable so
                // the rest of the deck can be inspected/repaired.
                continue;
            };

            asset_bytes.insert(asset.id, bytes);
        }

        Ok(Self {
            presentation,
            asset_bytes,
        })
    }

    /// Write this package to disk as a `.deckpkg` zip.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        let deck_json = serde_json::to_string_pretty(&self.presentation)?;

        writer.start_file("deck.json", options)?;
        writer.write_all(deck_json.as_bytes())?;

        for asset in &self.presentation.assets {
            let Some(bytes) = self.asset_bytes.get(&asset.id) else {
                continue;
            };

            let entry_name = format!("assets/{}", asset.file_name());

            writer.start_file(entry_name, options)?;
            writer.write_all(bytes)?;
        }

        writer.finish()?;

        Ok(())
    }
}

fn read_zip_entry_to_string<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    Some(contents)
}

fn read_zip_entry_to_bytes<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}
