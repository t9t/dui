use std::{
    env, f64,
    ffi::OsString,
    fs,
    io::{self, Error, Write},
    path::Path,
    time::Instant,
};

mod read;
mod write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let read_from_file: bool;
    let write_to_file: bool;
    let write_path: &Path;
    let crawl_path_arg: &str;
    if args.len() == 2 {
        read_from_file = false;
        write_to_file = false;
        write_path = Path::new("");
        crawl_path_arg = &args[1];
    } else if args.len() == 3 {
        if &args[1] != "-i" {
            return print_usage(&args);
        }
        read_from_file = true;
        write_to_file = false;
        write_path = Path::new(&args[2]);
        crawl_path_arg = &args[1];
    } else if args.len() == 4 {
        if &args[1] != "-o" {
            return print_usage(&args);
        }
        read_from_file = false;
        write_to_file = true;
        write_path = Path::new(&args[2]);
        if fs::symlink_metadata(write_path).is_ok() {
            panic!("{} already exists", &args[2]);
        }
        crawl_path_arg = &args[3];
    } else {
        return print_usage(&args);
    }

    let crawling_start = Instant::now();
    let crawl_path_path = Path::new(&crawl_path_arg);
    let r: Item;
    if read_from_file {
        let result = read::read(write_path).unwrap();
        // TODO: result.0 = base path
        r = result.1;
    } else {
        println!("Crawling {}", crawl_path_arg);
        r = walk(crawl_path_path).unwrap();
    }
    fn countall(i: &Item) -> u64 {
        let mut count = 1;
        for ii in &i.items {
            count += countall(&ii);
        }
        return count;
    }
    let count = countall(&r);
    println!(
        "Total: {} (count: {}; took {:?})",
        r.total().format_bytes(),
        count,
        Instant::now().duration_since(crawling_start),
    );

    if write_to_file {
        write::write(write_path, crawl_path_path, &r).unwrap();
        return;
    }

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
                if i.dir {
                    println!(
                        "  / {} {} ({:.2}%)",
                        i.name,
                        i.total().format_bytes(),
                        perc(i.total())
                    );
                }
            }

            for i in &current.items {
                if !i.dir {
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
                if i.dir {
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

fn print_usage(args: &Vec<String>) {
    let cmd = &args[0];
    println!("Usage:");
    println!(
        "    {} <directory>                 crawl and inspect the <directory>",
        cmd
    );
    println!("    {} -o <outfile> <directory>    crawl and inspect the <directory>, and write the report to <outfile>", cmd);
    println!(
        "    {} -i <infile>                 read the report from <infile> and inspect",
        cmd
    );
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
            dir: false,
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
                dir: true,
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
    dir: bool,
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
}
