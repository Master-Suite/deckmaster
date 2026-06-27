use clap::{Parser, Subcommand};
use deckmaster_core::{
    move_element as core_move_element, resize_element as core_resize_element,
    update_text as core_update_text, DeckPackage, Document, Element, Presentation, Slide,
    Severity,
};
use deckmaster_pptx::{EmbeddedJsonExporter, PptxExporter, PptxImporter};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "deckmaster")]
#[command(version = "0.2.0")]
#[command(about = "DeckMaster presentation engine CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show a human-readable summary of a .deckpkg's contents.
    Inspect { file: PathBuf },

    /// Create a new, empty .deckpkg.
    New { file: PathBuf, title: String },

    /// Run the validation checklist against a .deckpkg.
    Validate { file: PathBuf },

    /// Pack a deck.json (+ optional assets dir) into a .deckpkg.
    Pack {
        /// Path to a deck.json file.
        deck_json: PathBuf,

        /// Output .deckpkg path.
        output: PathBuf,

        /// Directory containing asset files named {asset_id}.{ext}.
        /// Defaults to a sibling "assets" directory next to deck_json.
        #[arg(long)]
        assets_dir: Option<PathBuf>,
    },

    /// Unpack a .deckpkg into a deck.json + assets/ directory on disk.
    Unpack {
        file: PathBuf,
        /// Directory to unpack into. Defaults to the .deckpkg's name
        /// without its extension.
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },

    AddSlide { file: PathBuf, title: String },

    AddText {
        file: PathBuf,
        slide: usize,
        text: String,
    },

    /// Import a .pptx into a .deckpkg.
    ImportPptx { input: PathBuf, output: PathBuf },

    /// Export a .deckpkg to .pptx.
    ExportPptx { input: PathBuf, output: PathBuf },

    /// Export a .deckpkg to a single self-contained JSON file with
    /// images inlined as data: URLs. One-way convenience export -- see
    /// docs/DECKPKG_SPEC.md. Not re-importable.
    ExportEmbeddedJson { input: PathBuf, output: PathBuf },

    MoveElement {
        file: PathBuf,
        slide_id: Uuid,
        element_id: Uuid,
        x: f32,
        y: f32,
    },

    ResizeElement {
        file: PathBuf,
        slide_id: Uuid,
        element_id: Uuid,
        width: f32,
        height: f32,
    },

    UpdateText {
        file: PathBuf,
        slide_id: Uuid,
        element_id: Uuid,
        text: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Inspect { file } => inspect(file),
        Commands::New { file, title } => create_new(file, title),
        Commands::Validate { file } => validate_command(file),
        Commands::Pack {
            deck_json,
            output,
            assets_dir,
        } => pack(deck_json, output, assets_dir),
        Commands::Unpack { file, out_dir } => unpack(file, out_dir),
        Commands::AddSlide { file, title } => add_slide(file, title),
        Commands::AddText { file, slide, text } => add_text(file, slide, text),
        Commands::ImportPptx { input, output } => import_pptx(input, output),
        Commands::ExportPptx { input, output } => export_pptx(input, output),
        Commands::ExportEmbeddedJson { input, output } => export_embedded_json(input, output),

        Commands::MoveElement {
            file,
            slide_id,
            element_id,
            x,
            y,
        } => move_element_command(file, slide_id, element_id, x, y),

        Commands::ResizeElement {
            file,
            slide_id,
            element_id,
            width,
            height,
        } => resize_element_command(file, slide_id, element_id, width, height),

        Commands::UpdateText {
            file,
            slide_id,
            element_id,
            text,
        } => update_text_command(file, slide_id, element_id, text),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn create_new(file: PathBuf, title: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut presentation = Presentation::new(title);

    let mut slide = Slide::new(Some("Slide 1".to_string()));

    slide.add_text("Welcome to DeckMaster", 100.0, 100.0, 500.0, 100.0);

    presentation.slides.push(slide);

    Document::create(&file, presentation)?;

    println!("Presentation created: {}", file.display());

    Ok(())
}

fn add_slide(file: PathBuf, title: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::open(&file)?;

    document.add_slide(title);

    document.save()?;

    println!("Slide added.");

    Ok(())
}

fn add_text(
    file: PathBuf,
    slide: usize,
    text: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if slide == 0 {
        return Err("slide numbers start at 1".into());
    }

    let mut document = Document::open(&file)?;

    document.add_text(slide - 1, text)?;

    document.save()?;

    println!("Text added.");

    Ok(())
}

fn import_pptx(input: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let package = PptxImporter::import(&input)?;

    package.save(&output)?;

    println!("PPTX imported to {}", output.display());

    Ok(())
}

fn export_pptx(input: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let package = DeckPackage::open(&input)?;

    PptxExporter::export(&package, &output)?;

    println!("PPTX exported to {}", output.display());

    Ok(())
}

fn export_embedded_json(input: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let package = DeckPackage::open(&input)?;

    EmbeddedJsonExporter::write(&package, &output)?;

    println!(
        "Embedded JSON exported to {} (one-way convenience export, not for re-import)",
        output.display()
    );

    Ok(())
}

fn pack(
    deck_json: PathBuf,
    output: PathBuf,
    assets_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(&deck_json)?;
    let presentation: Presentation = serde_json::from_str(&source)?;

    let assets_dir = assets_dir.unwrap_or_else(|| {
        deck_json
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("assets")
    });

    let mut package = DeckPackage::new(presentation);

    for asset in &package.presentation.assets {
        let asset_path = assets_dir.join(asset.file_name());

        if !asset_path.exists() {
            return Err(format!(
                "asset {} ({}) is declared in deck.json but not found at {}",
                asset.id,
                asset.file_name(),
                asset_path.display()
            )
            .into());
        }

        let bytes = fs::read(&asset_path)?;

        package.asset_bytes.insert(asset.id, bytes);
    }

    let issues = deckmaster_core::validate(&package);
    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    if !errors.is_empty() {
        for issue in &errors {
            eprintln!("error: {}", issue.message);
        }

        return Err(format!(
            "refusing to pack: {} validation error(s) found in {}",
            errors.len(),
            deck_json.display()
        )
        .into());
    }

    package.save(&output)?;

    println!("Packed {}", output.display());

    Ok(())
}

fn unpack(file: PathBuf, out_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let package = DeckPackage::open(&file)?;

    let out_dir = out_dir.unwrap_or_else(|| {
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "deck".to_string());

        PathBuf::from(stem)
    });

    fs::create_dir_all(&out_dir)?;

    let deck_json_path = out_dir.join("deck.json");
    fs::write(&deck_json_path, serde_json::to_string_pretty(&package.presentation)?)?;

    if !package.asset_bytes.is_empty() {
        let assets_dir = out_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        for asset in &package.presentation.assets {
            if let Some(bytes) = package.asset_bytes.get(&asset.id) {
                fs::write(assets_dir.join(asset.file_name()), bytes)?;
            }
        }
    }

    println!("Unpacked to {}", out_dir.display());

    Ok(())
}

fn validate_command(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let package = DeckPackage::open(&file)?;

    let issues = deckmaster_core::validate(&package);

    let errors: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();

    let infos: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Info)
        .collect();

    for issue in &errors {
        println!("ERROR: {}", issue.message);
    }

    for issue in &infos {
        println!("info: {}", issue.message);
    }

    if errors.is_empty() {
        println!("{}: valid ({} note(s))", file.display(), infos.len());
        Ok(())
    } else {
        Err(format!(
            "{}: {} error(s) found",
            file.display(),
            errors.len()
        )
        .into())
    }
}

fn inspect(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let package = DeckPackage::open(&file)?;
    let presentation = &package.presentation;

    println!("Title: {}", presentation.metadata.title);
    println!("Slides: {}", presentation.slides.len());
    println!("Assets: {}", presentation.assets.len());

    for asset in &presentation.assets {
        let status = if package.asset_bytes.contains_key(&asset.id) {
            format!("{} bytes", package.asset_bytes[&asset.id].len())
        } else {
            "MISSING from package".to_string()
        };

        println!(
            "  - Asset [{}] {} ({})",
            asset.id, asset.media_type, status
        );
    }

    for (index, slide) in presentation.slides.iter().enumerate() {
        let name = slide.name.as_deref().unwrap_or("(untitled)");

        println!();
        println!("Slide {}: {}", index + 1, name);
        println!("  ID: {}", slide.id);
        println!("  Elements: {}", slide.elements.len());

        for element in &slide.elements {
            match element {
                Element::Text(text) => {
                    println!("  - Text [{}]: {}", text.id, text.text);
                }

                Element::Image(image) => {
                    let alt = image.alt.as_deref().unwrap_or("(no alt)");

                    println!(
                        "  - Image [{}]: {} (asset_id {})",
                        image.id, alt, image.asset_id
                    );
                }

                Element::Shape(shape) => {
                    println!("  - Shape [{}]", shape.id);
                }

                Element::Table(table) => {
                    println!("  - Table [{}]", table.id);
                }

                Element::Chart(chart) => {
                    println!("  - Chart [{}]", chart.id);
                }
            }
        }
    }

    Ok(())
}

fn move_element_command(
    file: PathBuf,
    slide_id: Uuid,
    element_id: Uuid,
    x: f32,
    y: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::open(&file)?;

    core_move_element(document.presentation_mut(), slide_id, element_id, x, y)?;

    document.save()?;

    println!("Element moved.");

    Ok(())
}

fn resize_element_command(
    file: PathBuf,
    slide_id: Uuid,
    element_id: Uuid,
    width: f32,
    height: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::open(&file)?;

    core_resize_element(document.presentation_mut(), slide_id, element_id, width, height)?;

    document.save()?;

    println!("Element resized.");

    Ok(())
}

fn update_text_command(
    file: PathBuf,
    slide_id: Uuid,
    element_id: Uuid,
    text: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut document = Document::open(&file)?;

    core_update_text(document.presentation_mut(), slide_id, element_id, text)?;

    document.save()?;

    println!("Text updated.");

    Ok(())
}

