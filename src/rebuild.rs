use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, Write};
use std::path::PathBuf;

use crate::{shared, RebuildInfo};

pub fn rebuild(
    rebuild_info: RebuildInfo,
    input_directory: PathBuf,
    output_file: PathBuf,
    cache_dir: Option<PathBuf>,
) -> Result<(), crate::Error> {
    let mut output_file = File::create(output_file)?;

    let headers = File::open(input_directory.join(rebuild_info.headers))?;
    let mut headers = BufReader::new(headers);

    rebuild_info
        .files
        .into_iter()
        .try_for_each(|reinsert_info| {
            // Copy everything before the file first
            let bytes_to_copy = reinsert_info.offset - output_file.stream_position()?;
            shared::copy_bytes(&mut headers, &mut output_file, bytes_to_copy)?;

            // Reinsert the actual file
            if let Some(diff) = reinsert_info.diff {
                // Try a cached file first
                let mut cache_found = false;
                if let (Some(hash), Some(cache_dir)) = (&reinsert_info.hash, &cache_dir) {
                    let file_path = cache_dir.join(hash);
                    if file_path.exists() {
                        // Reinsert as-is from cache
                        let mut file = File::open(file_path)?;
                        io::copy(&mut file, &mut output_file)?;
                        cache_found = true;
                    }
                }
                if !cache_found {
                    // Use preflate
                    let mut file = File::open(input_directory.join(reinsert_info.data))?;
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;

                    let mut file = File::open(input_directory.join(diff))?;
                    let mut diff = Vec::new();
                    file.read_to_end(&mut diff)?;

                    let reencoded_data =
                        preflate_bindings::preflate_reencode(diff.as_slice(), data.as_slice())?;
                    output_file.write_all(reencoded_data.as_slice())?;

                    if let (Some(hash), Some(cache_dir)) = (reinsert_info.hash, &cache_dir) {
                        let mut cache_file = File::create(cache_dir.join(hash))?;
                        cache_file.write_all(reencoded_data.as_slice())?;
                    }
                }
            } else {
                // Reinsert as-is
                let mut file = File::open(input_directory.join(reinsert_info.data))?;
                io::copy(&mut file, &mut output_file)?;
            }

            Ok::<(), crate::Error>(())
        })?;

    // Copy the central directory at the end
    io::copy(&mut headers, &mut output_file)?;

    Ok(())
}
