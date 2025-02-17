use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

// Struct pour représenter structure de données FAT32
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

// structure fat 32 boot sector
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

pub struct FileSystem {
    boot_sector: Fat32BootSector,
    fat_table: Vec<u32>,
    data_region: Vec<u8>,
}

impl FileSystem {
    pub fn new() -> Self {
        // init avec des values par défaut
        let boot_sector = Fat32BootSector {
            jmp_boot: [0xEB, 0x58, 0x90],  // jump instruction
            oem_name: *b"MSWIN4.1",        // oem name
            bytes_per_sector: 512,
            sectors_per_cluster: 1,
            reserved_sectors: 32,
            num_fats: 2,
            root_entries: 0,               // toujours 0 pour FAT32
            total_sectors_16: 0,           // pour FAT32, utiliser total_sectors_32
            media: 0xF8,                   // type de média fixe
            fat_size_16: 0,               // non utilisé en FAT32
            sectors_per_track: 32,
            num_heads: 64,
            hidden_sectors: 0,
            total_sectors_32: 0x2000,      // 8192 secteurs de 512 octets
            fat_size_32: 32,              // taille de la FAT en secteurs
            ext_flags: 0,
            fs_version: 0,
            root_cluster: 2,              // 1er cluster de la racine
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

        // initialisation d'une FAT simple
        let fat_size = (boot_sector.fat_size_32 as usize) * (boot_sector.bytes_per_sector as usize);
        let mut fat_table = Vec::with_capacity(fat_size / 4);
        fat_table.resize(fat_size / 4, 0);

        // marquer les deux premiers clusters comme utilisés
        fat_table[0] = 0x0FFFFF00;
        fat_table[1] = 0x0FFFFFFF;

        // region de données initiale
        let data_size = (boot_sector.total_sectors_32 as usize - boot_sector.reserved_sectors as usize 
            - (boot_sector.num_fats as usize * boot_sector.fat_size_32 as usize)) 
            * boot_sector.bytes_per_sector as usize;
        let mut data_region = Vec::with_capacity(data_size);
        data_region.resize(data_size, 0);

        FileSystem {
            boot_sector,
            fat_table,
            data_region,
        }
    }

    // fonction pour allouer un cluster
    pub fn allocate_cluster(&mut self) -> Option<u32> {
        for (i, &entry) in self.fat_table.iter().enumerate() {
            if entry == 0 {
                self.fat_table[i] = 0x0FFFFFFF;  //  marqueur de fin de fichier
                return Some(i as u32);
            }
        }
        None
    }

    pub fn read_cluster(&self, cluster: u32) -> Option<&[u8]> {
        let start = (cluster as usize - 2) * (self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize);
        if start >= self.data_region.len() {
            return None;
        }
        let end = start + (self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize);
        Some(&self.data_region[start..end])
    }

    pub fn write_cluster(&mut self, cluster: u32, data: &[u8]) -> bool {
        let start = (cluster as usize - 2) * (self.boot_sector.sectors_per_cluster as usize 
            * self.boot_sector.bytes_per_sector as usize);
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

// instance globale du système de fichiers
lazy_static! {
    pub static ref FILESYSTEM: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}