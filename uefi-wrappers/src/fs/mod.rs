use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{FileHandle, File, FileMode, FileAttribute};
use alloc::string::{String, ToString};
    /// This function returns a FileHandle from a given path relative to root.
    /// Returns error message on Err.
    /// Does not discriminate on trailing slashes
pub fn get_file_from_path(fs: &mut SimpleFileSystem, path: &str) -> Result<FileHandle,String> {
    let mut root = fs.open_volume()
        .expect("Failed to open filesystem root").log();
    if ! path.starts_with('/'){
        return Result::Err("Invalid path".to_string());
    };

    let mut current_file = root.open(".",FileMode::Read,FileAttribute::READ_ONLY)
        .expect("Failed to get root handle. please open issue on github").log();

    let path_it = path.split('/');

    for file in path_it{
        //there is probably a better way to do this
        let new_file_result = current_file.open(file,FileMode::Read,FileAttribute::READ_ONLY);
        let new_file;

        match new_file_result{
            Ok(i) => {
                new_file = i.log();
            }
            Err(i) => {
                error!("{}", i.status().0);
                return Err("Failed to get file, errors logged".to_string())
            }
        }

        current_file = new_file;
    };

    return Result::Ok(current_file)

}