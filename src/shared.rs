use std::io;
use std::io::{Read, Write};

pub fn copy_bytes<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    size: u64,
) -> Result<(), crate::Error> {
    let mut data = reader.take(size);
    io::copy(&mut data, writer)?;
    Ok(())
}
