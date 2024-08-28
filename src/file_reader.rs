use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use config::builder::DefaultState;
use config::ConfigBuilder;
use regex::Regex;

use crate::app::InputMode;
use crate::note::Link;
use crate::note::Note;
use crate::note::Tag;
use crate::utils::rc_rc;
use crate::utils::RcRc;

use config::Config;



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

pub fn get_notes(path: &str) -> Vec<RcRc<Note>> {
    let files = list_files(path);
    let mut contents: Vec<RcRc<Note>> = vec![];
    for file in files.iter() {
        let parsed = parse_file(read_file(file), file);
        contents.push(parsed);
    }
    println!("lengthof contents {}", contents.len());
    contents
}

pub fn parse_file(file_contents: String, path: &Path) -> RcRc<Note> {
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
    rc_rc(Note {
        title: title.to_str().unwrap().to_owned(),
        text: note_text.to_owned(),
        tags: if tags.is_empty() { None } else { Some(tags) },
        links: if links.is_empty() { None } else { Some(links) },
        mode: InputMode::Normal,
        edited: false,
        is_active: false,
        old_title: None,
    })
}

pub fn write_file(note: &mut Note) {
    let home = std::env::home_dir().unwrap();
    let path = home.to_str().unwrap().to_string() + "/.config/noter/config";
    


    let config_build = Config::builder().add_source(config::File::with_name(&path)).build().unwrap() ;

    let path = config_build.try_deserialize::<HashMap<String,String>>().unwrap().get("path").unwrap().to_owned();
    let test_location = "/mnt/g/My Drive/JamiesVault/".to_string();
    let file_name = path + &note.title + ".md";
    fs::write(file_name, note.clone().text.into_bytes()).unwrap();
}
