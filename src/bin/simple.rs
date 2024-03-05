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
    /// Dump an archive
    Dump {
        /// Input zip file
        input: PathBuf,
    },
    /// Rebuild an archive
    Rebuild {
        /// Rebuild info file
        input: PathBuf,
        /// Output zip file, if not specified defaults to the original file name
        #[clap(long, short)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Dump { input } => {
            let output_dir = input.file_stem().unwrap();
            fs::create_dir_all(output_dir)?;

            let mut output_file = File::create(format!("{}.rebuild_info.json", input.display()))?;

            let rebuild_info = zip_rebuild::dump(input.clone(), PathBuf::from(output_dir), false)?;

            serde_json::to_writer(&mut output_file, &rebuild_info)?;
        }
        Commands::Rebuild { input, output } => {
            let rebuild_info: RebuildInfo = serde_json::from_reader(File::open(input)?)?;

            let original_filename = PathBuf::from(rebuild_info.original_filename.clone());
            let output_file = output.unwrap_or(original_filename.clone());
            let input_directory = original_filename.file_stem().unwrap();

            zip_rebuild::rebuild(rebuild_info, PathBuf::from(input_directory), output_file)?;
        }
    }
    Ok(())
}
