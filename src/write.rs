use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

// use main::Entry;
use crate::Item;

pub(crate) fn write(out_path: &Path, entry: &Item) -> io::Result<()> {
    println!(
        "will write to file: {:?}; entry size: {}",
        out_path,
        entry.total()
    );
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)?;

    file.write_all(b"Hello, world!\n")?;

    return Ok(());
}
