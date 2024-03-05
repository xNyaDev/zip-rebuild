use crate::{shared, RebuildInfo};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, Write};
use std::path::PathBuf;

pub fn rebuild(
    rebuild_info: RebuildInfo,
    input_directory: PathBuf,
    output_file: PathBuf,
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
