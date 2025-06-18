use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use md5;
use serde::{Deserialize, Serialize};
use xz2::read::{XzDecoder, XzEncoder};

use crate::dialog;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Repo {
    Directory {
        name: String,
        children: Vec<Repo>,
        dependencies: Option<Vec<String>>,
        description: Option<String>,
    },
    File {
        name: String,
        content: String,
        original_size: Option<u64>,
        original_md5: Option<String>,
        is_compressed: Option<bool>,
        is_optional: Option<bool>,
    },
    None {},
}

impl Repo {
    pub fn new(uri: &String) -> io::Result<Self> {
        let dialog = dialog::Dialog::new(format!("Loading uri {}", uri).to_string());
        dialog.start_print(dialog::Status::Unknown);

        let resolve_uri = Self::whereis(uri)?;

        if resolve_uri.starts_with("http://") || resolve_uri.starts_with("https://") {
            let response = reqwest::blocking::get(resolve_uri).unwrap();
            let bytes = response.bytes().unwrap();
            dialog.end_print(dialog::Status::Ok);
            Ok(serde_yml::from_slice(&bytes).unwrap())
        } else {
            let file = File::open(&resolve_uri).unwrap();
            let content = io::read_to_string(&file).unwrap();
            if resolve_uri.ends_with(".json") {
                dialog.end_print(dialog::Status::Ok);
                Ok(serde_json::from_str(&content)?)
            } else if resolve_uri.ends_with(".yaml") || resolve_uri.ends_with(".yml") {
                dialog.end_print(dialog::Status::Ok);

                Ok(serde_yml::from_str(&content).unwrap())
            } else {
                dialog.end_print(dialog::Status::Failed);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unsupported file type",
                ))
            }
        }
    }

    pub fn whereis(uri: &String) -> io::Result<String> {
        // 1. Check if the uri is a local file
        if uri.starts_with('/') || uri.starts_with("./") || uri.starts_with(".\\") {
            return Ok(uri.to_string());
        }

        // 2. Check if the uri is in the share directory
        let share_dir = if cfg!(windows) {
            env::var("KRAGLE_SHARE_DIR").unwrap_or_else(|_| "/usr/share/kragle".to_string())
        } else {
            env::var("APPDATA")
                .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string())
                + "\\kragle"
        };

        if Path::new(&share_dir).exists() {
            let filepath = format!("{}/{}", share_dir, uri);
            if Path::new(&filepath).exists() {
                return Ok(filepath);
            }
        }

        // 3. Assume the uri is on gihub
        Ok(format!(
            "https://github.com/Jxtopher/kragle/blob/main/kraglefile/{}.yaml?raw=true",
            &uri,
        ))
    }

    /// Converts a folder and its tree into a JSON structure.
    pub fn from_folder<P: AsRef<Path>>(
        path: P,
        is_compressed: bool,
        depth: usize,
    ) -> io::Result<Self> {
        let path = path.as_ref();
        let name = if depth == 0 {
            ".".to_string()
        } else {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        };
        let mut children = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                children.push(Repo::from_folder(&entry_path, is_compressed, depth + 1)?);
            } else {
                // Read file content for md5 and contents
                let mut file = File::open(&entry_path)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;

                let original_md5 = format!("{:x}", md5::compute(&buf));
                let original_size = buf.len() as u64;
                let content = if is_compressed || Self::is_binary(&entry_path) {
                    let mut xz = XzEncoder::new(&buf[..], 6);
                    let mut compressed = Vec::new();
                    xz.read_to_end(&mut compressed)?;
                    base85::encode(&compressed)
                } else {
                    String::from_utf8_lossy(&buf).to_string()
                };

                children.push(Repo::File {
                    name: entry.file_name().to_string_lossy().to_string(),
                    original_size: Some(original_size),
                    original_md5: Some(original_md5),
                    is_compressed: Some(is_compressed),
                    content,
                    is_optional: Some(false),
                });
            }
        }

        Ok(Repo::Directory {
            name,
            children,
            dependencies: None,
            description: None,
        })
    }

    pub fn get_dependency(&self, uri: &String) -> io::Result<Repo> {
        if uri.starts_with("http://") || uri.starts_with("https://") {
            let response = reqwest::blocking::get(uri).unwrap();
            let bytes = response.bytes().unwrap();
            Ok(serde_yml::from_slice(&bytes).unwrap())
        } else {
            Repo::new(uri)
        }
    }

    /// Recreates a folder and file tree from a JSON structure.
    pub fn to_folder<P: AsRef<Path>>(&self, target_path: P) -> io::Result<()> {
        match self {
            Repo::Directory {
                name,
                children,
                dependencies,
                ..
            } => {
                match dependencies {
                    None => {} // No dependencies
                    Some(dependencies) => {
                        for dependency in dependencies {
                            println!("Dependancie found: {}", dependency);
                            let dependent_repo = self.get_dependency(dependency)?;
                            dependent_repo.to_folder(target_path.as_ref())?;
                        }
                    }
                }

                let dir_path = target_path.as_ref().join(name);
                if !Path::new(&dir_path).exists() {
                    fs::create_dir_all(&dir_path)?;
                    println!("Created directory: {}", dir_path.display());
                }

                for child in children {
                    child.to_folder(&dir_path)?;
                }
            }
            Repo::File {
                name,
                is_compressed,
                content,
                original_md5,
                ..
            } => {
                let file_path = target_path.as_ref().join(name);
                let file_content = match is_compressed {
                    Some(true) => {
                        let decoded = base85::decode(content).map_err(io::Error::other)?;
                        let mut xz = XzDecoder::new(&decoded[..]);
                        let mut decompressed = Vec::new();
                        xz.read_to_end(&mut decompressed)?;
                        decompressed
                    }
                    Some(false) | None => content.as_bytes().to_vec(),
                };

                // Write as text file, assuming utf-8
                let mut f = File::create(&file_path)?;
                f.write_all(&file_content)?;

                // Check MD5
                let actual_md5 = format!("{:x}", md5::compute(&file_content));
                // if let Some(expected_md5) = original_md5.as_ref() {

                match original_md5 {
                    Some(original_md5) => {
                        if &actual_md5 == original_md5 {
                            println!("Created file: {} (MD5 verified)", file_path.display());
                        } else {
                            println!(
                                "WARNING: MD5 mismatch for {}. Expected {}, got {}",
                                file_path.display(),
                                original_md5,
                                actual_md5
                            );
                        }
                    }
                    None => {
                        println!(
                            "WARNING: MD5 not provided for {} got {}",
                            file_path.display(),
                            actual_md5
                        );
                    }
                }
                // } else {
                //     println!("Created file: {}", file_path.display());
                // }
            }
            Repo::None { .. } => {}
        }
        Ok(())
    }

    pub fn display_tree(&self, prefix: &str, last: bool) -> io::Result<()> {
        match self {
            Repo::Directory { name, children, .. } => {
                println!(
                    "{}{} \x1b[34m{}\x1b[0m",
                    prefix,
                    if last { "└──" } else { "├──" },
                    name
                );
                let new_prefix = format!("{}{}", prefix, if last { "    " } else { "│   " });
                let count = children.len();
                for (i, child) in children.iter().enumerate() {
                    child.display_tree(&new_prefix, i == count - 1)?;
                }
            }
            Repo::File { name, .. } => {
                println!("{}{} {}", prefix, if last { "└──" } else { "├──" }, name);
            }
            Repo::None { .. } => {}
        }
        Ok(())
    }

    pub fn validated<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();

        match self {
            Repo::Directory { name, children, .. } => {
                let dir_path = path.join(name);
                // Check if directory exists and is a directory
                let meta = fs::metadata(&dir_path)?;
                if !meta.is_dir() {
                    return Err(io::Error::other(format!(
                        "{} is not a directory",
                        dir_path.display()
                    )));
                }
                // Recursively validate children
                for child in children {
                    child.validated(&dir_path)?;
                }
                Ok(())
            }
            Repo::File {
                name,
                original_size,
                original_md5,
                is_optional,
                ..
            } => {
                let file_path = path.join(name);
                let meta = fs::metadata(&file_path);

                if let Err(e) = meta {
                    match is_optional {
                        Some(true) => return Ok(()), // If file is optional, skip missing files
                        Some(false) | None => {
                            println!("File {} not found: {}", file_path.display(), e);
                            return Ok(());
                        }
                    }
                }
                let meta = meta?;
                if !meta.is_file() {
                    println!("File {} is not a file", file_path.display());
                    // return Err(io::Error::new(
                    //     io::ErrorKind::Other,
                    //     format!("{} is not a file", file_path.display()),
                    // ));
                }

                match original_size {
                    Some(original_size) => {
                        // Check file size
                        if meta.len() != *original_size {
                            println!(
                                "File {} size mismatch: expected {}, found {}",
                                file_path.display(),
                                original_size,
                                meta.len()
                            );
                            // return Err(io::Error::new(
                            //     io::ErrorKind::Other,
                            //     format!(
                            //         "File {} size mismatch: expected {}, found {}",
                            //         file_path.display(),
                            //         original_size,
                            //         meta.len()
                            //     ),
                            // ));
                        }
                    }
                    None => todo!(),
                }

                // Check file md5
                match original_md5 {
                    Some(original_md5) => {
                        let file_data = fs::read(&file_path)?;
                        let computed_md5 = format!("{:x}", md5::compute(&file_data));
                        if &computed_md5 != original_md5 {
                            println!(
                                "File {} md5 mismatch: expected {}, found {}",
                                file_path.display(),
                                original_md5,
                                computed_md5
                            );
                            // return Err(io::Error::new(
                            //     io::ErrorKind::Other,
                            //     format!(
                            //         "File {} md5 mismatch: expected {}, found {}",
                            //         file_path.display(),
                            //         original_md5,
                            //         computed_md5
                            //     ),
                            // ));
                        }
                    }
                    None => todo!(),
                }

                Ok(())
            }
            Repo::None {} => Ok(()),
        }
    }

    fn is_binary(path: &PathBuf) -> bool {
        let binary_exts = [
            "exe", "bin", "png", "jpg", "jpeg", "gif", "bmp", "ico", "pdf", "zip", "rar", "tar",
            "gz", "7z", "dll", "so", "mp3", "mp4", "avi", "mov", "ogg", "wav", "flac", "class",
            "o", "obj", "ttf", "woff", "woff2",
        ];

        // Récupère l'extension en minuscule
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            let ext = ext.to_ascii_lowercase();
            return binary_exts.contains(&ext.as_str());
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::repo::Repo;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_temp_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().to_owned();
        (temp_dir, target_path)
    }

    #[test]
    fn create_file() {
        let (_, target_path) = setup_temp_dir();
        let content = "Hello, World!";
        let md5_checksum = format!("{:x}", md5::compute(content));
        let repo = Repo::File {
            name: "file.txt".to_string(),
            is_compressed: Some(false),
            content: content.to_string(),
            original_size: Some(0),
            original_md5: Some(md5_checksum),
            is_optional: Some(false),
        };

        fs::create_dir_all(&target_path).unwrap();

        repo.to_folder(&target_path).unwrap();

        assert!(fs::metadata(target_path.join("file.txt")).is_ok());
        let file_content = fs::read_to_string(target_path.join("file.txt")).unwrap();
        assert_eq!(file_content, content);

        fs::remove_dir_all(target_path).unwrap();
    }

    #[test]
    fn create_directory() {
        let (_, target_path) = setup_temp_dir();
        let repo = Repo::Directory {
            name: "dir".to_string(),
            children: vec![],
            dependencies: None,
            description: None,
        };

        fs::create_dir_all(&target_path).unwrap();

        repo.to_folder(&target_path).unwrap();

        assert!(fs::metadata(target_path.join("dir")).is_ok());

        fs::remove_dir_all(target_path).unwrap();
    }
}
