use std::fs;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::mem::size_of;
// [修正 1] 移除未使用的 Path 引用
// use std::path::Path;

// --- 設定 ---
const SOURCE_DIR: &str = "fs_root"; // 來源資料夾
const TARGET_IMG: &str = "disk.img";
const DISK_SIZE: u64 = 32 * 1024 * 1024; // 32MB

// --- SimpleFS 結構定義 ---

#[repr(C)]
// [修正 2] 移除 Default，因為陣列太大無法自動推導
#[derive(Debug, Clone, Copy)]
struct Superblock {
    magic: u32,
    file_count: u32,
    _padding: [u8; 504],
}

// [修正 3] 手動實作 Default
impl Default for Superblock {
    fn default() -> Self {
        Self {
            magic: 0,
            file_count: 0,
            _padding: [0; 504], // 手動初始化為 0
        }
    }
}

// 64 bytes per entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEntryAligned {
    name: [u8; 32],
    start_sector: u32,
    size: u32,
    _padding: [u8; 24],
}

// [註] [u8; 24] 小於 32，所以這裡其實可以用 derive(Default)，
// 但為了保持風格一致，手動寫也沒問題 (或者保留原本的 impl Default)
impl Default for DirEntryAligned {
    fn default() -> Self {
        Self {
            name: [0; 32],
            start_sector: 0,
            size: 0,
            _padding: [0; 24],
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("--- SimpleFS Packer ---");
    println!("Scanning directory: {}", SOURCE_DIR);

    // 1. 掃描資料夾並讀取檔案
    let mut files_to_pack: Vec<(String, Vec<u8>)> = Vec::new();
    
    // 檢查目錄是否存在
    let paths = match fs::read_dir(SOURCE_DIR) {
        Ok(p) => p,
        Err(_) => {
            println!("Error: Directory '{}' not found.", SOURCE_DIR);
            println!("Please create it and put files (e.g., program.elf) inside.");
            return Ok(());
        }
    };

    for entry in paths {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                // 只打包 .txt 和 .elf
                if extension == "txt" || extension == "elf" {
                    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                    
                    // 讀取檔案內容
                    let mut file = File::open(&path)?;
                    let mut content = Vec::new();
                    file.read_to_end(&mut content)?;
                    
                    println!("Found: {} ({} bytes)", filename, content.len());
                    files_to_pack.push((filename, content));
                }
            }
        }
    }

    if files_to_pack.is_empty() {
        println!("No .txt or .elf files found in '{}'. Exiting.", SOURCE_DIR);
        return Ok(());
    }

    // 檢查檔案數量限制 (Kernel 限制讀取 Sector 1，最多 8 個檔案)
    if files_to_pack.len() > 8 {
        panic!("Error: Too many files! Kernel only supports 8 files currently.");
    }

    // 2. 建立磁碟映像檔
    let mut disk = File::create(TARGET_IMG)?;
    disk.set_len(DISK_SIZE)?;

    // 3. 寫入 Superblock (Sector 0)
    let mut sb = Superblock::default();
    sb.magic = 0x53465331;
    sb.file_count = files_to_pack.len() as u32;

    let sb_bytes = unsafe {
        std::slice::from_raw_parts(
            &sb as *const Superblock as *const u8,
            size_of::<Superblock>()
        )
    };
    disk.write_all(sb_bytes)?;

    // 4. 寫入檔案內容並建立目錄表
    let mut current_sector = 10; // Data area starts at sector 10
    let mut dir_entries = Vec::new();

    for (name, content) in files_to_pack {
        // 寫入檔案內容
        disk.seek(SeekFrom::Start(current_sector as u64 * 512))?;
        disk.write_all(&content)?;

        // 計算佔用 Sector 數
        let sectors_needed = (content.len() + 511) / 512;

        // 建立目錄項目
        let mut entry = DirEntryAligned::default();
        let name_bytes = name.as_bytes();
        
        if name_bytes.len() > 32 {
            panic!("Filename '{}' is too long (max 32 bytes)", name);
        }
        
        // 複製檔名
        for (i, &b) in name_bytes.iter().enumerate() {
            entry.name[i] = b;
        }
        
        entry.start_sector = current_sector as u32;
        entry.size = content.len() as u32;
        
        dir_entries.push(entry);
        
        current_sector += sectors_needed as u32;
    }

    // 5. 寫入目錄表 (Directory Table - Sector 1)
    disk.seek(SeekFrom::Start(1 * 512))?;
    for entry in dir_entries {
        let entry_bytes = unsafe {
            std::slice::from_raw_parts(
                &entry as *const DirEntryAligned as *const u8,
                size_of::<DirEntryAligned>()
            )
        };
        disk.write_all(entry_bytes)?;
    }

    println!("------------------------------------------------");
    println!("Successfully created '{}' with {} files.", TARGET_IMG, sb.file_count);
    Ok(())
}