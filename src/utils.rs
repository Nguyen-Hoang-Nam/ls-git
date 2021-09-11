use crate::model::{FileType, LastCommit, Row};

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

fn duration_to_time_since(duration: u64) -> String {
    let time_since;
    if duration < 60 {
        time_since = format!("{} seconds ago", duration);
    } else if duration < 120 {
        time_since = format!("1 minute ago");
    } else if duration < 3600 {
        time_since = format!("{} minutes ago", duration / 60);
    } else if duration < 7200 {
        time_since = format!("1 hour ago");
    } else if duration < 86400 {
        time_since = format!("{} hours ago", duration / 3600);
    } else if duration < 172800 {
        time_since = format!("yesterday");
    } else if duration < 2678400 {
        time_since = format!("{} days ago", duration / 86400);
    } else {
        time_since = format!("{} months ago", duration / 2678400);
    }

    return time_since;
}

pub fn sort_file(
    unorder_files: HashMap<String, LastCommit>,
) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let mut directory_rows: Vec<Row> = Vec::new();
    let mut file_rows: Vec<Row> = Vec::new();

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    for (path, last_commit) in unorder_files.iter() {
        let since_last_commit = current_time - (last_commit.time as u64);
        let time_since = duration_to_time_since(since_last_commit);

        let file_name = match last_commit.file_type {
            FileType::File => format!("  {}", path),
            FileType::Directory => format!("  {}", path),
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

    return Ok(directory_rows);
}
