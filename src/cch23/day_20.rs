use std::fmt::Display;
use std::io;
use std::path::Path;
use std::process::Command;

use rocket::post;
use serde::{Deserialize, Serialize};

#[post("/archive_files", data = "<bytes>")]
fn archive_files(bytes: Vec<u8>) -> io::Result<String> {
    let mut archive = tar::Archive::new(&bytes[..]);
    let entries = archive.entries()?;
    let mut file_count = 0;

    for entry in entries {
        let entry = entry?;
        match entry.header().entry_type() {
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                file_count += 1;
            }
            _ => (),
        }
    }

    Ok(file_count.to_string())
}

#[post("/archive_files_size", data = "<bytes>")]
fn archive_files_size(bytes: Vec<u8>) -> io::Result<String> {
    let mut archive = tar::Archive::new(&bytes[..]);
    let entries = archive.entries()?;
    let mut file_size = 0;

    for entry in entries {
        let entry = entry?;
        let header = entry.header();
        match header.entry_type() {
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                file_size += header.size()?;
            }
            _ => (),
        }
    }

    Ok(file_size.to_string())
}

#[derive(Debug, Deserialize, Serialize)]
struct CommitRef {
    author: String,
    hash: String,
    tree: String,
}

impl CommitRef {
    fn search_tree<P: AsRef<Path>>(&self, dir: P, keyword: &str, glob: &str) -> io::Result<bool> {
        let tree = self.tree.as_str();
        Command::new("git")
            .args(["grep", keyword, tree, "--", glob])
            .current_dir(dir)
            .status()
            .map(|exit_status| exit_status.success())
    }
}

impl Display for CommitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.author, self.hash)
    }
}

#[post("/cookie", data = "<bytes>")]
fn cookie(bytes: Vec<u8>) -> io::Result<String> {
    let tmp_dir = tempfile::tempdir()?;
    let mut archive = tar::Archive::new(&bytes[..]);

    archive.unpack(&tmp_dir)?;

    let branch_name = "christmas";
    let keyword = "COOKIE";
    let glob = "*santa.txt";
    let output = Command::new("git")
        .args([
            "log",
            r#"--format={"author":"%an","hash":"%H","tree":"%T"}"#,
            branch_name,
        ])
        .current_dir(&tmp_dir)
        .output()?;

    if let Ok(text) = std::str::from_utf8(&output.stdout) {
        let commits: Vec<CommitRef> = text.lines().flat_map(serde_json::from_str).collect();

        for commit_ref in commits {
            let found_cookie = commit_ref.search_tree(&tmp_dir, keyword, glob)?;

            if found_cookie {
                return Ok(commit_ref.to_string());
            }
        }
    }

    // this does NOT actually create a 404 :/ blah!
    Err(io::Error::new(io::ErrorKind::NotFound, "No commit found"))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![archive_files, archive_files_size, cookie]
}
