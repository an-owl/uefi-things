use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{FileHandle, File, FileMode, FileAttribute};
use alloc::string::{String, ToString};
    /// This function returns a FileHandle from a given path relative to root.
    /// Returns error message on Err.
    /// Does not discriminate on trailing slashes
pub fn get_file_from_path(fs: &mut SimpleFileSystem, path: &str, mode: FileMode, attributes: FileAttribute) -> GetFileStatus {
    let mut root = fs.open_volume()
        .expect("Failed to open filesystem root").log();
    if ! path.starts_with('/'){
        return GetFileStatus::Err(uefi::Status::ABORTED);
    };

    let mut current_file = root.open(".",mode,attributes)
        .expect("Failed to get root handle. please open issue on github").log();

    let path_it = path.split('/');

    for file in path_it{
        //there is probably a better way to do this
        let new_file_result = current_file.open(file,mode,attributes);
        let new_file;

        match new_file_result{
            Ok(i) => {
                new_file = i.log();
            },
            Err(i) => {
                if i.status() == uefi::Status::NOT_FOUND{
                    return GetFileStatus::NotFound(file.to_string());
                }
                return GetFileStatus::Err(i.status())
            }
        }

        current_file = new_file
    };

    return GetFileStatus::Found(current_file)

}

pub enum GetFileStatus{
    Found(FileHandle),
    NotFound(String),
    Err(uefi::Status),
}