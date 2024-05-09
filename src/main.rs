use std::{
    env, ffi::OsString, fs, io::Error, os::unix::fs::MetadataExt, path::Path, time::SystemTime,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("just 1 argument please");
    }
    let path = &args[1];

    let crawling_start = SystemTime::now();
    println!("Crawling {}", path);
    let r = walk(Path::new(&path)).unwrap();
    println!(
        "{} ({} MB; {} MiB); size: {} (took {:?})",
        r.total,
        r.total as f64 / 1_000_000.0,
        r.total as f64 / 1024.0 / 1024.0,
        r.size,
        SystemTime::now().duration_since(crawling_start).unwrap(),
    );
    let count_start = SystemTime::now();
    let ct = count_total(&r);
    let count_duration = SystemTime::now().duration_since(count_start).unwrap();

    println!("{} counted total (took {:?})", ct, count_duration);
}

fn count_total(i: &Item) -> u64 {
    let mut total = i.size;
    for sub in &i.items {
        total += count_total(&sub);
    }
    return total;
}

fn walk(p: &Path) -> Result<Item, Error> {
    let m = fs::symlink_metadata(p)?;
    let name: OsString = p.file_name().map(|s| s.into()).unwrap_or_default();
    let size = m.len();
    if !m.is_dir() {
        //println!("file {} {:?}", size, name);
        return Ok(Item {
            name,
            size,
            total: size,
            items: Vec::new(),
        });
    }
    let mut total: u64 = 0; // TODO: size
    return match fs::read_dir(p) {
        Ok(entries) => {
            let mut items: Vec<Item> = Vec::new();
            for entry in entries {
                let e = entry?;
                let pp = e.path();
                let item = walk(&pp);
                match item {
                    Ok(i) => {
                        total += i.total;
                        items.push(i);
                    }
                    Err(err) => {
                        eprintln!("err {:?}: {}", pp, err)
                    }
                }
            }
            //println!("{} {:?}", total, p);
            return Ok(Item {
                name,
                size: 0, // TODO: size,
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
