use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use zip_rebuild::RebuildInfo;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dump a single archive
    DumpSingle {
        /// Input zip file
        input: PathBuf,
        /// Dumped files output directory, if not specified defaults to the original archive name without the extension
        #[clap(long, short)]
        output_dir: Option<PathBuf>,
        /// Rebuild info file to write, if not specified defaults to the original archive name + .rebuild_info.json
        #[clap(long, short)]
        rebuild_info: Option<PathBuf>,
        /// Use BLAKE3 hashes for the filenames instead of the original names
        #[clap(long)]
        hash_names: bool,
    },
    /// Dump multiple archives
    DumpMultiple {
        /// Input zip files matched using wildcards
        input: String,
        /// Dumped files output directory
        #[clap(long, short)]
        output_dir: PathBuf,
        /// Rebuild info directory
        #[clap(long, short)]
        rebuild_info: PathBuf,
    },
    /// Rebuild a single archive
    RebuildSingle {
        /// Rebuild info file to read
        rebuild_info: PathBuf,
        /// Directory with all dumped files to use, if not specified defaults to the original archive name without the
        /// extension
        #[clap(long, short)]
        input_dir: Option<PathBuf>,
        /// Output zip file, if not specified defaults to the original file name
        #[clap(long, short)]
        output: Option<PathBuf>,
        /// Cache directory, if not specified, no caching is used
        #[clap(long, short)]
        cache_dir: Option<PathBuf>,
        /// Do not delete the cache directory
        #[clap(long, short)]
        keep_cache: bool,
    },
    /// Rebuild multiple archives
    RebuildMultiple {
        /// Rebuild info directory
        rebuild_info: PathBuf,
        /// Dumped files input directory
        #[clap(long, short)]
        input_dir: PathBuf,
        /// Output directory
        #[clap(long, short)]
        output_dir: PathBuf,
        /// Cache directory, if not specified, no caching is used
        #[clap(long, short)]
        cache_dir: Option<PathBuf>,
        /// Do not delete the cache directory
        #[clap(long, short)]
        keep_cache: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::DumpSingle {
            input,
            output_dir,
            rebuild_info,
            hash_names,
        } => {
            let output_dir =
                output_dir.unwrap_or_else(|| PathBuf::from(input.file_stem().unwrap()));
            fs::create_dir_all(&output_dir)?;

            let output_file_path = rebuild_info
                .unwrap_or_else(|| PathBuf::from(format!("{}.rebuild_info.json", input.display())));
            let mut output_file = File::create(output_file_path)?;

            let rebuild_info = zip_rebuild::dump(input.clone(), output_dir, hash_names)?;

            serde_json::to_writer(&mut output_file, &rebuild_info)?;
        }
        Commands::DumpMultiple {
            input,
            output_dir,
            rebuild_info,
        } => {
            fs::create_dir_all(&output_dir)?;
            fs::create_dir_all(&rebuild_info)?;
            for file in glob::glob(&input)?.flatten() {
                let output_file_path = rebuild_info.join(format!(
                    "{}.rebuild_info.json",
                    file.file_name().unwrap().to_string_lossy()
                ));
                let mut output_file = File::create(output_file_path)?;

                let rebuild_info = zip_rebuild::dump(file, output_dir.clone(), true)?;

                serde_json::to_writer(&mut output_file, &rebuild_info)?;
            }
        }
        Commands::RebuildSingle {
            rebuild_info,
            input_dir,
            output,
            cache_dir,
            keep_cache,
        } => {
            let rebuild_info: RebuildInfo = serde_json::from_reader(File::open(rebuild_info)?)?;

            let original_filename = PathBuf::from(rebuild_info.original_filename.clone());
            let output_file = output.unwrap_or(original_filename.clone());

            let input_directory =
                input_dir.unwrap_or_else(|| PathBuf::from(original_filename.file_stem().unwrap()));

            if let Some(cache_dir) = &cache_dir {
                fs::create_dir_all(cache_dir)?;
            }

            zip_rebuild::rebuild(
                rebuild_info,
                input_directory,
                output_file,
                cache_dir.clone(),
            )?;

            if !keep_cache {
                if let Some(cache_dir) = cache_dir {
                    fs::remove_dir_all(cache_dir)?;
                }
            }
        }
        Commands::RebuildMultiple {
            rebuild_info,
            input_dir,
            output_dir,
            cache_dir,
            keep_cache,
        } => {
            if let Some(cache_dir) = &cache_dir {
                fs::create_dir_all(cache_dir)?;
            }

            fs::create_dir_all(&output_dir)?;
            for file in
                glob::glob(&format!("{}/*.rebuild_info.json", rebuild_info.display()))?.flatten()
            {
                let rebuild_info: RebuildInfo = serde_json::from_reader(File::open(file)?)?;

                let output_file = PathBuf::from(rebuild_info.original_filename.clone());

                zip_rebuild::rebuild(
                    rebuild_info,
                    input_dir.clone(),
                    output_dir.join(output_file),
                    cache_dir.clone(),
                )?;
            }

            if !keep_cache {
                if let Some(cache_dir) = &cache_dir {
                    fs::remove_dir_all(cache_dir)?;
                }
            }
        }
    }
    Ok(())
}
