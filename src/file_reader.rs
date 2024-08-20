use std::any::Any;
use std::env;
use std::fmt::Debug;
use std::fmt::Write;
use std::fs;
use std::fs::FileType;
use std::path::PathBuf;

use regex::Regex;

use crate::app::InputMode;
use crate::note::Link;
use crate::note::Note;
use crate::note::Tag;

pub fn read_file(path: PathBuf) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn list_files(path: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(path).unwrap();
    let mut files: Vec<PathBuf> = vec![];
    for path in paths {
        if let Ok(file) = path {
            println!("{:?}", file);
            match file.file_type() {
                Ok(s) => if s.is_file() {files.push(file.path())},
                Err(_) => continue
            }
        } 
    };
    files
}

pub fn parse_file(file_contents: String) -> Note {
    let note_text = &file_contents;
    let mut tags: Vec<Tag> = vec![];
    let link_regex = Regex::new(r"]\((.+)\)").unwrap();
    let mut links: Vec<Link> = vec![];
    let t = link_regex.captures_iter(&file_contents);
    for c in t {
        let extract = *c.extract::<1>().1.first().unwrap();
        links.push(Link(extract.to_owned()));
        println!("{:?}",extract);
    }
    for token in file_contents.split_whitespace() {
        if token.starts_with("#") {
            tags.push(Tag(token.to_owned()));
        }
    }
    Note {
        title: "test".to_string(),
        text: note_text.to_owned(),
        tags: if tags.is_empty() { None } else { Some(tags) },
        links: if links.is_empty() { None } else { Some(links)},
        mode: InputMode::Normal,
        edited: false,
    }
}

pub fn write_file(note: Note) {
    let test_location = "/home/jsmith49/testnotes/".to_string();
    let file_name = test_location + &note.title + ".md";
    fs::write(file_name,note.text.into_bytes());
}
