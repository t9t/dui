use std::{
    fs::OpenOptions,
    io::{BufWriter, Result, Write},
    path::Path,
};

use crate::Item;

pub(crate) fn write(out_path: &Path, entry: &Item) -> Result<()> {
    println!(
        "will write to file: {:?}; entry size: {}",
        out_path,
        entry.total()
    );
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)?;

    let mut writer = BufWriter::new(file);
    write_entry(&mut writer, entry)?;
    writer.flush()?;

    drop(writer); // TODO: is this idiomatic?

    return Ok(());
}

fn write_entry(writer: &mut dyn EntryWriter, entry: &Item) -> Result<()> {
    let x = format!("Hello, write_entry world! {}\n", entry.total());
    writer.write_string(&x)?;
    return Ok(());
}

trait EntryWriter: Write {
    fn write_string(&mut self, s: &str) -> Result<()>;
}

impl<T> EntryWriter for T
where
    T: Write,
{
    fn write_string(&mut self, s: &str) -> Result<()> {
        return self.write_all(s.as_bytes());
    }
}
