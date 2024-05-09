use std::{
    env, f64,
    ffi::OsString,
    fs,
    io::{self, Error, Write},
    path::Path,
    time::Instant,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("just 1 argument please");
    }
    let path = &args[1];

    let crawling_start = Instant::now();
    println!("Crawling {}", path);
    let r = walk(Path::new(&path)).unwrap();
    println!(
        "Total: {} (took {:?})",
        r.total().format_bytes(),
        Instant::now().duration_since(crawling_start),
    );

    let mut full_path = Vec::new();
    full_path.push(r.name.clone());
    let mut stack: Vec<&Item> = Vec::new();
    stack.push(&r);
    let mut print = true;
    loop {
        if print {
            let current = stack.last().unwrap();
            let total = current.total();
            println!(
                "{} {} ({} sub items)",
                stack
                    .iter()
                    .map(|i| i.name.as_str())
                    .collect::<Vec<&str>>()
                    .join("/"), // TODO: platform specific path separators
                total.format_bytes(),
                current.items.len()
            );

            let perc = |i: u64| -> f64 {
                return i as f64 / (total as f64 / 100.0);
            };

            println!("  / ..");

            for i in &current.items {
                if i.is_dir() {
                    println!(
                        "  / {} {} ({:.2}%)",
                        i.name,
                        i.total().format_bytes(),
                        perc(i.total())
                    );
                }
            }

            for i in &current.items {
                if !i.is_dir() {
                    println!(
                        "  {} {} ({:.2}%)",
                        i.name,
                        i.total().format_bytes(),
                        perc(i.total())
                    );
                }
            }
            print = false;
        }

        print!("Folder (.. to go up; Ctrl+C to exit): ");
        let _ = io::stdout().flush();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        while buf.ends_with('\n') || buf.ends_with('\r') {
            buf.pop();
        }
        if buf.is_empty() {
            break;
        } else if buf == ".." {
            if stack.len() == 1 {
                eprintln!("already at top level");
                continue;
            }
            stack.pop();
            print = true;
            continue;
        }

        let item = stack.last().unwrap();
        let mut found = false;
        for i in &item.items {
            if i.name == buf {
                found = true;
                if i.is_dir() {
                    stack.push(i);
                    print = true;
                } else {
                    eprintln!("\"{}\" is not a directory", buf);
                }
                break;
            }
        }
        if !found {
            eprintln!("directory \"{}\" not found", buf);
        }
    }
}

fn format_bytes(b: u64) -> String {
    if b >= 1_099_511_627_776 {
        return format!("{:.2}TiB", b as f64 / 1_099_511_627_776.0);
    } else if b >= 1_073_741_824 {
        return format!("{:.2}GiB", b as f64 / 1_073_741_824.0);
    } else if b >= 1_048_576 {
        return format!("{:.2}MiB", b as f64 / 1_048_576.0);
    } else if b >= 1024 {
        return format!("{:.2}KiB", b as f64 / 1024.0);
    }
    return format!("{}B", b);
}

trait WithFormatBytes {
    fn format_bytes(self) -> String;
}

impl WithFormatBytes for u64 {
    fn format_bytes(self) -> String {
        return format_bytes(self);
    }
}

fn walk(p: &Path) -> Result<Item, Error> {
    let m = fs::symlink_metadata(p)?;
    let os_name: OsString = p.file_name().map(|s| s.into()).unwrap_or_default();
    let name = os_name.into_string().unwrap_or_default();
    let size = m.len();
    if !m.is_dir() {
        //println!("file {} {:?}", size, name);
        return Ok(Item {
            name,
            size,
            items: Vec::new(),
        });
    }
    return match fs::read_dir(p) {
        Ok(entries) => {
            let mut items: Vec<Item> = Vec::new();
            for entry in entries {
                let e = entry?;
                let pp = e.path();
                let item = walk(&pp);
                match item {
                    Ok(i) => {
                        items.push(i);
                    }
                    Err(err) => {
                        eprintln!("err {:?}: {}", pp, err)
                    }
                }
            }
            items.sort_by_key(|i| i.total());
            items.reverse();
            //println!("{} {:?}", total, p);
            return Ok(Item {
                name,
                size: 0, // TODO: size,
                items,
            });
        }
        Err(err) => Err(err),
    };
}

#[derive(Debug)]
struct Item {
    name: String,
    size: u64,
    items: Vec<Item>,
}

impl Item {
    fn total(&self) -> u64 {
        let mut total = self.size;
        for i in &self.items {
            total += i.total();
        }
        return total;
    }

    fn is_dir(&self) -> bool {
        return !self.items.is_empty();
    }
}
