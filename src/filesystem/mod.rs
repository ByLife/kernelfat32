use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

// les erreurs qu'on peut avoir avec le systeme de fichiers
#[derive(Debug)]
pub enum FsError {
    FileNotFound,
    FileAlreadyExists,
    NoSpace,
    InvalidName,
    SystemError,
}

// structure pour une entrée fat32 classique
#[derive(Debug, Clone)]
pub struct FileEntry {
    name: String,
    first_cluster: u32,
    size: u32,
}

impl FileEntry {
    pub fn new(name: String, first_cluster: u32) -> Self {
        FileEntry {
            name,
            first_cluster,
            size: 0,
        }
    }
}

// systeme de fichiers simple en memoire
#[derive(Debug)]
pub struct FileSystem {
    fat_table: Vec<u32>,
    data_region: Vec<u8>,
    files: Vec<FileEntry>,
}

impl FileSystem {
    pub fn new() -> Self {
        // on fait un petit fs de 1 mio pour commencer
        let fat_size = 1024 * 1024 / 4; // 1 mio divisé par 4 car u32
        let mut fat_table = vec![0; fat_size];
        fat_table[0] = 0x0FFFFF00;
        fat_table[1] = 0x0FFFFFFF;

        // region de données de 1 mio aussi
        let data_region = vec![0; 1024 * 1024];

        FileSystem {
            fat_table,
            data_region,
            files: Vec::new(),
        }
    }

    // crée un nouveau fichier
    pub fn create_file(&mut self, name: &str) -> Result<(), FsError> {
        if name.len() > 255 {
            return Err(FsError::InvalidName);
        }

        if self.files.iter().any(|f| f.name == name) {
            return Err(FsError::FileAlreadyExists);
        }

        let cluster = self.allocate_cluster().ok_or(FsError::NoSpace)?;
        let file_entry = FileEntry::new(String::from(name), cluster);
        self.files.push(file_entry);
        Ok(())
    }

    // ecrit du contenu dans un fichier
    pub fn write_file(&mut self, name: &str, content: &str) -> Result<(), FsError> {
        let content_bytes = content.as_bytes();
        
        // on cherche le fichier par son index
        let file_index = self.files.iter()
            .position(|f| f.name == name)
            .ok_or(FsError::FileNotFound)?;

        let mut current_cluster = self.files[file_index].first_cluster;
        let mut bytes_written = 0;

        // on ecrit par morceaux de 512 octets
        for chunk in content_bytes.chunks(512) {
            if !self.write_cluster(current_cluster, chunk) {
                return Err(FsError::SystemError);
            }
            bytes_written += chunk.len();

            if bytes_written < content_bytes.len() {
                let next_cluster = self.allocate_cluster().ok_or(FsError::NoSpace)?;
                self.fat_table[current_cluster as usize] = next_cluster;
                current_cluster = next_cluster;
            } else {
                self.fat_table[current_cluster as usize] = 0x0FFFFFFF;
            }
        }

        self.files[file_index].size = content_bytes.len() as u32;
        Ok(())
    }

    // lit le contenu d'un fichier
    pub fn read_file(&self, name: &str) -> Result<String, FsError> {
        let file = self.files.iter()
            .find(|f| f.name == name)
            .ok_or(FsError::FileNotFound)?;

        let mut content = Vec::new();
        let mut current_cluster = file.first_cluster;

        while let Some(data) = self.read_cluster(current_cluster) {
            content.extend_from_slice(data);
            if let Some(&next_cluster) = self.fat_table.get(current_cluster as usize) {
                if next_cluster >= 0x0FFFFFF8 {
                    break;
                }
                current_cluster = next_cluster;
            } else {
                break;
            }
        }

        content.truncate(file.size as usize);
        String::from_utf8(content).map_err(|_| FsError::SystemError)
    }

    // liste tous les fichiers
    pub fn list_files(&self) -> Result<Vec<String>, FsError> {
        Ok(self.files.iter().map(|f| f.name.clone()).collect())
    }

    // trouve un cluster libre
    fn allocate_cluster(&mut self) -> Option<u32> {
        for (i, &entry) in self.fat_table.iter().enumerate() {
            if entry == 0 {
                self.fat_table[i] = 0x0FFFFFFF;
                return Some(i as u32);
            }
        }
        None
    }

    // lit un cluster de données
    fn read_cluster(&self, cluster: u32) -> Option<&[u8]> {
        let start = cluster as usize * 512;
        if start >= self.data_region.len() {
            return None;
        }
        let end = start + 512;
        if end > self.data_region.len() {
            return None;
        }
        Some(&self.data_region[start..end])
    }

    // ecrit dans un cluster
    fn write_cluster(&mut self, cluster: u32, data: &[u8]) -> bool {
        let start = cluster as usize * 512;
        if start >= self.data_region.len() {
            return false;
        }
        let end = start + data.len();
        if end > self.data_region.len() {
            return false;
        }
        self.data_region[start..end].copy_from_slice(data);
        true
    }
}

// instance globale du fs
lazy_static! {
    pub static ref FS: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}