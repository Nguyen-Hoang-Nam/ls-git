mod model;
mod utils;

use clap::{App, Arg};
use git2::Repository;
use model::{FileType, LastCommit};
use std::{collections::HashMap, fs, path::PathBuf};
use utils::{get_theme, print_rows, sort_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ls-git")
        .version("1.0.0")
        .author("N.H Nam <nguyenhoangnam.dev@gmail.com>")
        .about("View last commit and date")
        .arg(
            Arg::with_name("theme")
                .short("t")
                .long("theme")
                .help("Change theme")
                .takes_value(true),
        )
        .arg(Arg::with_name("INPUT").help("Relative path").index(1))
        .get_matches();

    let theme = get_theme(matches.value_of("theme").unwrap_or("dimm").to_string());

    let directory = matches.value_of("INPUT").unwrap_or(".");
    let directory_path = PathBuf::from(directory).canonicalize()?;

    let mut path_and_last_commit: HashMap<String, LastCommit> = HashMap::new();

    let repository = Repository::discover(directory)?;
    let repository_path = repository.path().parent().unwrap();

    let relative_path = directory_path.strip_prefix(repository_path)?;

    let mut revwalk = repository.revwalk()?;

    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit = repository.find_commit(commit_id?)?;

        if commit.parent_count() == 1 {
            let tree = commit.tree()?;
            let prev_tree = commit.parent(0)?.tree()?;

            let diff = repository.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)?;

            for delta in diff.deltas() {
                let mut file_path = delta.new_file().path().unwrap();
                if file_path.starts_with(relative_path) {
                    file_path = file_path.strip_prefix(relative_path)?;

                    let mut file_path_str = file_path.to_str().unwrap();

                    let file_path_components = file_path_str.split('/').collect::<Vec<&str>>();

                    let file_type;
                    if file_path_components.len() > 1 {
                        file_path_str = &file_path_components[0];
                        file_type = FileType::Directory;
                    } else {
                        file_type = FileType::File;
                    }

                    // let unix_time = commit.time().seconds();

                    let last_commit = LastCommit {
                        summary: commit.summary().unwrap().to_string(),
                        time: commit.time().seconds(),
                        file_type,
                    };

                    path_and_last_commit
                        .entry(file_path_str.to_string())
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
        } else if commit.parent_count() == 0 {
            let mut tree = commit.tree()?;

            if relative_path.to_str().unwrap() != "" {
                let entry_id = tree.get_path(relative_path)?.id();

                tree = repository.find_tree(entry_id)?;
            }

            for entry in tree.iter() {
                // let unix_time = commit.time().seconds();

                let last_commit = LastCommit {
                    summary: commit.summary().unwrap().to_string(),
                    time: commit.time().seconds(),
                    file_type: match entry.kind().unwrap() {
                        git2::ObjectType::Tree => FileType::Directory,
                        _ => FileType::File,
                    },
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

    let paths = fs::read_dir(directory)?;
    for path in paths {
        let name = path?
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        match path_and_last_commit.get(&name) {
            Some(value) => {
                current_files.insert(name.to_string(), value.to_owned());
            }
            None => {}
        }
    }

    let rows = sort_file(current_files)?;
    print_rows(rows, theme);

    Ok(())
}
