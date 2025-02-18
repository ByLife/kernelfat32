use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug)]
pub enum FsError {
    FileNotFound,
    FileAlreadyExists,
    NoSpace,
    InvalidName,
    SystemError,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    name: String,
    content: String,
}

impl FileEntry {
    pub fn new(name: String) -> Self {
        FileEntry {
            name,
            content: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct FileSystem {
    files: Vec<FileEntry>,
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem {
            files: Vec::with_capacity(10), // on reserve de la place pour 10 fichiers
        }
    }

    pub fn create_file(&mut self, name: &str) -> Result<(), FsError> {
        if name.len() > 16 {
            return Err(FsError::InvalidName);
        }

        if self.files.iter().any(|f| f.name == name) {
            return Err(FsError::FileAlreadyExists);
        }

        let file_entry = FileEntry::new(String::from(name));
        self.files.push(file_entry);
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, content: &str) -> Result<(), FsError> {
        let file = self.files.iter_mut()
            .find(|f| f.name == name)
            .ok_or(FsError::FileNotFound)?;

        file.content = String::from(content);
        Ok(())
    }

    pub fn read_file(&self, name: &str) -> Result<String, FsError> {
        let file = self.files.iter()
            .find(|f| f.name == name)
            .ok_or(FsError::FileNotFound)?;

        Ok(file.content.clone())
    }

    pub fn list_files(&self) -> Result<Vec<String>, FsError> {
        Ok(self.files.iter().map(|f| f.name.clone()).collect())
    }
}

lazy_static! {
    pub static ref FS: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}