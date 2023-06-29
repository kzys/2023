use chrono::NaiveDate;
use pulldown_cmark::{html, Parser};
use regex::Regex;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{self, Path, PathBuf},
    result,
};

fn scan_year_dir(
    dates: &mut BTreeMap<NaiveDate, PathBuf>,
    dir: &path::Path,
    year: i32,
) -> result::Result<(), Box<dyn Error>> {
    let re = Regex::new(r"^(\d{2})-(\d{2})\.md$")?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let basename = entry.file_name();
        let s = basename.to_str().unwrap();
        let cap = re.captures(s);
        if let Some(cap) = cap {
            let month = cap.get(1).unwrap().as_str().parse::<u32>()?;
            let day = cap.get(2).unwrap().as_str().parse::<u32>()?;

            let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            dates.insert(date, entry.path());
        }
    }

    Ok(())
}

fn real_main() -> result::Result<(), Box<dyn Error>> {
    let mut dates: BTreeMap<NaiveDate, PathBuf> = BTreeMap::new();
    for entry in fs::read_dir("data")? {
        let entry = entry?;
        let basename = entry.file_name();
        let s = basename.to_str().unwrap();
        let year = s.parse::<i32>();
        if let Ok(year) = year {
            scan_year_dir(&mut dates, &entry.path(), year)?;
        }
    }

    let fw = File::create("html/index.html")?;
    let mut bw = BufWriter::new(fw);

    bw.write(b"<!doctype><meta charset=utf-8>")?;

    for (date, path) in dates.iter().rev().take(5) {
        let f = File::open(path)?;
        let mut buf_reader = BufReader::new(f);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;

        let parser = Parser::new(&content);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        bw.write(html.as_bytes())?;
    }

    Ok(())
}

fn main() {
    real_main();
}
