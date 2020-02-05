use std::env::args;
use std::io::{Write,stdin,stdout};
use std::process::Command;

use chrono::prelude::*;

fn main() {
    let cmd: String = args().skip(1).take(1).collect();
    let query: Vec<String> = args().skip(2).collect();

    let _ = match cmd.as_ref() {
        "find" => Some(find(&query)),
        "new" => {new(); None},
        "open" => {open(&query); None},
        "view" => {view(&query); None},
        _ => {println!("Unknown cmd '{}'", cmd); None},
    };
}

fn find(query: &[String]) -> Vec<String> {
    let query: Vec<&str> = query.iter().map(|x| x.as_str()).collect();
    let filt = tagsearch::filter::Filter::new(&query, false);

    if let Ok(files) = tagsearch::utility::get_files(None) {
        match filt.files_matching_tag_query(&files) {
            Ok(files) => files,
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
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

fn view(query: &[String]) {
    if let Some(entry) = get_choice_from_find(query) {
        if let Ok(contents) = std::fs::read_to_string(entry) {
            println!();
            println!("{}", contents);
        }
    }
}

fn open(query: &[String]) {
    if let Some(entry) = get_choice_from_find(query) {
        if let Ok(contents) = std::fs::read_to_string(entry) {
            let url: String = contents.lines().filter(|x| x.starts_with("url")).map(|x| x.split(' ').nth(1).unwrap()).collect();
            if let Err(e) = Command::new("xdg-open").arg(url).status() {
                println!("{}: ", e);
            }
        }
    }
}

fn get_choice_from_find(query: &[String]) -> Option<String> {
    let files = find(query);
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
