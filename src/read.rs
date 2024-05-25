use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Result},
    path::Path,
};

use crate::Item;

// TODO: find out how to share with write.rs
const MAGIC_SIGNATURE: [u8; 3] = *b"DUI";
const VERSION: u8 = 1;
const DIR_CHILDREN_START_MARKER: u8 = b'/';
const ENTRY_SEPARATOR: u8 = 0;

pub(crate) fn read(in_path: &Path) -> Result<(String, Item)> {
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create_new(false)
        .open(in_path)?;

    let mut reader = BufReader::new(file);

    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer)?;

    if buffer[0..3] != MAGIC_SIGNATURE {
        panic!("I don't know how to return errors yet (signature mismatch)"); // TODO:
    }
    if buffer[3] != VERSION {
        panic!("I don't know how to return errors yet (version mismatch)"); // TODO:
    }

    let mut base_bytes = Vec::new();
    reader.read_until(ENTRY_SEPARATOR, &mut base_bytes)?;
    let base_path = String::from_utf8(base_bytes).unwrap(); // TODO: error handling
    let entry = read_entry(&mut reader)?;

    drop(reader); // TODO: is this idiomatic?

    return Ok((base_path, entry));
}

fn read_entry(reader: &mut dyn BufRead) -> Result<Item> {
    let mut buffer = [0; 1];
    let mut name_vec = Vec::new();
    let mut item = Item {
        name: String::new(),
        size: 0,
        dir: false,
        items: Vec::new(),
    };

    loop {
        reader.read_exact(&mut buffer)?;
        let b = buffer[0];
        if b == ENTRY_SEPARATOR {
            break;
        }
        if b == DIR_CHILDREN_START_MARKER {
            item.dir = true;
            loop {
                let next = reader.fill_buf()?[0];
                if next == ENTRY_SEPARATOR {
                    // Dir listing done
                    reader.consume(1);
                    break;
                }
                let child = read_entry(reader)?;
                item.items.push(child);
            }
            break;
        }
        name_vec.push(b);
    }

    item.name = String::from_utf8(name_vec).unwrap(); // TODO: error handling

    let mut size_buffer = [0; 8];
    reader.read_exact(&mut size_buffer)?;
    item.size = u64::from_be_bytes(size_buffer);

    return Ok(item);
}
