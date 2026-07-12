//! Server-side TeX (Beamer) export for DeckMaster.
//!
//! Rust port of the editor's `texExport.ts`, byte-compatible in intent:
//! same slide-sized TikZ canvas, same bp coordinate convention, same
//! escaping rules. Differences from the free client export:
//!
//! - `watermark: None` produces clean, watermark-free output (Pro).
//! - `use_inter_font` loads the `inter` LaTeX package so text matches
//!   the editor's on-screen font (Tectonic fetches it automatically).
//!
//! Install: copy into `deckmaster-pptx/src/tex_export.rs`, then add to
//! `deckmaster-pptx/src/lib.rs`:
//!
//! ```rust
//! pub mod tex_export;
//! pub use tex_export::*;
//! ```

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use deckmaster_core::{Asset, DeckPackage, Element, Presentation, Slide};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::{PptxError, Result};

pub struct TexExportOptions {
    /// `Some("Made with DeckMaster --- free export")` for the free tier,
    /// `None` for Pro output.
    pub watermark: Option<String>,
    /// Load `\usepackage[sfdefault]{inter}` so PDF text matches the
    /// editor. Requires an engine that can fetch it (Tectonic does).
    pub use_inter_font: bool,
}

impl Default for TexExportOptions {
    fn default() -> Self {
        Self {
            watermark: None,
            use_inter_font: true,
        }
    }
}

pub struct TexExporter;

impl TexExporter {
    /// Export a package to a `.tex.zip`: `main.tex` at the root plus
    /// `img/{asset_id}.{ext}` for every image-referenced asset.
    ///
    /// Every fallible step (TeX rendering, asset byte resolution) runs
    /// before the output file is created, so a failed export never
    /// leaves a partial zip on disk -- same policy as PptxExporter.
    pub fn export_zip(
        package: &DeckPackage,
        output: impl AsRef<Path>,
        options: &TexExportOptions,
    ) -> Result<()> {
        let tex = render_beamer_tex(&package.presentation, options)?;
        let assets = assets_referenced_by_images(&package.presentation)?;

        let mut asset_files: Vec<(String, &[u8])> = Vec::new();

        for asset in &assets {
            let bytes = package.asset_bytes.get(&asset.id).ok_or_else(|| {
                PptxError::InvalidImageSource(format!(
                    "cannot export TeX zip: missing bytes for asset {}",
                    asset.id
                ))
            })?;

            asset_files.push((format!("img/{}", asset.file_name()), bytes));
        }

        let file = File::create(output)?;
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default();

        writer.start_file("main.tex", opts)?;
        writer.write_all(tex.as_bytes())?;

        for (name, bytes) in asset_files {
            writer.start_file(name, opts)?;
            writer.write_all(bytes)?;
        }

        writer.finish()?;

        Ok(())
    }
}

pub fn render_beamer_tex(
    presentation: &Presentation,
    options: &TexExportOptions,
) -> Result<String> {
    let Some(first_slide) = presentation.slides.first() else {
        return Err(PptxError::InvalidImageSource(
            "cannot export TeX: this deck has no slides".to_string(),
        ));
    };

    let paper_width = format_bp(first_slide.size.width);
    let paper_height = format_bp(first_slide.size.height);
    let background = hex_for_tex(&presentation.theme.background.value);

    let frames: Vec<String> = presentation
        .slides
        .iter()
        .map(|slide| render_slide(slide, presentation, options))
        .collect::<Result<Vec<_>>>()?;

    let inter = if options.use_inter_font {
        "\\usepackage[sfdefault]{inter}\n"
    } else {
        ""
    };

    let watermark_color = if options.watermark.is_some() {
        "\\definecolor{dmwatermark}{HTML}{8A8A8A}\n"
    } else {
        ""
    };

    Ok(format!(
        "% DeckMaster Beamer source export.\n\
         % Slide-sized TikZ canvas, absolute bp coordinates, no\n\
         % remember-picture overlay -- a single compile pass positions\n\
         % everything correctly.\n\
         \\documentclass{{beamer}}\n\
         \\usepackage[utf8]{{inputenc}}\n\
         \\usepackage{{tikz}}\n\
         \\usepackage{{graphicx}}\n\
         \\usepackage{{xcolor}}\n\
         \\usepackage{{amsmath}}\n\
         \\usepackage{{geometry}}\n\
         {inter}\
         \\geometry{{paperwidth={paper_width}bp, paperheight={paper_height}bp, margin=0pt}}\n\
         \\setbeamersize{{text margin left=0pt, text margin right=0pt}}\n\
         \\setbeamertemplate{{headline}}{{}}\n\
         \\setbeamertemplate{{footline}}{{}}\n\
         \\beamertemplatenavigationsymbolsempty\n\
         \\definecolor{{dmbackground}}{{HTML}}{{{background}}}\n\
         {watermark_color}\
         \\begin{{document}}\n\n\
         {frames}\n\
         \\end{{document}}\n",
        frames = frames.join("\n"),
    ))
}

fn render_slide(
    slide: &Slide,
    presentation: &Presentation,
    options: &TexExportOptions,
) -> Result<String> {
    let width = format_bp(slide.size.width);
    let height = format_bp(slide.size.height);

    let elements: String = slide
        .elements
        .iter()
        .map(|element| render_element(element, presentation))
        .collect::<Result<Vec<_>>>()?
        .join("");

    let watermark = options
        .watermark
        .as_deref()
        .map(|text| render_watermark(slide, text))
        .unwrap_or_default();

    Ok(format!(
        "\\begin{{frame}}[plain,t]\n\
         \\nointerlineskip\n\
         \\begin{{tikzpicture}}[x=1bp,y=1bp]\n\
         \\path[use as bounding box] (0,0) rectangle ({width},-{height});\n\
         \\fill[dmbackground] (0,0) rectangle ({width},-{height});\n\
         {elements}{watermark}\
         \\end{{tikzpicture}}\n\
         \\end{{frame}}\n"
    ))
}

fn render_watermark(slide: &Slide, text: &str) -> String {
    let x = format_bp(slide.size.width - 16.0);
    let y = format_bp(slide.size.height - 12.0);

    format!(
        "\\node[anchor=south east, inner sep=0pt] at ({x},-{y}) {{\
         \\fontsize{{6bp}}{{7.2bp}}\\selectfont\
         \\textcolor{{dmwatermark}}{{{}}}}};\n",
        escape_tex(text)
    )
}

fn render_element(element: &Element, presentation: &Presentation) -> Result<String> {
    match element {
        Element::Text(text) => {
            let x = format_bp(text.bounds.x);
            let y = format_bp(text.bounds.y);
            let width = format_bp(text.bounds.width);
            let font_size = format_bp(text.font_size);
            let leading = format_bp(text.font_size * 1.2);
            let color = hex_for_tex(&text.color.value);
            let content = render_text_content(&text.text);

            Ok(format!(
                "\\node[anchor=north west, inner sep=0pt, align=left, text width={width}bp] \
                 at ({x},-{y}) {{\
                 \\fontsize{{{font_size}}}{{{leading}}}\\selectfont\
                 \\textcolor[HTML]{{{color}}}{{{content}}}}};\n"
            ))
        }

        Element::Math(math) => {
            let x = format_bp(math.bounds.x);
            let y = format_bp(math.bounds.y);
            let width = format_bp(math.bounds.width);
            let font_size = format_bp(math.font_size);
            let leading = format_bp(math.font_size * 1.2);
            let color = hex_for_tex(&math.color.value);
            let content = normalize_math_tex(&math.tex);

            Ok(format!(
                "\\node[anchor=north west, inner sep=0pt, align=center, text width={width}bp] \
                 at ({x},-{y}) {{\
                 \\fontsize{{{font_size}}}{{{leading}}}\\selectfont\
                 \\textcolor[HTML]{{{color}}}{{\\ensuremath{{\\displaystyle {content}}}}}}};\n"
            ))
        }

        Element::Image(image) => {
            let asset = presentation.find_asset(image.asset_id).ok_or_else(|| {
                PptxError::InvalidImageSource(format!(
                    "cannot export TeX: image element {} references missing asset {}",
                    image.id, image.asset_id
                ))
            })?;

            // SVG can't be \includegraphics'd by stock LaTeX; fall back
            // to the raster render asset if one exists.
            let effective_asset = if asset.media_type == "image/svg+xml" {
                let render_id = image.render_asset_id.ok_or_else(|| {
                    PptxError::InvalidImageSource(format!(
                        "image element {} is SVG-backed but has no render_asset_id raster fallback for TeX export",
                        image.id
                    ))
                })?;

                presentation.find_asset(render_id).ok_or_else(|| {
                    PptxError::InvalidImageSource(format!(
                        "image element {} render_asset_id {} is not declared in assets[]",
                        image.id, render_id
                    ))
                })?
            } else {
                asset
            };

            let x = format_bp(image.bounds.x);
            let y = format_bp(image.bounds.y);
            let width = format_bp(image.bounds.width);
            let height = format_bp(image.bounds.height);

            Ok(format!(
                "\\node[anchor=north west, inner sep=0pt] at ({x},-{y}) {{\
                 \\includegraphics[width={width}bp,height={height}bp]{{img/{}}}}};\n",
                effective_asset.file_name()
            ))
        }

        Element::Shape(_) | Element::Table(_) | Element::Chart(_) => Ok(String::new()),
    }
}

/// Every asset the TeX export must ship in img/. Mirrors the element
/// resolution above: SVG-backed images ship their raster fallback.
fn assets_referenced_by_images(presentation: &Presentation) -> Result<Vec<Asset>> {
    let mut seen: BTreeSet<uuid::Uuid> = BTreeSet::new();
    let mut assets: Vec<Asset> = Vec::new();

    for slide in &presentation.slides {
        for element in &slide.elements {
            let Element::Image(image) = element else {
                continue;
            };

            let asset = presentation.find_asset(image.asset_id).ok_or_else(|| {
                PptxError::InvalidImageSource(format!(
                    "cannot export TeX zip: image element {} references missing asset {}",
                    image.id, image.asset_id
                ))
            })?;

            let effective_id = if asset.media_type == "image/svg+xml" {
                image.render_asset_id.ok_or_else(|| {
                    PptxError::InvalidImageSource(format!(
                        "image element {} is SVG-backed but has no render_asset_id for TeX export",
                        image.id
                    ))
                })?
            } else {
                asset.id
            };

            if seen.contains(&effective_id) {
                continue;
            }

            let effective_asset = presentation.find_asset(effective_id).ok_or_else(|| {
                PptxError::InvalidImageSource(format!(
                    "asset {effective_id} is referenced but not declared in assets[]"
                ))
            })?;

            seen.insert(effective_id);
            assets.push(effective_asset.clone());
        }
    }

    Ok(assets)
}

fn render_text_content(text: &str) -> String {
    let trimmed = text.trim();
    let looks_like_math =
        trimmed.len() >= 2 && trimmed.starts_with('$') && trimmed.ends_with('$');

    if looks_like_math {
        trimmed.to_string()
    } else {
        escape_tex(text)
    }
}

/// Strip `$$...$$`, `\[...\]`, or `$...$` delimiters if present --
/// writers are told to omit them, readers normalize them.
fn normalize_math_tex(tex: &str) -> String {
    let trimmed = tex.trim();

    if trimmed.len() >= 4 && trimmed.starts_with("$$") && trimmed.ends_with("$$") {
        return trimmed[2..trimmed.len() - 2].trim().to_string();
    }

    if trimmed.starts_with("\\[") && trimmed.ends_with("\\]") {
        return trimmed[2..trimmed.len() - 2].trim().to_string();
    }

    if trimmed.len() >= 2 && trimmed.starts_with('$') && trimmed.ends_with('$') {
        return trimmed[1..trimmed.len() - 1].trim().to_string();
    }

    trimmed.to_string()
}

/// Single pass over the original input. Do not rewrite as chained
/// `.replace()` calls: escaping backslash inserts braces, and later
/// brace replacement would corrupt the generated escape sequence.
fn escape_tex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\textbackslash{}"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '$' => out.push_str("\\$"),
            '&' => out.push_str("\\&"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '%' => out.push_str("\\%"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '↔' => out.push_str("\\ensuremath{\\leftrightarrow}"),
            '→' => out.push_str("\\ensuremath{\\rightarrow}"),
            '←' => out.push_str("\\ensuremath{\\leftarrow}"),
            '\n' => out.push_str("\\\\\n"),
            _ => out.push(ch),
        }
    }

    out
}

fn hex_for_tex(value: &str) -> String {
    let hex = value.trim().trim_start_matches('#').to_uppercase();

    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        hex
    } else {
        "FFFFFF".to_string()
    }
}

fn format_bp(value: f32) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }

    let formatted = format!("{value:.3}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}
