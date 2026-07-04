use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Presentation {
    pub id: Uuid,
    pub metadata: Metadata,
    pub theme: Theme,
    pub assets: Vec<Asset>,
    pub slides: Vec<Slide>,
}

impl Presentation {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: Metadata {
                title: title.into(),
                author: None,
            },
            theme: Theme::default(),
            assets: vec![],
            slides: vec![],
        }
    }

    /// Look up an asset by id. Used by exporters/validators that need to
    /// resolve an `ImageElement.asset_id` back to its declared media type.
    pub fn find_asset(&self, asset_id: Uuid) -> Option<&Asset> {
        self.assets.iter().find(|asset| asset.id == asset_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub title: String,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            background: Color::hex("#FFFFFF"),
            foreground: Color::hex("#111111"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slide {
    pub id: Uuid,
    pub name: Option<String>,
    pub size: SlideSize,
    pub elements: Vec<Element>,
}

impl Slide {
    pub fn new(name: impl Into<Option<String>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            size: SlideSize::widescreen(),
            elements: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlideSize {
    pub width: f32,
    pub height: f32,
}

impl SlideSize {
    pub fn widescreen() -> Self {
        Self {
            // Canonical DeckMaster units are points.
            // 16:9 widescreen = 13.333in x 7.5in.
            // 1 inch = 72 points.
            width: 960.0,
            height: 540.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub value: String,
}

impl Color {
    pub fn hex(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// A package-level asset record. Per the .deckpkg spec, the actual bytes
/// live at `assets/{id}.{ext}` inside the package — this struct only
/// carries the metadata `deck.json` needs to resolve and validate that
/// file, never the bytes themselves.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub id: Uuid,
    pub media_type: String,
    #[serde(default)]
    pub alt: Option<String>,
}

impl Asset {
    pub fn new(media_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            media_type: media_type.into(),
            alt: None,
        }
    }

    /// The canonical in-package file name for this asset, e.g.
    /// `3f9a1c20-....png`. Mirrors `extension_for_media_type` in
    /// deckmaster-pptx, kept here too since deckmaster-core has no
    /// dependency on deckmaster-pptx.
    pub fn file_name(&self) -> String {
        format!("{}.{}", self.id, extension_for_media_type(&self.media_type))
    }
}

pub fn extension_for_media_type(media_type: &str) -> &'static str {
    match media_type {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpeg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "application/pdf" => "pdf",
        _ => "png",
    }
}

pub fn media_type_for_extension(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "png" => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Element {
    Text(TextElement),
    Math(MathElement),
    Image(ImageElement),
    Shape(ShapeElement),
    Table(TableElement),
    Chart(ChartElement),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

/// A semantic TeX equation/formula element. `tex` is the editable source of
/// truth. Raster renderers may attach `render_asset_id` as a PNG fallback for
/// targets that cannot consume TeX directly, such as PPTX.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MathElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub tex: String,
    pub font_size: f32,
    pub color: Color,
    #[serde(default)]
    pub render_asset_id: Option<Uuid>,
}

/// An image element references a package asset by id. It never carries
/// image bytes or a data: URL directly — see docs/DECKPKG_SPEC.md §4.
///
/// `render_asset_id` is optional and exists for source assets that are not
/// directly embeddable everywhere, especially `application/pdf` images that
/// need a raster fallback for PPTX or web previews.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub asset_id: Uuid,
    #[serde(default)]
    pub render_asset_id: Option<Uuid>,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub kind: ShapeKind,
    pub fill: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub title: String,
}

impl Element {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Element::Text(_) => "Text",
            Element::Math(_) => "Math",
            Element::Image(_) => "Image",
            Element::Shape(_) => "Shape",
            Element::Table(_) => "Table",
            Element::Chart(_) => "Chart",
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Element::Text(text) => text.id,
            Element::Math(math) => math.id,
            Element::Image(image) => image.id,
            Element::Shape(shape) => shape.id,
            Element::Table(table) => table.id,
            Element::Chart(chart) => chart.id,
        }
    }
}

impl Slide {
    pub fn add_text(&mut self, text: impl Into<String>, x: f32, y: f32, width: f32, height: f32) {
        self.elements.push(Element::Text(TextElement {
            id: Uuid::new_v4(),
            bounds: Rect {
                x,
                y,
                width,
                height,
            },
            text: text.into(),
            font_size: 24.0,
            color: Color::hex("#111111"),
        }));
    }
}
