use crate::io::Result;
use crate::package::DeckPackage;
use crate::{Presentation, Slide};

use std::path::{Path, PathBuf};

/// A `.deckpkg` open on disk, tracking the path it was opened from so
/// `save()` can round-trip back to the same file. This is the type CLI
/// commands operate on; everything below is package-aware (it carries
/// asset bytes alongside the presentation), not just-a-JSON-file aware.
pub struct Document {
    path: PathBuf,
    package: DeckPackage,
}

impl Document {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let package = DeckPackage::open(&path)?;

        Ok(Self { path, package })
    }

    pub fn create(path: impl AsRef<Path>, presentation: Presentation) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let package = DeckPackage::new(presentation);

        let document = Self { path, package };

        document.save()?;

        Ok(document)
    }

    pub fn save(&self) -> Result<()> {
        self.package.save(&self.path)
    }

    pub fn presentation(&self) -> &Presentation {
        &self.package.presentation
    }

    pub fn presentation_mut(&mut self) -> &mut Presentation {
        &mut self.package.presentation
    }

    pub fn package(&self) -> &DeckPackage {
        &self.package
    }

    pub fn package_mut(&mut self) -> &mut DeckPackage {
        &mut self.package
    }

    pub fn find_slide(&self, slide_id: uuid::Uuid) -> Option<&Slide> {
        self.package
            .presentation
            .slides
            .iter()
            .find(|slide| slide.id == slide_id)
    }

    pub fn find_slide_mut(&mut self, slide_id: uuid::Uuid) -> Option<&mut Slide> {
        self.package
            .presentation
            .slides
            .iter_mut()
            .find(|slide| slide.id == slide_id)
    }

    pub fn add_slide(&mut self, title: impl Into<String>) {
        let slide_number = self.package.presentation.slides.len() + 1;

        let mut slide = Slide::new(Some(title.into()));

        slide.add_text(
            format!("Slide {}", slide_number),
            100.0,
            100.0,
            500.0,
            100.0,
        );

        self.package.presentation.slides.push(slide);
    }

    pub fn add_text(&mut self, slide_index: usize, text: impl Into<String>) -> Result<()> {
        let slide = self
            .package
            .presentation
            .slides
            .get_mut(slide_index)
            .ok_or_else(|| {
                crate::io::DeckMasterError::Unsupported("slide does not exist".to_string())
            })?;

        slide.add_text(text, 100.0, 200.0, 600.0, 100.0);

        Ok(())
    }
}
