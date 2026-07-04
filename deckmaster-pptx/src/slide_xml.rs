use crate::{Package, Result};

pub struct SlideXml {
    xml: String,
}

impl SlideXml {
    pub fn load(package: &mut Package, path: &str) -> Result<Self> {
        let full_path = format!("ppt/{}", path);

        let xml = package.read_string(&full_path)?;

        Ok(Self { xml })
    }

    pub fn xml(&self) -> &str {
        &self.xml
    }
}
