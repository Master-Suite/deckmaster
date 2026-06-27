use crate::io::{DeckMasterError, Result};
use crate::model::{extension_for_media_type, Element, Presentation, Rect};
use crate::package::DeckPackage;

use uuid::Uuid;

pub fn move_element(
    presentation: &mut Presentation,
    slide_id: Uuid,
    element_id: Uuid,
    x: f32,
    y: f32,
) -> Result<()> {
    let element = find_element_mut(presentation, slide_id, element_id)?;

    let bounds = element_bounds_mut(element);

    bounds.x = x;
    bounds.y = y;

    Ok(())
}

pub fn resize_element(
    presentation: &mut Presentation,
    slide_id: Uuid,
    element_id: Uuid,
    width: f32,
    height: f32,
) -> Result<()> {
    let element = find_element_mut(presentation, slide_id, element_id)?;

    let bounds = element_bounds_mut(element);

    bounds.width = width;
    bounds.height = height;

    Ok(())
}

pub fn update_text(
    presentation: &mut Presentation,
    slide_id: Uuid,
    element_id: Uuid,
    text: impl Into<String>,
) -> Result<()> {
    let element = find_element_mut(presentation, slide_id, element_id)?;

    match element {
        Element::Text(text_element) => {
            text_element.text = text.into();
            Ok(())
        }

        _ => Err(DeckMasterError::Unsupported(
            "element is not text".to_string(),
        )),
    }
}

fn find_element_mut(
    presentation: &mut Presentation,
    slide_id: Uuid,
    element_id: Uuid,
) -> Result<&mut Element> {
    let slide = presentation
        .slides
        .iter_mut()
        .find(|slide| slide.id == slide_id)
        .ok_or_else(|| DeckMasterError::Unsupported(format!("slide not found: {slide_id}")))?;

    slide
        .elements
        .iter_mut()
        .find(|element| element.id() == element_id)
        .ok_or_else(|| DeckMasterError::Unsupported(format!("element not found: {element_id}")))
}

fn element_bounds_mut(element: &mut Element) -> &mut Rect {
    match element {
        Element::Text(text) => &mut text.bounds,
        Element::Image(image) => &mut image.bounds,
        Element::Shape(shape) => &mut shape.bounds,
        Element::Table(table) => &mut table.bounds,
        Element::Chart(chart) => &mut chart.bounds,
    }
}

/// One validation issue found in a package. `severity` distinguishes hard
/// failures (the package is broken) from informational notes (the
/// package is fine but worth flagging, e.g. an unused asset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}

impl ValidationIssue {
    fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
        }
    }

    fn info(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            message: message.into(),
        }
    }
}

/// Runs the flat validation checklist from docs/DECKPKG_SPEC.md §6
/// against an already-opened package. Deliberately not a generic schema
/// validator -- see the spec for why: extend this list directly rather
/// than reaching for a validation framework.
pub fn validate(package: &DeckPackage) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let presentation = &package.presentation;

    if presentation.slides.is_empty() {
        issues.push(ValidationIssue::error("presentation has no slides"));
    }

    for slide in &presentation.slides {
        if slide.size.width <= 0.0 || slide.size.height <= 0.0 {
            issues.push(ValidationIssue::error(format!(
                "slide {} has a non-positive size ({} x {})",
                slide.id, slide.size.width, slide.size.height
            )));
        }

        for element in &slide.elements {
            let bounds = match element {
                Element::Text(text) => &text.bounds,
                Element::Image(image) => &image.bounds,
                Element::Shape(shape) => &shape.bounds,
                Element::Table(table) => &table.bounds,
                Element::Chart(chart) => &chart.bounds,
            };

            if bounds.width < 0.0 || bounds.height < 0.0 {
                issues.push(ValidationIssue::error(format!(
                    "element {} on slide {} has negative bounds",
                    element.id(),
                    slide.id
                )));
            }

            if let Element::Image(image) = element {
                let asset_declared = presentation.find_asset(image.asset_id);

                match asset_declared {
                    None => {
                        issues.push(ValidationIssue::error(format!(
                            "image element {} references asset_id {} which is not declared in assets[]",
                            image.id, image.asset_id
                        )));
                    }
                    Some(asset) => {
                        if !package.asset_bytes.contains_key(&asset.id) {
                            issues.push(ValidationIssue::error(format!(
                                "asset {} ({}) is declared but missing from assets/ in the package",
                                asset.id,
                                asset.file_name()
                            )));
                        }
                    }
                }
            }
        }
    }

    // Referential integrity + file presence for every declared asset,
    // independent of whether anything currently references it.
    let mut referenced_assets = std::collections::HashSet::new();

    for slide in &presentation.slides {
        for element in &slide.elements {
            if let Element::Image(image) = element {
                referenced_assets.insert(image.asset_id);
            }
        }
    }

    for asset in &presentation.assets {
        if !package.asset_bytes.contains_key(&asset.id) {
            // Already reported above if it's referenced; only add a
            // fresh issue here if nothing referenced it yet (so we don't
            // duplicate the message for referenced-and-missing assets).
            if !referenced_assets.contains(&asset.id) {
                issues.push(ValidationIssue::error(format!(
                    "asset {} ({}) is declared in assets[] but missing from the package",
                    asset.id,
                    asset.file_name()
                )));
            }
        }

        let expected_extension = extension_for_media_type(&asset.media_type);
        let actual_file_name = asset.file_name();

        if !actual_file_name.ends_with(expected_extension) {
            issues.push(ValidationIssue::error(format!(
                "asset {} declares media_type {} but its file name {} doesn't match",
                asset.id, asset.media_type, actual_file_name
            )));
        }

        if !referenced_assets.contains(&asset.id) {
            issues.push(ValidationIssue::info(format!(
                "asset {} ({}) is declared but not referenced by any element",
                asset.id,
                asset.file_name()
            )));
        }
    }

    issues
}
