use std::fs::File;
use std::io::Read;
use std::path::Path;

use zip::ZipArchive;

use crate::Result;

pub struct Package {
    archive: ZipArchive<File>,
}

impl Package {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;

        let archive = ZipArchive::new(file)?;

        Ok(Self { archive })
    }

    pub fn read_bytes(&mut self, path: &str) -> Result<Vec<u8>> {
        let mut file = self.archive.by_name(path)?;

        let mut bytes = Vec::new();

        file.read_to_end(&mut bytes).map_err(crate::PptxError::Io)?;

        Ok(bytes)
    }

    pub fn contains(&mut self, path: &str) -> bool {
        self.archive.by_name(path).is_ok()
    }

    pub fn read_string(&mut self, path: &str) -> Result<String> {
        let mut file = self.archive.by_name(path)?;

        let mut xml = String::new();

        file.read_to_string(&mut xml)
            .map_err(crate::PptxError::Io)?;

        Ok(xml)
    }

    pub fn file_names(&mut self) -> Vec<String> {
        let mut names = Vec::new();

        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                names.push(file.name().to_string());
            }
        }

        names
    }
}
