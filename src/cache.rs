use anyhow::anyhow;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Retrieves the content from the cache or downloads and stores the file (in bytes)
pub fn get_uri(uri: &str) -> anyhow::Result<Vec<u8>> {
    match is_cached(uri) {
        Ok(_) => Ok(get_file(uri)?),
        Err(_) => {
            if uri.starts_with("http://") || uri.starts_with("https://") {
                let response = reqwest::blocking::get(uri)?;
                let bytes = response.bytes()?;
                create_file(uri, &bytes)?;
                Ok(bytes.to_vec())
            } else {
                Err(anyhow!("URI must start with http:// or https://"))
            }
        }
    }
}

/// Retrieves the content of the cached file (in bytes)
fn get_file(uri: &str) -> io::Result<Vec<u8>> {
    match is_cached(uri) {
        Ok(_) => {
            let filepath = uri_path(uri)?;
            let mut fd = File::open(filepath)?;
            let mut content = Vec::new();
            fd.read_to_end(&mut content)?;
            Ok(content)
        }
        Err(_) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Cache file not found",
        )),
    }
}

/// Checks if the file is in the cache and not too old
pub fn is_cached(uri: &str) -> anyhow::Result<()> {
    let filepath = uri_path(uri)?;

    let metadata = fs::metadata(&filepath)?;
    let modified_time = metadata.modified()?;

    if modified_time.elapsed()? < Duration::new(24 * 60 * 60, 0) {
        Ok(())
    } else {
        Err(anyhow!(format!(
            "File {} in cache is too old (more than 24 hours)",
            uri
        )))
    }
}

/// Creates a file in the cache from bytes
fn create_file(uri: &str, content: &[u8]) -> io::Result<()> {
    let filepath = uri_path(uri)?;
    let mut fd = File::create(&filepath)?;
    fd.write_all(content)
}

/// Removes a file from the cache
#[allow(dead_code)]
fn remove_file(uri: &str) -> io::Result<()> {
    let filepath = uri_path(uri)?;
    fs::remove_file(&filepath)
}

pub fn cache_path() -> io::Result<PathBuf> {
    let home = if cfg!(windows) {
        env::var("USERPROFILE")
    } else {
        env::var("HOME")
    };

    let mut cache_dir = match home {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "HOME environment variable not found",
            ));
        }
    };
    cache_dir.push(".cache");
    cache_dir.push("kragle");

    // Create the directory if it does not exist
    let dir_path = Path::new(cache_dir.as_path());
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }

    Ok(cache_dir)
}

fn uri_path(uri: &str) -> io::Result<PathBuf> {
    let mut cache_path = cache_path()?;
    cache_path.push(format!("{:x}", md5::compute(uri)));
    Ok(cache_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_URI: &str = "https://example.com/helloworld.txt";
    const TEST_URI_CONTENT: &[u8] = b"Hello, world!";

    #[test]
    fn test_uri_path() {
        let home = if cfg!(windows) {
            env::var("USERPROFILE")
        } else {
            env::var("HOME")
        };

        let expected_path = PathBuf::from(format!(
            "{}{}",
            home.unwrap(),
            "/.cache/kragle/51c330cea8883b5c48a58b7e9676ffe0"
        ));

        let filepath = uri_path(TEST_URI);

        match filepath {
            Err(_) => panic!("Failed to get file path"),
            Ok(filepath) => assert_eq!(filepath, expected_path),
        }
    }

    #[test]
    fn test_caching() {
        // Remove existent file in the cache
        let _ = remove_file(TEST_URI);

        // Get non-existent file in the cache
        let content = get_file(TEST_URI);
        assert!(content.is_err());

        // Create a new file in the cache
        assert!(create_file(TEST_URI, TEST_URI_CONTENT).is_ok());

        // Get existent file in the cache
        let content = get_file(TEST_URI);
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), TEST_URI_CONTENT);

        // Clean up after test
        let _ = remove_file(TEST_URI);
    }
}
