#![feature(proc_macro_hygiene)]

extern crate chrono;
extern crate clap;
extern crate fs_extra;
extern crate glob;
extern crate maud;
extern crate pulldown_cmark;

use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use chrono::{Date, Local, TimeZone};
use clap::{App, Arg, SubCommand};
use fs_extra::dir::*;
use maud::{html, PreEscaped, DOCTYPE};
use pulldown_cmark::{html, Options, Parser};

struct Daily {
    day: Date<Local>,
    title: String,
    content: String,
}

impl Daily {
    fn generate_html(&self, before: Option<&Daily>, after: Option<&Daily>) -> String {
        let higlightjs = r##"<script src="//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.12.0/highlight.min.js"></script><script>hljs.initHighlightingOnLoad();</script>"##;
        let csslist = [
            "https://cdnjs.cloudflare.com/ajax/libs/normalize/7.0.0/normalize.css",
            "/static/css/layers.min.css",
            "/static/css/layers.section.min.css",
            "/static/css/index.css",
            "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.12.0/styles/hopscotch.min.css",
            "https://fonts.googleapis.com/earlyaccess/mplus1p.css",
        ];
        let title = self.day.format("%Y/%m/%d").to_string() + &" - " + &self.title;
        let markup = html! {
            (DOCTYPE)
            html lang="ja" {
                head {
                    meta chaset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    @for url in &csslist {
                        link rel="stylesheet" href=(url);
                    }
                    (PreEscaped(higlightjs))
                    title {(title)}
                }
                body{
                    div.row {
                        div.row-content.buffer {
                            div.column.twelve.top#header {
                                a href=("/") {
                                    h1.title {"Daily Bread"}
                                }
                                p { "It's alright , I remember sometimes the time we chose what to bring on the journey" }
                            }
                            div.clear {

                            }
                            div.info {
                                time {(self.day.format("%Y/%m/%d"))};
                                h1 {(self.title)};
                            }
                            div.daily {
                                (PreEscaped(&self.content))
                            }
                            footer {
                                hr.footer;
                                div.row {
                                    div.clear {
                                    }
                                    div.row-content {
                                        div.column.small-full.medium-half.large-half  {
                                            @if after.is_some() {
                                                @let link = "/".to_string() + &(after.unwrap().day.format("%Y/%m/%d").to_string()) + ".html";
                                                time {(after.unwrap().day.format("%Y/%m/%d"))}
                                                div.day {
                                                    a href=(link) {
                                                        p {(&after.unwrap().title)}
                                                    }
                                                }
                                            }
                                        }
                                        div.column.small-full.medium-half.medium-last {
                                            @if before.is_some() {
                                                @let link = "/".to_string() + &(before.unwrap().day.format("%Y/%m/%d").to_string()) + ".html";
                                                time {(before.unwrap().day.format("%Y/%m/%d"))}
                                                div.day {
                                                    a href=(link) {
                                                        p {(&before.unwrap().title)}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                p {(PreEscaped("&copy; 2017-2018 <a href=\"http://sh4869.net\">sh4869</a>") )}

                            }
                        }
                    }
                }
            }
        };
        return markup.into_string();
    }
}

fn get_title(md: &String) -> io::Result<String> {
    let v: Vec<&str> = md.split("---").collect();
    if v.len() < 2 {
        return Err(Error::new(ErrorKind::InvalidData, "title not found"));
    }
    Ok((v[1].split("title:").collect::<Vec<&str>>())[1].trim().into())
}

fn get_date(filepath: &String) -> io::Result<Date<Local>> {
    let dailystr = filepath.clone().replace(".md", "");
    let dailyv: Vec<&str> = dailystr.split(MAIN_SEPARATOR).collect();
    let y = try!(dailyv[1].parse::<i32>().map_err(|err| Error::new(ErrorKind::InvalidData, err)));
    let m = try!(dailyv[2].parse::<u32>().map_err(|err| Error::new(ErrorKind::InvalidData, err)));
    let d = try!(dailyv[3].parse::<u32>().map_err(|err| Error::new(ErrorKind::InvalidData, err)));
    let date = Local.ymd(y, m, d);
    Ok(date)
}

fn convert_markdown(md: &str) -> io::Result<String> {
    let parser = Parser::new_ext(&md, Options::all());
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    Ok(html_buf)
}

fn write_day_file(daily: &Daily, before: Option<&Daily>, after: Option<&Daily>) -> io::Result<()> {
    let destpath = "docs/".to_string() + &daily.day.format("%Y/%m/%d").to_string() + &".html";
    let parent = Path::new(&destpath).parent().unwrap();
    if parent.exists() == false {
        fs::create_dir_all(parent.to_str().unwrap())?;
    }
    let mut file = File::create(&destpath)?;
    file.write_all(daily.generate_html(before, after).as_bytes())?;
    Ok(())
}

fn parse_daily(path: &Path) -> io::Result<Daily> {
    let mut file = File::open(path)?;
    let date;
    match get_date(&path.to_str().unwrap().into()) {
        Ok(d) => date = d,
        Err(e) => {
            println!("{}", e.to_string());
            return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
        }
    }
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    // タイトルの取得
    let title;
    match get_title(&mut file_content) {
        Ok(s) => title = s,
        Err(e) => {
            return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
        }
    };
    // 中身の取得 & Markdownの変換
    let md = file_content.splitn(3, "---").collect::<Vec<&str>>()[2];
    let content;
    match convert_markdown(&md) {
        Ok(md) => content = md,
        Err(e) => {
            println!("Error: {}", e.to_string());
            return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
        }
    };
    let daily = Daily {
        content: content,
        title: title,
        day: date,
    };
    print!(">>>>> Parse {}\r", daily.day.format("%Y/%m/%d"));
    Ok(daily)
}

fn build_daily(dailies: &mut Vec<Daily>) -> io::Result<()> {
    for i in 0..dailies.len() {
        let back = if i == 0 { None } else { dailies.get(i - 1) };
        let after = dailies.get(i + 1);
        match write_day_file(&dailies[i], back, after) {
            Ok(()) => print!(">>>>> Parse {}\r", dailies[i].day.format("%Y/%m/%d")),
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
    Ok(())
}

fn build_top_page(dailies: &mut Vec<Daily>) -> io::Result<()> {
    dailies.sort_by(|a, b| b.day.cmp(&a.day));
    dailies.retain(|daily| daily.title != "SKIP");
    let csslist = [
        "https://cdnjs.cloudflare.com/ajax/libs/normalize/7.0.0/normalize.css",
        "/static/css/layers.min.css",
        "/static/css/layers.section.min.css",
        "/static/css/index.css",
        "https://fonts.googleapis.com/earlyaccess/mplus1p.css",
    ];
    let markup = html! {
        (DOCTYPE)
        html lang="ja" {
            head {
                meta chaset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                @for url in &csslist {
                    link rel="stylesheet" href=(url);
                }
                title {"Daily Bread"}
            }
            body {
                div.row {
                    div.row-content.buffer {
                        div.column.twelve.top#header {
                            a href=("/") {
                                h1.title {"Daily Bread"}
                            }
                            p { "It's alright , I remember sometimes the time we chose what to bring on the journey" }
                        }
                        div.clear {

                        }
                        @for (i,daily) in dailies.iter().enumerate() {
                            @let link = daily.day.format("%Y/%m/%d").to_string() + ".html";
                            @if i % 2 == 0 {
                                div.column.small-full.medium-half.large-half {
                                    div.day {
                                        time {(daily.day.format("%Y/%m/%d"))};
                                        div {
                                            a href=(link) {
                                                h2 {(daily.title)}
                                            }
                                        }
                                    }
                                }
                            } @else {
                                div.column.small-full.medium-half.medium-last {
                                    div.day {
                                        time {(daily.day.format("%Y/%m/%d"))};
                                        div {
                                            a href=(link) {
                                                h2 {(daily.title)}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        footer {
                            a href=("/") {"Daily Bread"}
                            p {(PreEscaped("&copy; 2017 <a href=\"http://sh4869.net\">sh4869</a>") )}
                        }
                    }
                }
            }
        }
    };
    let mut file = File::create("docs/index.html")?;
    file.write_all(markup.into_string().as_bytes())?;
    Ok(())
}

fn build() -> io::Result<()> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in glob::glob("diary/**/*.md").map_err(|err| Error::new(ErrorKind::InvalidData, err))? {
        match entry {
            Ok(path) => paths.push(path),
            Err(e) => println!("{}", e.to_string()),
        }
    }
    let mut v: Vec<Daily> = Vec::new();
    for path in paths {
        match parse_daily(path.as_path()) {
            Ok(daily) => v.push(daily),
            Err(e) => println!("\r\n{}", e),
        }
    }
    match build_daily(&mut v) {
        Ok(()) => println!("\nBuilded!"),
        Err(e) => println!("Error: {}", e.to_string()),
    }
    match build_top_page(&mut v) {
        Ok(()) => println!(">>> Build toppage"),
        Err(e) => println!("Error: {}", e.to_string()),
    }
    Ok(())
}

fn prepear_dir() -> io::Result<()> {
    if Path::new("docs/").exists() == false {
        fs::create_dir("docs/")?;
    }
    if Path::new("docs/static").exists() == false {
        fs::create_dir("docs/static")?;
    }
    Ok(())
}

fn copy_css_image() -> io::Result<()> {
    let mut options = CopyOptions::new(); //Initialize default values for CopyOptions
    options.overwrite = true;
    for entry in fs::read_dir("static")? {
        let path = entry?.path();
        match copy(path, "docs/static", &options) {
            Ok(_d) => {}
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
    Ok(())
}

fn create_diary_template(date: Date<Local>) -> io::Result<bool> {
    let path = "diary/".to_string() + &date.format("%Y/%m/%d").to_string() + &".md";;
    if !Path::new(&path).exists() {
        let parent = Path::new(&path).parent().unwrap();
        if parent.exists() == false {
            fs::create_dir_all(parent.to_str().unwrap())?;
        }
        let mut file = File::create(&path)?;
        file.write_all("---\ntitle:\n---\n".as_bytes())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn create_templates(since: Date<Local>) -> io::Result<()> {
    let mut date = since;
    while date != Local::today() {
        date = date + chrono::Duration::days(1);
        match create_diary_template(date) {
            Ok(true) => println!(">>> Create Template on {}", date.format("%Y/%m/%d")),
            Ok(false) => {},
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("Daily Generator")
        .version("0.1")
        .author("sh4869 <nobuk4869@gmail.com>")
        .about("generate daily program")
        .subcommand(
            SubCommand::with_name("new")
                .about("generate new file")
                .arg(Arg::with_name("all").short("a").help("generate all diary not created")),
        )
        .get_matches();
    
    if let Some(matches_new) = matches.subcommand_matches("new") {
        if matches_new.is_present("all") {
            match create_templates(Local::today() - chrono::Duration::days(15)) {
                Ok(()) => println!(">>> Created templates"),
                Err(e) => println!("Error: {}", e.to_string()),
            }
        } else {
            match create_diary_template(Local::today().pred()) {
                Ok(true) => println!(">>> Create date file."),
                Ok(false) => {},
                Err(e) => println!("Error: {}", e.to_string()),
            }
        }
    } else {
        match prepear_dir() {
            Ok(()) => println!(">>> Create docs directory"),
            Err(e) => println!("Error: {}", e.to_string()),
        }
        match copy_css_image() {
            Ok(()) => println!(">>> Copied css files."),
            Err(e) => println!("Error: {}", e.to_string()),
        }
        match build() {
            Ok(()) => println!(">>> All Dailies built"),
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
}
