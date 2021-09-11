mod model;
mod utils;

use clap::{App, Arg};
use git2::{Error, Repository};
use model::{FileType, LastCommit};
use std::{collections::HashMap, fs};
use utils::sort_file;

fn main() -> Result<(), Error> {
    let matches = App::new("ls-git")
        .version("1.0.0")
        .author("N.H Nam <nguyenhoangnam.dev@gmail.com>")
        .about("View last commit and date")
        .arg(Arg::with_name("INPUT").help("Relative path").index(1))
        .get_matches();

    let directory = matches.value_of("INPUT").unwrap_or(".");

    let mut path_and_last_commit: HashMap<String, LastCommit> = HashMap::new();

    let repository = Repository::discover(directory)?;
    let mut revwalk = repository.revwalk()?;

    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit_id = commit_id?;
        let commit = repository.find_commit(commit_id)?;

        if commit.parent_count() == 1 {
            let tree = commit.tree()?;
            let prev_tree = commit.parent(0).unwrap().tree().unwrap();

            let diff = repository.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)?;

            for delta in diff.deltas() {
                let file_path = delta.new_file().path().unwrap();
                let file_path_str = file_path.to_str().unwrap();

                let file_path_components = file_path_str.split('/').collect::<Vec<&str>>();

                let file_type;
                let file_path_decorate;
                if file_path_components.len() > 1 {
                    file_path_decorate = &file_path_components[0];
                    file_type = FileType::Directory;
                } else {
                    file_path_decorate = &file_path_str;
                    file_type = FileType::File;
                }

                let file_mod_time = commit.time();
                let summary = commit.summary();
                let unix_time = file_mod_time.seconds();

                let last_commit = LastCommit {
                    summary: summary.unwrap().to_string(),
                    time: unix_time,
                    file_type,
                };

                path_and_last_commit
                    .entry(file_path_decorate.to_string())
                    // .and_modify(|t| {
                    //     *t = if t.time < unix_time {
                    //         last_commit.clone()
                    //     } else {
                    //         t.clone()
                    //     }
                    // })
                    .or_insert(last_commit);
            }
        } else if commit.parent_count() == 0 {
            let tree = commit.tree()?;

            for entry in tree.iter() {
                let file_mod_time = commit.time();
                let summary = commit.summary();
                let unix_time = file_mod_time.seconds();

                let last_commit = LastCommit {
                    summary: summary.unwrap().to_string(),
                    time: unix_time,
                    file_type: FileType::File,
                };

                path_and_last_commit
                    .entry(entry.name().unwrap().to_string())
                    // .and_modify(|t| {
                    //     *t = if t.time < unix_time {
                    //         last_commit.clone()
                    //     } else {
                    //         t.clone()
                    //     }
                    // })
                    .or_insert(last_commit);
            }
        }
    }

    let mut current_files: HashMap<String, LastCommit> = HashMap::new();

    let paths = fs::read_dir(directory).unwrap();
    for path in paths {
        let name = path
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if name != ".git" {
            current_files.insert(
                name.to_string(),
                path_and_last_commit.get(&name).unwrap().to_owned(),
            );
        }
    }

    let rows = sort_file(current_files);
    for row in rows.iter() {
        let file_name = &row.file_name;
        let file_name_len = file_name.len();

        if file_name_len > 35 {
            let mut file_name = file_name[0..30].to_string();
            file_name.push_str("...  ")
        }

        println!("{: <30} {: <50} {}", file_name, row.summary, row.time_since);
    }

    Ok(())
}
