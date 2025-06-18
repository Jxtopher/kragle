use clap::Parser;
use kragle::cache::cache_path;
use kragle::dialog::{self, Dialog};
use kragle::globals::MANIFEST;
use kragle::manifest::{load_manifest, print_manifest};
use kragle::repo::Repo;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Export/import a folder structure as JSON (with optional compression)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Export a folder to a JSON file
    Export {
        /// Path to the folder to export
        folder: String,
        /// Output JSON file
        output: String,
        /// Compress file contents (a85/xz)
        #[arg(short, long)]
        compressed: bool,
    },
    /// Import a folder structure from a JSON file
    Import {
        /// Input JSON file
        input: String,
        /// Target folder to recreate
        target_folder: String,
    },
    /// List contents of directories in a tree-like format
    Tree {
        /// Input JSON file
        input: String,
    },
    /// Validated the structure of a directory from a JSON
    Validate {
        /// Input JSON file
        input: String,
        /// Target folder to validate against
        target_folder: String,
    },
    /// list kraglefile based on the manifest
    List,
    /// Purge cache
    Cache {
        #[arg(short, long)]
        /// Clear the cache completely
        clear: bool,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Export {
            folder,
            output,
            compressed,
        } => {
            let repo = Repo::from_folder(folder, *compressed, 0)?;
            let mut file = File::create(output)?;
            if output.ends_with(".json") {
                serde_json::to_writer_pretty(&mut file, &repo)?;
            } else if output.ends_with(".yaml") || output.ends_with(".yml") {
                serde_yml::to_writer(file, &repo).unwrap();
            } else {
                panic!("Unsupported file format");
            }

            writeln!(
                io::stdout(),
                "Exported folder \"{}\" to \"{}\" (compressed: {})",
                folder,
                output,
                compressed
            )?;
        }
        Commands::Import {
            input,
            target_folder,
        } => {
            let repo = Repo::new(input)?;

            if !Path::new(&target_folder).exists() {
                fs::create_dir_all(target_folder)?;
                println!("Created directory: {}", target_folder);
            }

            repo.to_folder(target_folder)?;
            writeln!(
                io::stdout(),
                "Imported structure from \"{}\" into \"{}\"",
                input,
                target_folder
            )?;
        }
        Commands::Tree { input } => {
            let repo = Repo::new(input)?;
            repo.display_tree("", true)?
        }
        Commands::Validate {
            input,
            target_folder,
        } => {
            let repo = Repo::new(input)?;
            repo.validated(target_folder)?
        }
        Commands::List => {
            let manifest = load_manifest(&MANIFEST);
            print_manifest(&manifest.unwrap())?;
        }
        Commands::Cache { clear } => {
            if *clear {
                let dialog = Dialog::new("Clearing cache".to_string());
                dialog.start_print(dialog::Status::Unknown);

                let status = match fs::remove_dir_all(cache_path()?) {
                    Ok(_) => dialog::Status::Ok,
                    Err(_) => dialog::Status::Failed,
                };

                dialog.end_print(status);
            }
        }
    }
    Ok(())
}
