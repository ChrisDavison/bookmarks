use std::env::args;

mod command;

const USAGE: &str = "usage: bookmarks new|list|open|view TAGS... [--title TITLE...]";

#[derive(Debug)]
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

    dbg!(&args);

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
    dbg!(substrings);
    substrings.is_empty() || substrings.iter().all(|substring| s.contains(substring))
}

fn title_to_filename(t: &str) -> String {
    let mut filename = String::new();
    let mut prev = '_';
    for ch in t.chars() {
        if ch.is_alphanumeric() {
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

fn bookmarks_dir() -> std::path::PathBuf {
    match std::env::var("BOOKMARKS_DIR") {
        Ok(dir) => std::path::PathBuf::from(dir),
        _ => dirs::home_dir().unwrap(),
    }
}
