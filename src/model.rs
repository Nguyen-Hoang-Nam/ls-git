#[derive(Clone)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Clone)]
pub struct LastCommit {
    pub summary: String,
    pub time: i64,
    pub file_type: FileType,
}

pub struct Row {
    pub file_name: String,
    pub time_since: String,
    pub summary: String,
}
