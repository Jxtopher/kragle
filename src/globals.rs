use once_cell::sync::Lazy;

pub static MANIFEST: Lazy<String> = Lazy::new(|| {
    std::env::var("MANIFEST").unwrap_or_else(|_| {
        "https://github.com/Jxtopher/kragle/blob/main/kraglefile/manifest.yaml?raw=true".to_string()
    })
});
