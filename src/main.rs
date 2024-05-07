use std::{ffi::OsString, fs, io::Error, path::Path};

fn main() {
    let r = walk(Path::new("/Users/thomasvk/")).unwrap();
    println!("{:?}", r.total);
}

fn walk(p: &Path) -> Result<Item, Error> {
    let m = fs::metadata(p)?;
    let name: OsString = p.file_name().map(|s| s.into()).unwrap_or_default();
    if !m.is_dir() {
        return Ok(Item {
            name,
            size: m.len(),
            total: m.len(),
            items: Vec::new(),
        });
    }
    let mut total: u64 = m.len();
    return match fs::read_dir(p) {
        Ok(entries) => {
            let mut items: Vec<Item> = Vec::new();
            for entry in entries {
                let e = entry?;
                let pp = e.path();
                let item = walk(&pp);
                match item {
                    Ok(i) => {
                        total += i.size;
                        items.push(i);
                    }
                    Err(_) => {}
                }
            }
            return Ok(Item {
                name,
                size: m.len(),
                total,
                items,
            });
        }
        Err(err) => Err(err),
    };
}

#[derive(Debug)]
struct Item {
    name: OsString,
    size: u64,
    total: u64,
    items: Vec<Item>,
}
