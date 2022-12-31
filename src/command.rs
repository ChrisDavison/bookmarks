use std::io::{stdin, stdout, Write};
use std::process::Command;

use chrono::prelude::*;

use super::{bookmarks_dir, matches_all_strings, title_to_filename};

fn path_relative_to_bookmarks(filename: &str) -> String {
    std::path::Path::new(&filename)
        .strip_prefix(bookmarks_dir())
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub fn view(query: &[String], title_query: &[String]) {
    if let Some(entries) = get_choices_from_find(query, title_query) {
        for entry in entries {
            if let Ok(contents) = std::fs::read_to_string(entry.clone()) {
                println!("{}", "-".repeat(40));
                println!("filename: {}", path_relative_to_bookmarks(&entry));
                println!();
                println!("{}", contents);
            }
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
    if let Some(entries) = get_choices_from_find(query, title_query) {
        for entry in entries {
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
}

pub fn find(query: &[String], title_query: &[String]) -> Vec<String> {
    let query: Vec<&str> = query.iter().map(|x| x.as_str()).collect();
    let filt = tagsearch::filter::Filter::new(&query, false);

    dbg!(bookmarks_dir().to_string_lossy().to_string());
    if let Ok(files) =
        tagsearch::utility::get_files(Some(bookmarks_dir().to_string_lossy().to_string()))
    {
        dbg!(&files);
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
    let mut filepath = bookmarks_dir();
    filepath.push(title_to_filename(&title).trim());
    let mut out = format!("title: {}\n", title);
    out += &format!("url: {}\n", url);
    out += &format!("date: {}\n", date);
    out += "\n";
    out += &format!("{}\n", tags);
    std::fs::write(filepath, out).expect("Couldn't write file");
}

fn display_files(files: &[String]) {
    for (i, filename) in files.iter().enumerate() {
        println!("{:-5}: {}", i, path_relative_to_bookmarks(filename));
    }
}

fn get_numbers(response: String) -> Vec<usize> {
    let mut nums = Vec::new();
    let mut current = String::new();
    for character in response.chars() {
        if !character.is_numeric() && !current.is_empty() {
            nums.push(current.parse().unwrap());
            current = String::new();
        } else {
            current.push(character);
        }
    }
    if !current.is_empty() {
        nums.push(current.parse().unwrap())
    }
    nums
}

fn get_choices_from_find(query: &[String], title_query: &[String]) -> Option<Vec<String>> {
    let files = find(query, title_query);
    if files.is_empty() {
        println!("No files matching query.");
        return None;
    }
    println!("Bookmarks matching query:");
    display_files(&files);
    println!();
    let response = get_numbers(prompt_for_string("Entry"));
    let mut choices = Vec::new();
    for num in response {
        if num < files.len() {
            choices.push(files[num].clone());
        }
    }
    Some(choices)
}
