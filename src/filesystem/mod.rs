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

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FatEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attributes: u8,
    reserved: [u8; 10],
    cluster_high: u16,
    time: u16,
    date: u16,
    cluster_low: u16,
    size: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Fat32BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entries: u16,
    total_sectors_16: u16,
    media: u8,
    fat_size_16: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    fat_size_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    name: String,
    first_cluster: u32,
    size: u32,
    is_directory: bool,
}

impl FileEntry {
    pub fn new(name: String, first_cluster: u32) -> Self {
        FileEntry {
            name,
            first_cluster,
            size: 0,
            is_directory: false,
        }
    }
}

#[derive(Debug)]
pub struct FileSystem {
    boot_sector: Fat32BootSector,
    fat_table: Vec<u32>,
    data_region: Vec<u8>,
    files: Vec<FileEntry>,
}

impl FileSystem {
    pub fn new() -> Self {
        let boot_sector = Fat32BootSector {
            jmp_boot: [0xEB, 0x58, 0x90],
            oem_name: *b"MSWIN4.1",
            bytes_per_sector: 512,
            sectors_per_cluster: 1,
            reserved_sectors: 32,
            num_fats: 2,
            root_entries: 0,
            total_sectors_16: 0,
            media: 0xF8,
            fat_size_16: 0,
            sectors_per_track: 32,
            num_heads: 64,
            hidden_sectors: 0,
            total_sectors_32: 0x2000,
            fat_size_32: 32,
            ext_flags: 0,
            fs_version: 0,
            root_cluster: 2,
            fs_info: 1,
            backup_boot_sector: 6,
            reserved: [0; 12],
            drive_number: 0x80,
            reserved1: 0,
            boot_signature: 0x29,
            volume_id: 0x1234_5678,
            volume_label: *b"NO NAME    ",
            fs_type: *b"FAT32   ",
        };

        let mut fs = FileSystem {
            boot_sector,
            fat_table: Vec::new(),
            data_region: Vec::new(),
            files: Vec::new(),
        };

        let fat_size = (fs.boot_sector.fat_size_32 as usize) * (fs.boot_sector.bytes_per_sector as usize);
        fs.fat_table = vec![0; fat_size / 4];
        fs.fat_table[0] = 0x0FFFFF00;
        fs.fat_table[1] = 0x0FFFFFFF;

        let data_size = (fs.boot_sector.total_sectors_32 as usize 
            - fs.boot_sector.reserved_sectors as usize 
            - (fs.boot_sector.num_fats as usize * fs.boot_sector.fat_size_32 as usize)) 
            * fs.boot_sector.bytes_per_sector as usize;
        fs.data_region = vec![0; data_size];

        fs
    }

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

    pub fn write_file(&mut self, name: &str, content: &str) -> Result<(), FsError> {
        let content_bytes = content.as_bytes();
        let cluster_size = self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize;

        // Trouver l'index du fichier au lieu d'une référence mutabl
        let file_index = self.files.iter()
            .position(|f| f.name == name)
            .ok_or(FsError::FileNotFound)?;

        let mut current_cluster = self.files[file_index].first_cluster;
        let mut bytes_written = 0;

        for chunk in content_bytes.chunks(cluster_size) {
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

    pub fn list_files(&self) -> Result<Vec<String>, FsError> {
        Ok(self.files.iter().map(|f| f.name.clone()).collect())
    }

    fn allocate_cluster(&mut self) -> Option<u32> {
        for (i, &entry) in self.fat_table.iter().enumerate() {
            if entry == 0 {
                self.fat_table[i] = 0x0FFFFFFF;
                return Some(i as u32);
            }
        }
        None
    }

    fn read_cluster(&self, cluster: u32) -> Option<&[u8]> {
        let cluster_size = self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize;
        let start = (cluster as usize - 2) * cluster_size;
        
        if start >= self.data_region.len() {
            return None;
        }
        
        let end = start + cluster_size;
        if end > self.data_region.len() {
            return None;
        }
        
        Some(&self.data_region[start..end])
    }

    fn write_cluster(&mut self, cluster: u32, data: &[u8]) -> bool {
        let cluster_size = self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize;
        let start = (cluster as usize - 2) * cluster_size;
        
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

lazy_static! {
    pub static ref FS: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}