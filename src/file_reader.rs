use std::any::Any;
use std::cell::RefCell;
use std::env;
use std::fmt::Debug;
use std::fmt::Result;
use std::fmt::Write;
use std::fs;
use std::fs::FileType;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use regex::Regex;

use crate::app::arc_ex;
use crate::app::ArcEx;
use crate::app::InputMode;
use crate::note::Link;
use crate::note::Note;
use crate::note::Tag;

pub fn read_file(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn list_files(path: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(path).unwrap();
    let mut files: Vec<PathBuf> = vec![];
    for path in paths {
        match path {
            Ok(file) => match file.file_type() {
                Ok(s) => {
                    if s.is_file() {
                        files.push(file.path())
                    }
                }
                Err(_) => continue,
            },
            Err(_) => todo!(),
        }
    }
    files
}

pub fn get_notes(path: &str) -> Vec<Note> {
    let files = list_files(path);
    let contents: Vec<Note> = files
        .iter()
        .map(|file| parse_file(read_file(file), file))
        .collect();
    contents
}

pub fn parse_file(file_contents: String, path: &Path) -> Note {
    let note_text = &file_contents;
    let mut tags: Vec<Tag> = vec![];
    let link_regex = Regex::new(r"]\((.+)\)").unwrap();
    let mut links: Vec<Link> = vec![];
    let t = link_regex.captures_iter(&file_contents);
    for c in t {
        let extract = *c.extract::<1>().1.first().unwrap();
        links.push(Link(extract.to_owned()));
        println!("{:?}", extract);
    }
    for token in file_contents.split_whitespace() {
        if token.starts_with("#") {
            tags.push(Tag(token.to_owned()));
        }
    }
    let title = path.file_stem().unwrap();
    Note {
        title: title.to_str().unwrap().to_owned(),
        text: note_text.to_owned(),
        tags: if tags.is_empty() { None } else { Some(tags) },
        links: if links.is_empty() { None } else { Some(links) },
        mode: Rc::new(RefCell::new(InputMode::Normal)),
        edited: false,
        is_active:false,
        old_title: None
    }
}

pub fn write_file(note: &mut Note) {
    let test_location = "/mnt/g/My Drive/JamiesVault/".to_string();
    let file_name = test_location + &note.title + ".md";
    fs::write(file_name, note.clone().text.into_bytes()).unwrap();
}
