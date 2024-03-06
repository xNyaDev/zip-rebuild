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
        Commands::RebuildSingle {
            rebuild_info,
            input_dir,
            output,
        } => {
            let rebuild_info: RebuildInfo = serde_json::from_reader(File::open(rebuild_info)?)?;

            let original_filename = PathBuf::from(rebuild_info.original_filename.clone());
            let output_file = output.unwrap_or(original_filename.clone());

            let input_directory =
                input_dir.unwrap_or_else(|| PathBuf::from(original_filename.file_stem().unwrap()));

            zip_rebuild::rebuild(rebuild_info, input_directory, output_file)?;
        }
    }
    Ok(())
}
