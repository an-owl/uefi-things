//! Contains convenience functions when interacting with the filesystem

//! I'd like to put new filesystem drivers here but that's not likely


use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{FileHandle, File, FileMode, FileAttribute, RegularFile, FileType};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::Status;

/// This function returns a FileHandle from a given path relative to root.
/// Returns error message on Err.
/// Does not discriminate on trailing slashes
pub fn get_file_from_path(fs: &mut SimpleFileSystem, path: &str, mode: FileMode, attributes: FileAttribute) -> GetFileStatus {
    //TODO make this less shit



    let mut root = fs.open_volume()
        .expect("Failed to open filesystem root").log();
    if ! path.starts_with('/'){
        return GetFileStatus::Err(uefi::Status::ABORTED);
    };

    let mut current_file = root.open(".",mode,attributes)
        .expect("Failed to get root handle. This should never happen").log();

    let path_it = path.split('/'); //first element is always blank and causes

    for file in path_it.skip(1){
        //there is probably a better way to do this
        trace!("len {}",file.len());

        let new_file_result = current_file.open(file,mode,attributes);
        let new_file;

        match new_file_result{
            Ok(i) => {
                new_file = i.log();
            },
            Err(i) => {
                if i.status() == uefi::Status::NOT_FOUND{
                    //TODO return whole path until missing file
                    return GetFileStatus::NotFound(file.to_string());
                }
                return GetFileStatus::Err(i.status())
            }
        }

        current_file = new_file
    };

    return GetFileStatus::Found(current_file)
}

///Reads whole file
pub fn read_file(mut file: RegularFile) -> uefi::Result<Vec<u8>> {
    //get file size
    let mut buffer = Vec::new();
    if let Err(e) = file.set_position(RegularFile::END_OF_FILE){
        return Err(e);
    };
    buffer.resize(file.get_position().unwrap().unwrap() as usize,0);

    file.set_position(0).unwrap().unwrap();
    file.read(&mut buffer).unwrap().unwrap();
    //unwraps should not panic they should only trigger if the if let above does
    Ok(uefi::Completion::from(buffer))
}

/// Returned by functions
pub enum GetFileStatus {
    /// File has been found
    Found(FileHandle),
    /// File has not been found
    NotFound(String),
    /// An error other than [NotFound][GetFileStatus::NotFound] has occurred
    Err(uefi::Status),
}

impl GetFileStatus {

    /// Gets [FileType][uefi::proto::media::file::file::FileType] from [GetFileStatus]
    pub fn into_type(self) -> Result<FileType,Status>{
        return match self{
            GetFileStatus::Found(f) => {
                //TODO remove unwrap
                Ok(f.into_type().unwrap().unwrap())
            }
            GetFileStatus::NotFound(_) => {
                Err(Status::NOT_FOUND)
            }
            GetFileStatus::Err(e) => Err(e)
        }
    }
}