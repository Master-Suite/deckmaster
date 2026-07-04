use crate::{Package, Result};

pub struct PresentationXml {
    xml: String,
}

impl PresentationXml {
    pub fn load(package: &mut Package) -> Result<Self> {
        let xml = package.read_string("ppt/presentation.xml")?;

        Ok(Self { xml })
    }

    pub fn xml(&self) -> &str {
        &self.xml
    }
}
