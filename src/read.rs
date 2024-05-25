use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufRead, BufReader, ErrorKind, Read},
    path::Path,
};

use crate::Item;

// TODO: find out how to share with write.rs
const MAGIC_SIGNATURE: [u8; 3] = *b"DUI";
const VERSION: u8 = 1;
const DIR_CHILDREN_START_MARKER: u8 = b'/';
const ENTRY_SEPARATOR: u8 = 0;

pub(crate) fn read(in_path: &Path) -> Result<(String, Item), Box<dyn Error>> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create_new(false)
        .open(in_path)?;

    let mut reader = BufReader::new(file);

    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer)?;

    if buffer[0..3] != MAGIC_SIGNATURE {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::InvalidData,
            "file signature mismatch",
        )));
    }
    if buffer[3] != VERSION {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::InvalidData,
            "file version mismatch",
        )));
    }

    let mut base_bytes = Vec::new();
    reader.read_until(ENTRY_SEPARATOR, &mut base_bytes)?;
    let base_path = String::from_utf8(base_bytes)?;
    let entry = read_entry(&mut reader)?;

    drop(reader); // TODO: is this idiomatic?

    return Ok((base_path, entry));
}

fn read_entry(reader: &mut dyn BufRead) -> Result<Item, Box<dyn Error>> {
    let mut buffer = [0; 1];
    let mut name_vec = Vec::new();
    let mut is_dir = false;
    let mut children = Vec::new();
    loop {
        reader.read_exact(&mut buffer)?;
        let b = buffer[0];
        if b == ENTRY_SEPARATOR {
            break;
        }
        if b == DIR_CHILDREN_START_MARKER {
            is_dir = true;
            loop {
                let next = reader.fill_buf()?[0];
                if next == ENTRY_SEPARATOR {
                    // Dir listing done
                    reader.consume(1);
                    break;
                }
                let child = read_entry(reader)?;
                children.push(child);
            }
            break;
        }
        name_vec.push(b);
    }

    let mut size_buffer = [0; 8];
    reader.read_exact(&mut size_buffer)?;

    return Ok(Item {
        name: String::from_utf8(name_vec)?,
        size: u64::from_be_bytes(size_buffer),
        dir: is_dir,
        items: children,
    });
}
