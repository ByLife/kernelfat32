use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::format;

#[derive(Debug)]
pub enum FsError {
    FileNotFound,
    FileAlreadyExists,
    NoSpace,
    InvalidName,
    SystemError,
    NotADirectory,
    NotAFile,
    DirectoryNotEmpty,
}

#[derive(Debug, Clone)]
pub enum FsNode {
    File {
        name: String,
        content: String,
    },
    Directory {
        name: String,
        children: Vec<FsNode>,
    }
}

impl FsNode {
    pub fn new_file(name: String) -> Self {
        FsNode::File {
            name,
            content: String::new(),
        }
    }

    pub fn new_directory(name: String) -> Self {
        FsNode::Directory {
            name,
            children: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FsNode::File { name, .. } => name,
            FsNode::Directory { name, .. } => name,
        }
    }
}

#[derive(Debug)]
pub struct FileSystem {
    root: FsNode,
    current_path: Vec<String>,
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem {
            root: FsNode::new_directory(String::from("/")),
            current_path: Vec::new(),
        }
    }

    pub fn current_directory(&mut self) -> Result<&mut FsNode, FsError> {
        let mut current = &mut self.root;
        for name in &self.current_path {
            current = match current {
                FsNode::Directory { children, .. } => {
                    children.iter_mut()
                        .find(|node| node.name() == name)
                        .ok_or(FsError::FileNotFound)?
                },
                _ => return Err(FsError::NotADirectory),
            };
        }
        Ok(current)
    }

    pub fn create_directory(&mut self, name: &str) -> Result<(), FsError> {
        if name.len() > 64 {
            return Err(FsError::InvalidName);
        }

        let current_dir = match self.current_directory()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        if current_dir.iter().any(|node| node.name() == name) {
            return Err(FsError::FileAlreadyExists);
        }

        current_dir.push(FsNode::new_directory(String::from(name)));
        Ok(())
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), FsError> {
        if path == ".." {
            if !self.current_path.is_empty() {
                self.current_path.pop();
            }
            return Ok(());
        }

        if path == "/" {
            self.current_path.clear();
            return Ok(());
        }

        let current_dir = match self.current_directory()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        let target = current_dir.iter()
            .find(|node| node.name() == path)
            .ok_or(FsError::FileNotFound)?;

        match target {
            FsNode::Directory { .. } => {
                self.current_path.push(String::from(path));
                Ok(())
            },
            _ => Err(FsError::NotADirectory),
        }
    }

    pub fn print_working_directory(&self) -> Result<String, FsError> {
        if self.current_path.is_empty() {
            Ok(String::from("/"))
        } else {
            Ok(format!("/{}", self.current_path.join("/")))
        }
    }

    pub fn create_file(&mut self, name: &str) -> Result<(), FsError> {
        if name.len() > 64 {
            return Err(FsError::InvalidName);
        }

        let current_dir = match self.current_directory()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        if current_dir.iter().any(|node| node.name() == name) {
            return Err(FsError::FileAlreadyExists);
        }

        current_dir.push(FsNode::new_file(String::from(name)));
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, content: &str) -> Result<(), FsError> {
        let current_dir = match self.current_directory()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        let file = current_dir.iter_mut()
            .find(|node| node.name() == name)
            .ok_or(FsError::FileNotFound)?;

        match file {
            FsNode::File { content: file_content, .. } => {
                *file_content = String::from(content);
                Ok(())
            }
            _ => Err(FsError::NotAFile),
        }
    }

    pub fn read_file(&self, name: &str) -> Result<String, FsError> {
        let current_dir = match self.current_directory_ref()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        let file = current_dir.iter()
            .find(|node| node.name() == name)
            .ok_or(FsError::FileNotFound)?;

        match file {
            FsNode::File { content, .. } => Ok(content.clone()),
            _ => Err(FsError::NotAFile),
        }
    }

    pub fn remove(&mut self, name: &str) -> Result<(), FsError> {
        let current_dir = match self.current_directory()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        let index = current_dir.iter()
            .position(|node| node.name() == name)
            .ok_or(FsError::FileNotFound)?;

        match &current_dir[index] {
            FsNode::Directory { children, .. } if !children.is_empty() => {
                Err(FsError::DirectoryNotEmpty)
            },
            _ => {
                current_dir.remove(index);
                Ok(())
            }
        }
    }

    pub fn list_files(&self) -> Result<Vec<(String, bool)>, FsError> {
        let current_dir = match self.current_directory_ref()? {
            FsNode::Directory { children, .. } => children,
            _ => return Err(FsError::NotADirectory),
        };

        Ok(current_dir.iter().map(|node| {
            (String::from(node.name()), matches!(node, FsNode::Directory { .. }))
        }).collect())
    }

    fn current_directory_ref(&self) -> Result<&FsNode, FsError> {
        let mut current = &self.root;
        for name in &self.current_path {
            current = match current {
                FsNode::Directory { children, .. } => {
                    children.iter()
                        .find(|node| node.name() == name)
                        .ok_or(FsError::FileNotFound)?
                },
                _ => return Err(FsError::NotADirectory),
            };
        }
        Ok(current)
    }
}

lazy_static! {
    pub static ref FS: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}