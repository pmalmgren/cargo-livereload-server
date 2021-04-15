#[derive(Clone, Debug)]
pub enum FileChangeOperation {
    Chmod = 1,
    CloseWrite,
    Create,
    Remove,
    Rename,
    Rescan,
    Write,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct FileChange {
    path: String,
    operation: FileChangeOperation,
}

pub async fn watch() {

}
