use clap::{App, Arg};
use git2::{Error, Repository};
use std::collections::HashMap;

#[derive(Clone)]
struct LastCommit {
    summary: String,
    time: i64,
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

                let file_path_decorate;
                if file_path_components.len() > 1 {
                    file_path_decorate = format!("  {}", &file_path_components[0]);
                    // file_path_decorate
                    //     .to_string()
                    //     .push_str(&file_path_components[0]);
                } else {
                    file_path_decorate = format!("  {}", &file_path_str);
                    // file_path_decorate.to_string().push_str(&file_path_str);
                }

                let file_mod_time = commit.time();
                let summary = commit.summary();
                let unix_time = file_mod_time.seconds();

                let last_commit = LastCommit {
                    summary: summary.unwrap().to_string(),
                    time: unix_time,
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
        }
    }

    for (path, last_commit) in mtimes.iter() {
        println!("{} {} {}", path, last_commit.summary, last_commit.time);
    }

    Ok(())
}
