use anyhow::anyhow;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read, Write};

use crate::cache::get_uri;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    name: String,
    description: Option<String>,
}

pub fn load_manifest(uri: &str) -> anyhow::Result<Vec<Manifest>> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        let data = get_uri(uri).unwrap();
        let manifests: Vec<Manifest> =
            serde_yml::from_slice(&data).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        Ok(manifests)
    } else {
        let mut file = File::open(uri)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if uri.ends_with(".json") {
            let manifests: Vec<Manifest> = serde_json::from_str(&content)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
            Ok(manifests)
        } else if uri.ends_with(".yaml") || uri.ends_with(".yml") {
            let manifests: Vec<Manifest> =
                serde_yml::from_str(&content).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
            Ok(manifests)
        } else {
            Err(anyhow!("URI must start with http:// or https://"))
        }
    }
}

pub fn print_manifest(manifest: &[Manifest]) -> io::Result<()> {
    for entry in manifest.iter() {
        match entry.description {
            Some(ref description) => writeln!(io::stdout(), "{} - {:?}", entry.name, description)?,
            None => writeln!(io::stdout(), "{}", entry.name)?,
        };
    }
    Ok(())
}
