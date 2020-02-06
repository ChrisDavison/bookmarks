use std::env::args;
use std::io::{Write,stdin,stdout};
use std::process::Command;

use chrono::prelude::*;

const USAGE: &str = "usage: bookmarks new|list|open|view TAGS... [--title TITLE...]";

struct BMArgs {
    cmd: String,
    tag_query: Vec<String>,
    title_query: Vec<String>,
}

fn parse_args() -> BMArgs {
    let cmd: String = args().skip(1).take(1).collect();

    let mut tag_query = Vec::new();
    let mut title_query = Vec::new();
    let mut fill_titles_now = false;
    for arg in args().skip(2) {
        if arg == "--title" {
            fill_titles_now = true;
        } else if fill_titles_now {
            title_query.push(arg);
        } else {
            tag_query.push(arg);
        }
    }

    BMArgs{cmd, tag_query, title_query}
}

fn main() {
    let args = parse_args();

    match args.cmd.as_ref() {
        "list"|"" => list(&args.tag_query, &args.title_query),
        "new" => new(),
        "open" => open(&args.tag_query, &args.title_query),
        "view" => view(&args.tag_query, &args.title_query),
        _ => println!("Unknown cmd '{}'\n\n{}", args.cmd, USAGE),
    };
}

fn matches_all_strings(s: String, ss: &[String]) -> bool {
    for substring in ss {
        if !s.contains(substring) {
            return false;
        }
    }
    true
}

fn find(query: &[String], title_query: &[String]) -> Vec<String> {
    let query: Vec<&str> = query.iter().map(|x| x.as_str()).collect();
    let filt = tagsearch::filter::Filter::new(&query, false);

    let bookmarks_dir = format!("{}/Dropbox/bookmarks/", dirs::home_dir().unwrap().to_string_lossy());
    if let Ok(files) = tagsearch::utility::get_files(Some(bookmarks_dir)) {
        match filt.files_matching_tag_query(&files) {
            Ok(files) => {
                files.iter().filter(|x| matches_all_strings((*x).to_string(), title_query))
                .map(|x| x.to_string()).collect()
            },
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

fn list(query: &[String], title_query: &[String]) {
    let files = find(query, title_query);
    if files.is_empty() {
        println!("No bookmarks matching query.");
    } else {
        println!("Bookmarks matching query:");
        for (i, filename) in files.iter().enumerate() {
            println!("{:-5}: {}", i, filename);
        }
    }
}

fn new() {
    // A bunch of read_to_string, with prompts
    // for creating a new bookmark
    let mut filename = String::new();
    print!("Filename: ");
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut filename).expect("Invalid filename");

    let mut title = String::new();
    print!("Title: ");
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut title).expect("Invalid title");

    let mut url = String::new();
    print!("URL: ");
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut url).expect("Invalid URL");

    let mut tags = String::new();
    print!("Tags: ");
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut tags).expect("Invalid tags");
    let tags: String = tags.split(' ').map(|x| "@".to_owned() + x).collect::<Vec<String>>().join(" ");

    let date = Local::today().format("%Y-%m-%d").to_string();
    let filepath = format!("{}/Dropbox/bookmarks/{}.txt", dirs::home_dir().unwrap().to_string_lossy(), filename.trim());
    let mut out = format!("title: {}\n", title.trim());
    out += &format!("url: {}\n", url.trim());
    out += &format!("date: {}\n", date.trim());
    out += "\n";
    out += &format!("{}\n", tags.trim());
    std::fs::write(filepath, out).expect("Couldn't write file");
}

fn view(query: &[String], title_query: &[String]) {
    if let Some(entry) = get_choice_from_find(query, title_query) {
        if let Ok(contents) = std::fs::read_to_string(entry) {
            println!();
            println!("{}", contents);
        }
    }
}

fn open(query: &[String], title_query: &[String]) {
    if let Some(entry) = get_choice_from_find(query, title_query) {
        if let Ok(contents) = std::fs::read_to_string(entry) {
            let url: String = contents.lines().filter(|x| x.starts_with("url")).map(|x| x.split(' ').nth(1).unwrap()).collect();
            if let Err(e) = Command::new("xdg-open").arg(url).status() {
                println!("{}: ", e);
            }
        }
    }
}

fn get_choice_from_find(query: &[String], title_query: &[String]) -> Option<String> {
    let files = find(query, title_query);
    let mut response = String::new();
    if files.is_empty() {
        println!("No files matching query.");
        return None;
    }
    println!("Bookmarks matching query:");
    for (i, filename) in files.iter().enumerate() {
        println!("{:-5}: {}", i, filename);
    }
    println!();
    print!("Entry: ");
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut response).expect("Not a correct string");
    let entry: usize = response.trim_end().parse().expect("Failed to parse entry choice");
    if entry < files.len() {
        Some(files[entry].clone())
    } else {
        None
    }
}
