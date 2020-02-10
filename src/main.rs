use std::env::args;

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
        if arg == "--title" || arg == "-t" {
            fill_titles_now = true;
        } else if fill_titles_now {
            title_query.push(arg);
        } else {
            tag_query.push(arg);
        }
    }

    BMArgs {
        cmd,
        tag_query,
        title_query,
    }
}

fn main() {
    let args = parse_args();

    match args.cmd.as_ref() {
        "list" | "" => command::list(&args.tag_query, &args.title_query),
        "new" => command::new(),
        "open" => command::open(&args.tag_query, &args.title_query),
        "view" => command::view(&args.tag_query, &args.title_query),
        "related" => command::related(&args.tag_query, &args.title_query),
        _ => println!("Unknown cmd '{}'\n\n{}", args.cmd, USAGE),
    };
}

fn matches_all_strings(s: &str, substrings: &[String]) -> bool {
    substrings.iter().all(|substring| s.contains(substring))
}

fn title_to_filename(t: &str) -> String {
    let mut filename = String::new();
    let mut prev = '_';
    for ch in t.chars() {
        if ch.is_alphabetic() {
            filename.push(ch);
            prev = ch;
        } else if prev != '-' {
            filename.push('-');
            prev = '-';
        }
    }
    filename
        .to_ascii_lowercase()
        .trim_end_matches('-')
        .to_string()
}

mod command {
    use std::io::{stdin, stdout, Write};
    use std::process::Command;

    use chrono::prelude::*;

    use super::{matches_all_strings, title_to_filename};

    fn path_relative_to_bookmarks(filename: &str) -> String {
        let bookmarks_dir = format!(
            "{}/Dropbox/bookmarks/",
            dirs::home_dir().unwrap().to_string_lossy()
        );
        std::path::Path::new(&filename)
            .strip_prefix(bookmarks_dir)
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    pub fn view(query: &[String], title_query: &[String]) {
        if let Some(entry) = get_choice_from_find(query, title_query) {
            if let Ok(contents) = std::fs::read_to_string(entry.clone()) {
                println!("{}", "-".repeat(40));
                println!("filename: {}", path_relative_to_bookmarks(&entry));
                println!();
                println!("{}", contents);
            }
        }
    }

    pub fn related(query: &[String], title_query: &[String]) {
        let files = find(query, title_query);
        let query: Vec<&str> = query.iter().map(|x| x.as_str()).collect();
        let filt = tagsearch::filter::Filter::new(&query, false);
        if let Ok(tags) = filt.tags_matching_tag_query(files) {
            let tags: String = tags
                .iter()
                .filter(|&x| !query.contains(&x.as_str()))
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            println!("Related to `{}` :: {}", query.join(", "), tags);
        }
    }

    pub fn open(query: &[String], title_query: &[String]) {
        if let Some(entry) = get_choice_from_find(query, title_query) {
            if let Ok(contents) = std::fs::read_to_string(entry) {
                let url: String = contents
                    .lines()
                    .filter(|x| x.starts_with("url"))
                    .map(|x| x.split(' ').nth(1).unwrap())
                    .collect();
                if let Err(e) = Command::new("xdg-open").arg(url).status() {
                    println!("{}: ", e);
                }
            }
        }
    }

    pub fn find(query: &[String], title_query: &[String]) -> Vec<String> {
        let query: Vec<&str> = query.iter().map(|x| x.as_str()).collect();
        let filt = tagsearch::filter::Filter::new(&query, false);

        let bookmarks_dir = format!(
            "{}/Dropbox/bookmarks/",
            dirs::home_dir().unwrap().to_string_lossy()
        );
        if let Ok(files) = tagsearch::utility::get_files(Some(bookmarks_dir)) {
            match filt.files_matching_tag_query(&files) {
                Ok(files) => files
                    .iter()
                    .filter(|&x| matches_all_strings(x, title_query))
                    .map(|x| x.to_string())
                    .collect(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }

    pub fn list(query: &[String], title_query: &[String]) {
        let files = find(query, title_query);
        if files.is_empty() {
            println!("No bookmarks matching query.")
        } else {
            println!("Bookmarks matching query:");
            display_files(&files);
        }
    }

    pub fn prompt_for_string(prompt: &str) -> String {
        let mut response = String::new();
        print!("{}: ", prompt);
        stdout().flush().expect("Failed to flush stdout");
        let errmsg = format!("Invalid {}", prompt);
        stdin().read_line(&mut response).expect(&errmsg);
        response.trim().to_string()
    }

    pub fn new() {
        let title = prompt_for_string("Title");
        let url = prompt_for_string("URL");
        let tags = prompt_for_string("Tags")
            .split(' ')
            .map(|x| "@".to_owned() + x)
            .collect::<Vec<String>>()
            .join(" ");

        let date = Local::today().format("%Y-%m-%d").to_string();
        let filepath = format!(
            "{}/Dropbox/bookmarks/{}.txt",
            dirs::home_dir().unwrap().to_string_lossy(),
            title_to_filename(&title).trim()
        );

        let mut out = format!("title: {}\n", title);
        out += &format!("url: {}\n", url);
        out += &format!("date: {}\n", date);
        out += "\n";
        out += &format!("{}\n", tags);
        std::fs::write(filepath, out).expect("Couldn't write file");
    }

    fn display_files(files: &[String]) {
        for (i, filename) in files.iter().enumerate() {
            println!("{:-5}: {}", i, path_relative_to_bookmarks(&filename));
        }
    }

    fn get_choice_from_find(query: &[String], title_query: &[String]) -> Option<String> {
        let files = find(query, title_query);
        if files.is_empty() {
            println!("No files matching query.");
            return None;
        }
        println!("Bookmarks matching query:");
        display_files(&files);
        println!();
        let entry: usize = prompt_for_string("Entry")
            .parse()
            .expect("Failed to parse entry choice");
        if entry < files.len() {
            Some(files[entry].clone())
        } else {
            None
        }
    }
}
