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

    let mut time_since_last_commit;
    for (path, last_commit) in mtimes.iter() {
        let since_last_commit = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (last_commit.time as u64);

        if since_last_commit < 60 {
            time_since_last_commit = format!("{} seconds ago", since_last_commit);
        } else if since_last_commit < 3600 {
            time_since_last_commit = format!("{} minutes ago", since_last_commit / 60);
        } else if since_last_commit < 86400 {
            time_since_last_commit = format!("{} hours ago", since_last_commit / 3600);
        } else if since_last_commit < 2678400 {
            time_since_last_commit = format!("{} days ago", since_last_commit / 86400);
        } else {
            time_since_last_commit = format!("{} months ago", since_last_commit / 2678400);
        }

        println!(
            "{} {} {}",
            path, last_commit.summary, time_since_last_commit
        );
    }

    Ok(())
}
