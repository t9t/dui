use std::{
    fs::OpenOptions,
    io::{BufWriter, Result, Write},
    path::Path,
};

use crate::Item;

const MAGIC_SIGNATURE: [u8; 3] = *b"DUI";
const VERSION: [u8; 1] = [1];
const DIR_CHILDREN_START_MARKER: [u8; 1] = [b'/'];
const ENTRY_SEPARATOR: [u8; 1] = [0];

pub(crate) fn write(out_path: &Path, base_path: &Path, entry: &Item) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)?;

    let mut writer = BufWriter::new(file);

    writer.write_all(&MAGIC_SIGNATURE)?;
    writer.write_all(&VERSION)?;
    writer.write_all(&base_path.to_str().unwrap_or("/").as_bytes())?;
    writer.write_all(&ENTRY_SEPARATOR)?;

    write_entry(&mut writer, entry)?;

    writer.flush()?;
    drop(writer); // TODO: is this idiomatic?

    return Ok(());
}

fn write_entry(writer: &mut dyn Write, entry: &Item) -> Result<()> {
    writer.write_all(&entry.name.as_bytes())?;

    if entry.dir {
        writer.write_all(&DIR_CHILDREN_START_MARKER)?;
        for child in &entry.items {
            write_entry(writer, &child)?;
        }
    }

    writer.write_all(&ENTRY_SEPARATOR)?;
    writer.write_all(&entry.size.to_be_bytes())?;
    return Ok(());
}
