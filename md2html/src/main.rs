use chrono::NaiveDate;
use handlebars::Handlebars;
use pulldown_cmark::{html, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
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

fn collect_files(dir: &str) -> result::Result<BTreeMap<NaiveDate, PathBuf>, Box<dyn Error>> {
    let mut dates: BTreeMap<NaiveDate, PathBuf> = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let basename = entry.file_name();
        let s = basename.to_str().unwrap();
        let year = s.parse::<i32>();
        if let Ok(year) = year {
            scan_year_dir(&mut dates, &entry.path(), year)?;
        }
    }
    Ok(dates)
}

#[derive(Serialize)]
struct Post {
    date: String,
    html: String,
}

#[derive(Serialize)]
struct TemplateData {
    title: String,
    posts: Vec<Post>,
}

fn real_main() -> result::Result<(), Box<dyn Error>> {
    let dates = collect_files("data")?;

    let mut handlebars = Handlebars::new();

    let f = File::open("data/index.html")?;
    let mut reader = BufReader::new(f);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    handlebars.register_template_string("index", content)?;

    let mut td = TemplateData {
        title: "2023.8-p.info".to_string(),
        posts: Vec::new(),
    };

    for (date, path) in dates.iter().rev().take(5) {
        let f = File::open(path)?;
        let mut buf_reader = BufReader::new(f);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;

        let parser = Parser::new(&content);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        td.posts.push(Post {
            date: date.format("%Y-%m-%d").to_string(),
            html: html,
        });
    }

    let fw = File::create("html/index.html")?;
    let mut bw = BufWriter::new(fw);
    bw.write(handlebars.render("index", &td)?.as_bytes());

    Ok(())
}

fn main() {
    let result = real_main();
    if let Err(e) = result {
        eprintln!("{}", e)
    }
}
