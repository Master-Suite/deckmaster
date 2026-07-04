use crate::io::{DeckMasterError, Result};
use crate::model::{extension_for_media_type, Element, Presentation, Rect};
use crate::package::DeckPackage;

use std::collections::HashSet;
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
        Element::Math(math) => &mut math.bounds,
        Element::Image(image) => &mut image.bounds,
        Element::Shape(shape) => &mut shape.bounds,
        Element::Table(table) => &mut table.bounds,
        Element::Chart(chart) => &mut chart.bounds,
    }
}

fn element_bounds(element: &Element) -> &Rect {
    match element {
        Element::Text(text) => &text.bounds,
        Element::Math(math) => &math.bounds,
        Element::Image(image) => &image.bounds,
        Element::Shape(shape) => &shape.bounds,
        Element::Table(table) => &table.bounds,
        Element::Chart(chart) => &chart.bounds,
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
    let mut referenced_assets = HashSet::new();
    let mut missing_asset_bytes_reported = HashSet::new();

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
            let bounds = element_bounds(element);

            if bounds.width < 0.0 || bounds.height < 0.0 {
                issues.push(ValidationIssue::error(format!(
                    "element {} on slide {} has negative bounds",
                    element.id(),
                    slide.id
                )));
            }

            match element {
                Element::Text(text) => {
                    if text.font_size <= 0.0 {
                        issues.push(ValidationIssue::error(format!(
                            "text element {} on slide {} has non-positive font_size",
                            text.id, slide.id
                        )));
                    }
                }

                Element::Math(math) => {
                    if math.tex.trim().is_empty() {
                        issues.push(ValidationIssue::error(format!(
                            "math element {} on slide {} has empty tex",
                            math.id, slide.id
                        )));
                    }

                    if math.font_size <= 0.0 {
                        issues.push(ValidationIssue::error(format!(
                            "math element {} on slide {} has non-positive font_size",
                            math.id, slide.id
                        )));
                    }

                    if let Some(render_asset_id) = math.render_asset_id {
                        referenced_assets.insert(render_asset_id);

                        validate_asset_reference(
                            package,
                            &mut issues,
                            &mut missing_asset_bytes_reported,
                            render_asset_id,
                            format!(
                                "math element {} render_asset_id {}",
                                math.id, render_asset_id
                            ),
                            true,
                        );
                    }
                }

                Element::Image(image) => {
                    referenced_assets.insert(image.asset_id);

                    validate_asset_reference(
                        package,
                        &mut issues,
                        &mut missing_asset_bytes_reported,
                        image.asset_id,
                        format!("image element {} asset_id {}", image.id, image.asset_id),
                        false,
                    );

                    if let Some(render_asset_id) = image.render_asset_id {
                        referenced_assets.insert(render_asset_id);

                        validate_asset_reference(
                            package,
                            &mut issues,
                            &mut missing_asset_bytes_reported,
                            render_asset_id,
                            format!(
                                "image element {} render_asset_id {}",
                                image.id, render_asset_id
                            ),
                            true,
                        );
                    }
                }

                Element::Shape(_) | Element::Table(_) | Element::Chart(_) => {}
            }
        }
    }

    // Referential integrity + file presence for every declared asset,
    // independent of whether anything currently references it.
    for asset in &presentation.assets {
        if !package.asset_bytes.contains_key(&asset.id)
            && !missing_asset_bytes_reported.contains(&asset.id)
        {
            issues.push(ValidationIssue::error(format!(
                "asset {} ({}) is declared in assets[] but missing from the package",
                asset.id,
                asset.file_name()
            )));
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

fn validate_asset_reference(
    package: &DeckPackage,
    issues: &mut Vec<ValidationIssue>,
    missing_asset_bytes_reported: &mut HashSet<Uuid>,
    asset_id: Uuid,
    context: String,
    require_raster_image: bool,
) {
    let Some(asset) = package.presentation.find_asset(asset_id) else {
        issues.push(ValidationIssue::error(format!(
            "{context} references asset_id {asset_id} which is not declared in assets[]"
        )));
        return;
    };

    if !package.asset_bytes.contains_key(&asset.id) {
        missing_asset_bytes_reported.insert(asset.id);
        issues.push(ValidationIssue::error(format!(
            "asset {} ({}) referenced by {context} is declared but missing from assets/ in the package",
            asset.id,
            asset.file_name()
        )));
    }

    if require_raster_image && !asset.media_type.starts_with("image/") {
        issues.push(ValidationIssue::error(format!(
            "{context} must point to a raster image asset, but asset {} has media_type {}",
            asset.id, asset.media_type
        )));
    }
}
