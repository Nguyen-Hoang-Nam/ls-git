use crate::model::{FileType, LastCommit, Row, Theme};

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

        let file_name = path.clone();

        let row = Row {
            file_type: last_commit.clone().file_type,
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

pub fn get_theme(theme: String) -> Theme {
    match theme.as_ref() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        "contrast" => Theme::Contrast,
        _ => Theme::Dimm,
    }
}

pub fn print_rows(rows: Vec<Row>, theme: Theme) {
    println!("");
    for row in rows.iter() {
        let file_name = &row.file_name;
        let file_name_len = file_name.len();

        if file_name_len > 35 {
            let mut file_name = file_name[0..30].to_string();
            file_name.push_str("...  ")
        }

        let icon = match row.file_type {
            FileType::File => "".to_string(),
            FileType::Directory => "".to_string(),
        };

        let icon_color;
        let file_name_color;
        let summary_color;
        let time_since_color;
        match theme {
            Theme::Dimm => {
                icon_color = "\x1B[38;2;173;186;199m";
                file_name_color = "\x1B[38;2;173;186;199m";
                summary_color = "\x1B[38;2;118;131;144m";
                time_since_color = "\x1B[38;2;118;131;144m";
            }

            Theme::Light => {
                icon_color = match row.file_type {
                    FileType::File => "\x1B[38;2;87;96;106m",
                    FileType::Directory => "\x1B[38;2;84;174;255m",
                };
                file_name_color = "\x1B[38;2;36;41;47m";
                summary_color = "\x1B[38;2;87;96;106m";
                time_since_color = "\x1B[38;2;87;96;106m";
            }

            Theme::Dark => {
                icon_color = "\x1B[38;2;139;148;158m";
                file_name_color = "\x1B[38;2;173;186;199m";
                summary_color = "\x1B[38;2;139;148;158m";
                time_since_color = "\x1B[38;2;139;148;158m";
            }

            Theme::Contrast => {
                icon_color = "\x1B[38;2;240;243;246m";
                file_name_color = "\x1B[38;2;240;243;246m";
                summary_color = "\x1B[38;2;240;243;246m";
                time_since_color = "\x1B[38;2;240;243;246m";
            }
        };

        println!(
            "{}{} {}{: <30} {}{: <50} {}{}",
            icon_color,
            icon,
            file_name_color,
            file_name,
            summary_color,
            row.summary,
            time_since_color,
            row.time_since
        );
    }
}
