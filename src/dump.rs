use std::fs::File;
use std::io::{BufReader, Seek, Write};
use std::path::PathBuf;
use std::{fs, io};

use crate::{shared, RebuildInfo, ReinsertInfo};

pub fn dump(
    input_file: PathBuf,
    output_directory: PathBuf,
    hash_names: bool,
) -> Result<RebuildInfo, crate::Error> {
    let mut result = RebuildInfo {
        original_filename: input_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        headers: format!(
            "{}.headers",
            input_file.file_name().unwrap().to_string_lossy()
        ),
        files: vec![],
    };

    struct DumpInfo {
        offset: u64,
        size: u64,
        deflate: bool,
        name: PathBuf,
        is_dir: bool,
    }

    let mut files_to_dump = {
        let file = File::open(&input_file).unwrap();
        let mut archive = zip::ZipArchive::new(file)?;

        let file_names = archive
            .file_names()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        file_names
            .into_iter()
            .map(|file_name| {
                let file = archive.by_name(&file_name)?;
                Ok(DumpInfo {
                    offset: file.data_start(),
                    size: file.compressed_size(),
                    deflate: file.compression() == zip::CompressionMethod::Deflated,
                    name: match file.enclosed_name() {
                        None => file.mangled_name(),
                        Some(path) => PathBuf::from(path),
                    },
                    is_dir: file.is_dir(),
                })
            })
            .collect::<Result<Vec<DumpInfo>, zip::result::ZipError>>()?
            .into_iter()
            .filter(|x| !x.is_dir) // Skip all directories as they're created automatically if needed
            .collect::<Vec<DumpInfo>>()
    };
    files_to_dump.sort_unstable_by(|x, y| x.offset.cmp(&y.offset));

    let mut headers = Vec::new();

    let file = File::open(input_file).unwrap();
    let mut file = BufReader::new(file);
    files_to_dump.into_iter().try_for_each(|file_to_dump| {
        // Copy everything before the file first
        let bytes_to_copy = file_to_dump.offset - file.stream_position()?;
        shared::copy_bytes(&mut file, &mut headers, bytes_to_copy)?;

        // Dump the actual file
        if file_to_dump.deflate {
            let mut data = Vec::new();
            shared::copy_bytes(&mut file, &mut data, file_to_dump.size)?;
            let decode_result = preflate_bindings::preflate_decode(data.as_slice())?;

            let diff_file_name = if hash_names {
                blake3::hash(decode_result.preflate_diff.as_slice()).to_string()
            } else {
                format!("{}.preflate", file_to_dump.name.display())
            };
            let output_file_path = output_directory.join(&diff_file_name);
            if let Some(path) = output_file_path.parent() {
                fs::create_dir_all(path)?;
            }

            if !output_file_path.exists() {
                let mut output_file = File::create(output_file_path)?;
                output_file.write_all(decode_result.preflate_diff.as_slice())?;
            }

            let file_name = if hash_names {
                PathBuf::from(blake3::hash(decode_result.unpacked_output.as_slice()).to_string())
            } else {
                file_to_dump.name
            };

            let output_file_path = output_directory.join(&file_name);
            if let Some(path) = output_file_path.parent() {
                fs::create_dir_all(path)?;
            }
            if !output_file_path.exists() {
                let mut output_file = File::create(output_file_path)?;
                output_file.write_all(decode_result.unpacked_output.as_slice())?;
            }

            result.files.push(ReinsertInfo {
                offset: file_to_dump.offset,
                data: file_name.to_string_lossy().to_string(),
                diff: Some(diff_file_name),
                hash: if hash_names {
                    Some(blake3::hash(data.as_slice()).to_string())
                } else {
                    None
                },
            })
        } else {
            let file_name = if hash_names {
                let mut hasher = blake3::Hasher::new();
                shared::copy_bytes(&mut file, &mut hasher, file_to_dump.size)?;
                let hash = hasher.finalize().to_string();
                PathBuf::from(hash)
            } else {
                file_to_dump.name
            };

            let output_file_path = output_directory.join(&file_name);
            if !output_file_path.exists() {
                if let Some(path) = output_file_path.parent() {
                    fs::create_dir_all(path)?;
                }
                let mut output_file = File::create(output_file_path)?;
                shared::copy_bytes(&mut file, &mut output_file, file_to_dump.size)?;
            }

            result.files.push(ReinsertInfo {
                offset: file_to_dump.offset,
                data: file_name.to_string_lossy().to_string(),
                diff: None,
                hash: None,
            })
        }

        Ok::<(), crate::Error>(())
    })?;

    // Copy the central directory at the end
    io::copy(&mut file, &mut headers)?;

    if hash_names {
        result.headers = blake3::hash(headers.as_slice()).to_string();
    }

    let headers_file_path = output_directory.join(&result.headers);
    if !headers_file_path.exists() {
        let mut headers_file = File::create(headers_file_path)?;
        headers_file.write_all(headers.as_slice())?;
    }

    Ok(result)
}
