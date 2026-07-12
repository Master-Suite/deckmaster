pub mod embedded_json;
pub mod export;
pub mod import;
pub mod package;
pub mod presentation_parser;
pub mod presentation_xml;
pub mod relationships;
pub mod slide_parser;
pub mod slide_xml;
pub mod tex_export;
pub mod units;

use thiserror::Error;

pub use embedded_json::*;
pub use export::*;
pub use import::*;
pub use package::*;
pub use presentation_parser::*;
pub use presentation_xml::*;
pub use relationships::*;
pub use slide_parser::*;
pub use slide_xml::*;
pub use tex_export::*;
pub use units::*;

#[derive(Debug, Error)]
pub enum PptxError {
    #[error("PPTX import is not implemented yet")]
    ImportNotImplemented,

    #[error("PPTX export is not implemented yet")]
    ExportNotImplemented,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("DeckMaster core error: {0}")]
    Core(#[from] deckmaster_core::DeckMasterError),

    #[error("Invalid image source: {0}")]
    InvalidImageSource(String),
}

pub type Result<T> = std::result::Result<T, PptxError>;
