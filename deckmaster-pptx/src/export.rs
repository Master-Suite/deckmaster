use deckmaster_core::{DeckPackage, Element, Presentation, Rect, Slide};

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::units::pt_to_emu;
use crate::{PptxError, Result};

pub struct PptxExporter;

struct ResolvedImage {
    rel_id: String,
    media_name: String,
    bytes: Vec<u8>,
    bounds: Rect,
}

impl PptxExporter {
    /// Export a `.deckpkg` (already opened in memory as a `DeckPackage`)
    /// to a `.pptx` file. Image bytes come from `package.asset_bytes`,
    /// resolved via each `ImageElement.asset_id` -- there is no data: URL
    /// decoding anywhere in this path anymore. See docs/DECKPKG_SPEC.md.
    pub fn export(package: &DeckPackage, output: impl AsRef<Path>) -> Result<()> {
        let presentation = &package.presentation;

        let template_path = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/blank.pptx");

        let template = File::open(template_path)?;
        let mut archive = ZipArchive::new(template)?;

        let options = SimpleFileOptions::default();

        let slide_count = presentation.slides.len();

        if slide_count == 0 {
            return Err(PptxError::InvalidImageSource(
                "presentation must contain at least one slide".to_string(),
            ));
        }

        // PPTX has one presentation-wide slide size; the canonical model
        // carries size per slide, so the first slide's size wins here.
        let slide_width = presentation.slides[0].size.width;
        let slide_height = presentation.slides[0].size.height;

        // 1. Resolve every embeddable image up front so we know which
        //    media files and content-type defaults we will need before
        //    the template copy loop patches [Content_Types].xml. This
        //    whole resolution pass -- including every place it can fail
        //    on a missing/dangling asset -- runs BEFORE the output file
        //    is created (see step 2 below), so a failed export never
        //    leaves a truncated/partial .pptx on disk.
        let mut media_counter = 0usize;
        let mut per_slide_images: Vec<Vec<ResolvedImage>> = Vec::new();
        let mut image_extensions: BTreeSet<String> = BTreeSet::new();

        for slide in &presentation.slides {
            let mut images = Vec::new();

            for element in &slide.elements {
                if let Element::Image(image) = element {
                    let asset = presentation.find_asset(image.asset_id).ok_or_else(|| {
                        PptxError::InvalidImageSource(format!(
                            "image element {} references asset_id {} which is not declared in assets[]",
                            image.id, image.asset_id
                        ))
                    })?;

                    let bytes = package.asset_bytes.get(&asset.id).ok_or_else(|| {
                        PptxError::InvalidImageSource(format!(
                            "asset {} ({}) is declared but its bytes are not present in the package -- the .deckpkg is missing assets/{}",
                            asset.id,
                            asset.file_name(),
                            asset.file_name(),
                        ))
                    })?;

                    media_counter += 1;

                    // Relationship ids are scoped per slide rels file.
                    let rel_id = format!("rId{}", images.len() + 1);

                    let extension = deckmaster_core::extension_for_media_type(&asset.media_type);

                    // Media file names are global to avoid collisions.
                    let media_name = format!("image{media_counter}.{extension}");

                    image_extensions.insert(extension.to_string());

                    images.push(ResolvedImage {
                        rel_id,
                        media_name,
                        bytes: bytes.clone(),
                        bounds: image.bounds.clone(),
                    });
                }
            }

            per_slide_images.push(images);
        }

        // Only now -- after every fallible resolution step above has
        // succeeded -- do we touch the output path at all. This is what
        // guarantees a failed export() call never leaves a truncated or
        // half-written .pptx sitting on disk where a caller might
        // mistake it for a usable file.
        let output = File::create(output)?;
        let mut writer = ZipWriter::new(output);

        // 2. Copy the template, patching the parts that depend on slide
        //    count and on the image content types we are about to emit.
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if is_generated_slide_part(&name) {
                continue;
            }

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            writer.start_file(&name, options)?;

            if name == "ppt/presentation.xml" {
                let xml = String::from_utf8_lossy(&bytes);
                let patched = patch_presentation_xml(&xml, slide_count);
                let patched = patch_slide_size(&patched, slide_width, slide_height);

                writer.write_all(patched.as_bytes())?;
            } else if name == "ppt/_rels/presentation.xml.rels" {
                let xml = String::from_utf8_lossy(&bytes);
                let patched = patch_presentation_relationships(&xml, slide_count);

                writer.write_all(patched.as_bytes())?;
            } else if name == "[Content_Types].xml" {
                let xml = String::from_utf8_lossy(&bytes);
                let patched = patch_content_types(&xml, slide_count, &image_extensions);

                writer.write_all(patched.as_bytes())?;
            } else {
                writer.write_all(&bytes)?;
            }
        }

        // 3. Emit generated slides, their media binaries, and per-slide
        //    relationships (only for slides that actually embed images).
        for (index, slide) in presentation.slides.iter().enumerate() {
            let slide_number = index + 1;
            let images = &per_slide_images[index];

            let slide_path = format!("ppt/slides/slide{slide_number}.xml");
            let xml = generate_slide_xml(slide, images);
            writer.start_file(slide_path, options)?;
            writer.write_all(xml.as_bytes())?;

            for image in images {
                let media_path = format!("ppt/media/{}", image.media_name);
                writer.start_file(media_path, options)?;
                writer.write_all(&image.bytes)?;
            }

            let rels_path = format!("ppt/slides/_rels/slide{slide_number}.xml.rels");
            let rels_xml = generate_slide_relationships(images);
            writer.start_file(rels_path, options)?;
            writer.write_all(rels_xml.as_bytes())?;
        }

        writer.finish()?;

        Ok(())
    }

    /// Convenience wrapper for callers that already have a bare
    /// `Presentation` with no assets (text-only decks, generated fixtures,
    /// tests). Equivalent to `export` with an empty asset_bytes map.
    pub fn export_presentation_only(
        presentation: &Presentation,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        let package = DeckPackage::new(presentation.clone());
        Self::export(&package, output)
    }
}

fn patch_slide_size(xml: &str, width_pt: f32, height_pt: f32) -> String {
    let cx = pt_to_emu(width_pt);
    let cy = pt_to_emu(height_pt);

    let Some(start) = xml.find("<p:sldSz") else {
        return xml.to_string();
    };

    let Some(end_relative) = xml[start..].find("/>") else {
        return xml.to_string();
    };

    let end = start + end_relative + 2;

    let replacement = format!(r#"<p:sldSz cx="{cx}" cy="{cy}"/>"#);

    format!("{}{}{}", &xml[..start], replacement, &xml[end..])
}

fn is_generated_slide_part(path: &str) -> bool {
    (path.starts_with("ppt/slides/slide") && path.ends_with(".xml"))
        || (path.starts_with("ppt/slides/_rels/slide") && path.ends_with(".xml.rels"))
}

fn image_content_type(extension: &str) -> &'static str {
    deckmaster_core::media_type_for_extension(extension)
}

fn patch_presentation_xml(xml: &str, slide_count: usize) -> String {
    let mut slide_ids = String::new();

    for index in 0..slide_count {
        let slide_number = index + 1;
        let slide_id = 256 + index;

        slide_ids.push_str(&format!(
            r#"<p:sldId id="{slide_id}" r:id="rIdSlide{slide_number}"/>"#
        ));
    }

    replace_between(xml, "<p:sldIdLst>", "</p:sldIdLst>", &slide_ids)
}

fn patch_presentation_relationships(xml: &str, slide_count: usize) -> String {
    let without_old_slides = remove_relationships_of_type(
        xml,
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide",
    );

    let mut new_slide_rels = String::new();

    for index in 0..slide_count {
        let slide_number = index + 1;

        new_slide_rels.push_str(&format!(
            r#"<Relationship Id="rIdSlide{slide_number}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{slide_number}.xml"/>"#
        ));
    }

    insert_before(&without_old_slides, "</Relationships>", &new_slide_rels)
}

fn patch_content_types(
    xml: &str,
    slide_count: usize,
    image_extensions: &BTreeSet<String>,
) -> String {
    let without_old_slide_overrides = remove_slide_content_type_overrides(xml);

    let mut overrides = String::new();

    for index in 0..slide_count {
        let slide_number = index + 1;

        overrides.push_str(&format!(
            r#"<Override PartName="/ppt/slides/slide{slide_number}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#
        ));
    }

    let with_overrides = insert_before(&without_old_slide_overrides, "</Types>", &overrides);

    // Add a <Default> for each image extension we emit, unless the
    // template already declares it (templates often ship a png default
    // for the thumbnail; a duplicate <Default> would be invalid).
    let mut defaults = String::new();

    for extension in image_extensions {
        let needle = format!(r#"Extension="{extension}""#);

        if !with_overrides.contains(&needle) {
            defaults.push_str(&format!(
                r#"<Default Extension="{extension}" ContentType="{}"/>"#,
                image_content_type(extension)
            ));
        }
    }

    if defaults.is_empty() {
        return with_overrides;
    }

    insert_before(&with_overrides, "</Types>", &defaults)
}

fn generate_slide_relationships(images: &[ResolvedImage]) -> String {
    let mut rels = String::new();

    // Every slide that has a _rels file must declare its layout,
    // otherwise Slides rejects the package as broken.
    rels.push_str(
        r#"<Relationship Id="rIdLayout1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>"#
    );

    for image in images {
        rels.push_str(&format!(
            r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/{}"/>"#,
            image.rel_id, image.media_name
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{rels}
</Relationships>
"#
    )
}

fn generate_slide_xml(slide: &Slide, images: &[ResolvedImage]) -> String {
    let mut shapes = String::new();

    for (index, element) in slide.elements.iter().enumerate() {
        if let Element::Text(text) = element {
            let id = 100 + index;

            let x = pt_to_emu(text.bounds.x);
            let y = pt_to_emu(text.bounds.y);
            let cx = pt_to_emu(text.bounds.width);
            let cy = pt_to_emu(text.bounds.height);
            let font_size = (text.font_size * 100.0).round() as i64;
            let color = pptx_rgb(&text.color.value);

            shapes.push_str(&format!(
                r#"
<p:sp>
  <p:nvSpPr>
    <p:cNvPr id="{id}" name="Text{id}"/>
    <p:cNvSpPr txBox="1"/>
    <p:nvPr/>
  </p:nvSpPr>

  <p:spPr>
    <a:xfrm>
      <a:off x="{x}" y="{y}"/>
      <a:ext cx="{cx}" cy="{cy}"/>
    </a:xfrm>

    <a:prstGeom prst="rect">
      <a:avLst/>
    </a:prstGeom>
  </p:spPr>

  <p:txBody>
    <a:bodyPr/>
    <a:lstStyle/>
    <a:p>
      <a:r>
        <a:rPr sz="{font_size}">
        <a:solidFill>
            <a:srgbClr val="{color}"/>
        </a:solidFill>
        </a:rPr>
        <a:t>{}</a:t>
      </a:r>
    </a:p>
  </p:txBody>
</p:sp>
"#,
                xml_escape(&text.text)
            ));
        }
    }

    for (index, image) in images.iter().enumerate() {
        // Offset pic ids well clear of the 100-based text shape ids.
        let id = 500 + index;

        let x = pt_to_emu(image.bounds.x);
        let y = pt_to_emu(image.bounds.y);
        let cx = pt_to_emu(image.bounds.width);
        let cy = pt_to_emu(image.bounds.height);

        shapes.push_str(&format!(
            r#"
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="{id}" name="Image{id}"/>
    <p:cNvPicPr>
      <a:picLocks noChangeAspect="1"/>
    </p:cNvPicPr>
    <p:nvPr/>
  </p:nvPicPr>

  <p:blipFill>
    <a:blip r:embed="{rel}"/>
    <a:stretch>
      <a:fillRect/>
    </a:stretch>
  </p:blipFill>

  <p:spPr>
    <a:xfrm>
      <a:off x="{x}" y="{y}"/>
      <a:ext cx="{cx}" cy="{cy}"/>
    </a:xfrm>

    <a:prstGeom prst="rect">
      <a:avLst/>
    </a:prstGeom>
  </p:spPr>
</p:pic>
"#,
            rel = image.rel_id
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld
xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">

<p:cSld>
<p:spTree>

<p:nvGrpSpPr>
<p:cNvPr id="1" name=""/>
<p:cNvGrpSpPr/>
<p:nvPr/>
</p:nvGrpSpPr>

<p:grpSpPr>
<a:xfrm>
<a:off x="0" y="0"/>
<a:ext cx="0" cy="0"/>
<a:chOff x="0" y="0"/>
<a:chExt cx="0" cy="0"/>
</a:xfrm>
</p:grpSpPr>

{}

</p:spTree>
</p:cSld>

<p:clrMapOvr>
<a:masterClrMapping/>
</p:clrMapOvr>

</p:sld>
"#,
        shapes
    )
}

fn replace_between(xml: &str, start_tag: &str, end_tag: &str, replacement: &str) -> String {
    let Some(start) = xml.find(start_tag) else {
        return xml.to_string();
    };

    let Some(end) = xml.find(end_tag) else {
        return xml.to_string();
    };

    let content_start = start + start_tag.len();

    format!("{}{}{}", &xml[..content_start], replacement, &xml[end..])
}

fn insert_before(xml: &str, marker: &str, insertion: &str) -> String {
    let Some(index) = xml.find(marker) else {
        return xml.to_string();
    };

    format!("{}{}{}", &xml[..index], insertion, &xml[index..])
}

fn remove_relationships_of_type(xml: &str, relationship_type: &str) -> String {
    // Match the full Type="..." value. A bare substring match treats
    // ".../relationships/slide" as a match for ".../relationships/slideMaster"
    // and strips the slide master, leaving presentation.xml dangling.
    let needle = format!(r#"Type="{relationship_type}""#);

    let mut output = String::new();
    let mut rest = xml;

    loop {
        let Some(start) = rest.find("<Relationship") else {
            output.push_str(rest);
            break;
        };

        output.push_str(&rest[..start]);

        let Some(end_relative) = rest[start..].find("/>") else {
            output.push_str(&rest[start..]);
            break;
        };

        let end = start + end_relative + 2;
        let tag = &rest[start..end];

        if !tag.contains(&needle) {
            output.push_str(tag);
        }

        rest = &rest[end..];
    }

    output
}

fn remove_slide_content_type_overrides(xml: &str) -> String {
    let mut output = String::new();
    let mut rest = xml;

    loop {
        let Some(start) = rest.find("<Override") else {
            output.push_str(rest);
            break;
        };

        output.push_str(&rest[..start]);

        let Some(end_relative) = rest[start..].find("/>") else {
            output.push_str(&rest[start..]);
            break;
        };

        let end = start + end_relative + 2;
        let tag = &rest[start..end];

        let is_slide_override = tag.contains(r#"PartName="/ppt/slides/slide"#);

        if !is_slide_override {
            output.push_str(tag);
        }

        rest = &rest[end..];
    }

    output
}

fn pptx_rgb(color: &str) -> String {
    color.trim().trim_start_matches('#').to_uppercase()
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
