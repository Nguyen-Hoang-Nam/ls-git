use clap::{App, Arg};
use git2::{Error, Repository};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
enum FileType {
    File,
    Directory,
}

#[derive(Clone)]
struct LastCommit {
    summary: String,
    time: i64,
    file_type: FileType,
}

struct Row {
    file_name: String,
    time_since: String,
    summary: String,
}

fn duration_to_time_since(duration: u64) -> String {
    let time_since;
    if duration < 60 {
        time_since = format!("{} seconds ago", duration);
    } else if duration < 3600 {
        time_since = format!("{} minutes ago", duration / 60);
    } else if duration < 86400 {
        time_since = format!("{} hours ago", duration / 3600);
    } else if duration < 2678400 {
        time_since = format!("{} days ago", duration / 86400);
    } else {
        time_since = format!("{} months ago", duration / 2678400);
    }

    return time_since;
}

fn sort_file(unorder_files: HashMap<String, LastCommit>) -> Vec<Row> {
    let mut directory_rows: Vec<Row> = Vec::new();
    let mut file_rows: Vec<Row> = Vec::new();

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (path, last_commit) in unorder_files.iter() {
        let since_last_commit = current_time - (last_commit.time as u64);
        let time_since = duration_to_time_since(since_last_commit);

        let file_name = match last_commit.file_type {
            FileType::File => format!(" {}", path),
            FileType::Directory => format!(" {}", path),
        };

        let row = Row {
            file_name,
            time_since,
            summary: last_commit.clone().summary,
        };

        match last_commit.file_type {
            FileType::File => file_rows.push(row),
            FileType::Directory => directory_rows.push(row),
        };
    }

    directory_rows.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    file_rows.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    directory_rows.extend(file_rows);

    return directory_rows;
}

fn main() -> Result<(), Error> {
    let matches = App::new("ls-git")
        .version("1.0.0")
        .author("N.H Nam <nguyenhoangnam.dev@gmail.com>")
        .about("View last commit and date")
        .arg(Arg::with_name("INPUT").help("Relative path").index(1))
        .get_matches();

    let directory = matches.value_of("INPUT").unwrap_or(".");

    let mut mtimes: HashMap<String, LastCommit> = HashMap::new();

    let repo = Repository::discover(directory)?;
    let mut revwalk = repo.revwalk()?;

    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;

        // Ignore merge commits (2+ parents) because that's what 'git whatchanged' does.
        // Ignore commit with 0 parents (initial commit) because there's nothing to diff against
        if commit.parent_count() == 1 {
            let tree = commit.tree()?;

            let prev_commit = commit.parent(0)?;
            let prev_tree = prev_commit.tree()?;

            let diff = repo.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)?;

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

                mtimes
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

                mtimes
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

    let rows = sort_file(mtimes);
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
