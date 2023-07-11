use chrono::{Datelike, NaiveDate};
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
    process::exit,
    result,
};

use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct Options {
    /// an optional nickname for the pilot
    #[argh(option)]
    site_url: String,

    #[argh(positional)]
    dir: String,
}

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

#[derive(Serialize, Debug)]
struct Post {
    date: String,
    id: String,
    permalink: String,
    html: String,

    updated: String,
}

#[derive(Serialize, Debug)]
struct TemplateData {
    title: String,
    posts: Vec<Post>,

    updated: String,
    atom_url: String,
}

fn create_index(
    hb: &handlebars::Handlebars,
    dates: &BTreeMap<NaiveDate, PathBuf>,
) -> result::Result<(), Box<dyn Error>> {
    let mut posts: Vec<Post> = Vec::new();

    for (date, path) in dates.iter().rev().take(5) {
        let f = File::open(path)?;
        let mut buf_reader = BufReader::new(f);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;

        let parser = Parser::new(&content);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        posts.push(Post {
            date: date.format("%Y-%m-%d").to_string(),
            permalink: date.format("%Y%m.html#d%d").to_string(),
            id: "".to_string(),
            updated: date.format("%Y-%m-%dT00:00:00Z").to_string(),
            html,
        });
    }

    let td = TemplateData {
        title: "2023.8-p.info".to_string(),
        updated: "".to_string(),
        atom_url: "".to_string(),
        posts,
    };
    let fw = File::create("html/index.html")?;
    let mut bw = BufWriter::new(fw);
    bw.write(hb.render("index", &td)?.as_bytes())?;

    Ok(())
}

fn create_atom(
    hb: &handlebars::Handlebars,
    dates: &BTreeMap<NaiveDate, PathBuf>,
    site_url: &str,
) -> result::Result<(), Box<dyn Error>> {
    let mut posts: Vec<Post> = Vec::new();

    let mut updated: Option<NaiveDate> = None;

    for (date, path) in dates.iter().rev().take(5) {
        let f = File::open(path)?;
        let mut buf_reader = BufReader::new(f);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;

        let parser = Parser::new(&content);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        if updated == None {
            updated = Some(*date);
        }

        posts.push(Post {
            date: date.format("%Y-%m-%d").to_string(),
            permalink: format!("{}/{}", site_url, date.format("%Y%m.html#d%d")),
            id: "".to_string(),
            updated: date.format("%Y-%m-%dT00:00:00Z").to_string(),
            html,
        });
    }

    let td = TemplateData {
        title: "2023.8-p.info".to_string(),
        updated: updated.unwrap().format("%Y-%m-%dT00:00:00Z").to_string(),
        atom_url: format!("{}/atom.xml", site_url),
        posts,
    };

    let fw = File::create("html/atom.xml")?;
    let mut bw = BufWriter::new(fw);
    bw.write(hb.render("atom", &td)?.as_bytes())?;

    Ok(())
}

fn make_monthly(
    hb: &handlebars::Handlebars,
    dates_to_posts: &Vec<(&NaiveDate, &PathBuf)>,
) -> result::Result<(), Box<dyn Error>> {
    let mut posts: Vec<Post> = Vec::new();

    for (date, path) in dates_to_posts {
        let f = File::open(path)?;
        let mut buf_reader = BufReader::new(f);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;

        let parser = Parser::new(&content);
        let mut html = String::new();
        html::push_html(&mut html, parser);

        posts.push(Post {
            date: date.format("%Y-%m-%d").to_string(),
            permalink: "".to_string(),
            id: date.format("d%d").to_string(),
            updated: "".to_string(),
            html,
        });
    }

    let td = TemplateData {
        title: "2023.8-p.info".to_string(),
        updated: "".to_string(),
        atom_url: "".to_string(),
        posts,
    };

    let path = dates_to_posts[0].0.format("html/%Y%m.html").to_string();
    let fw = File::create(path)?;
    let mut bw = BufWriter::new(fw);
    bw.write(hb.render("index", &td)?.as_bytes())?;

    Ok(())
}

fn register_template(
    hb: &mut Handlebars,
    name: &str,
    path: &str,
) -> result::Result<(), Box<dyn Error>> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    hb.register_template_string(name, content)?;

    Ok(())
}

fn real_main(opts: &Options) -> result::Result<(), Box<dyn Error>> {
    let dates = collect_files(&opts.dir)?;

    let mut handlebars = Handlebars::new();

    register_template(&mut handlebars, "index", "data/index.html")?;
    register_template(&mut handlebars, "atom", "data/atom.xml")?;

    create_index(&handlebars, &dates)?;
    create_atom(&handlebars, &dates, &opts.site_url)?;

    let mut prev_date: Option<NaiveDate> = None;
    let mut posts: Vec<(&NaiveDate, &PathBuf)> = Vec::new();
    for (date, path) in dates.iter() {
        if let Some(prev_date) = prev_date {
            if prev_date.year() != date.year() || prev_date.month() != date.month() {
                make_monthly(&handlebars, &posts)?;
                posts.clear();
            }
        }

        posts.push((date, path));
        prev_date = Some(*date);
    }

    if posts.len() > 0 {
        make_monthly(&handlebars, &posts)?;
    }

    Ok(())
}

fn main() {
    let opts: Options = argh::from_env();

    let result = real_main(&opts);
    if let Err(e) = result {
        eprintln!("{}", e);
        exit(1);
    }
}
