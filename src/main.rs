use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize};

#[derive(Serialize)]
struct FileInfo {
    key: String,
    kind: String,
    name: String,
    ctime: u64,
    mtime: u64,
    size: u64,
    inode: u64,
}

#[derive(Serialize)]
struct DirectoryInfo {
    identifier: String,
    tuples: Vec<FileInfo>,
    inode: u64,
}

fn generate_tree_info(path: &Path, inode_counter: &mut u64) -> DirectoryInfo {
    let mut tuples = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let metadata = entry.metadata().unwrap();
                let inode = *inode_counter;
                *inode_counter += 1;

                let key = entry.file_name().to_string_lossy().to_string();
                let name = entry.file_name().to_string_lossy().to_string();
                let ctime = metadata.created().unwrap_or_else(|_| UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap().as_secs();
                let mtime = metadata.modified().unwrap_or_else(|_| UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap().as_secs();
                let size = metadata.len();
                let kind = if metadata.is_file() { "file" } else { "directory" };

                tuples.push(FileInfo {
                    key,
                    kind: kind.to_string(),
                    name,
                    ctime,
                    mtime,
                    size,
                    inode,
                });

                if metadata.is_dir() {
                    let subdir_info = generate_tree_info(&entry.path(), inode_counter);
                    tuples.push(FileInfo {
                        key: subdir_info.identifier.clone(),
                        kind: "directory".to_string(),
                        name: subdir_info.identifier,
                        ctime: 0,
                        mtime: 0,
                        size: 0,
                        inode: subdir_info.inode,
                    });
                }
            }
        }
    }

    DirectoryInfo {
        identifier: path.file_name().unwrap().to_string_lossy().to_string(),
        tuples,
        inode: 0,
    }
}

fn main() -> io::Result<()> {
    let root_directory = "examples/baf1"; 
    let output_file_path = Path::new(root_directory).join("directory_tree.json");

    let mut inode_counter = 1; 
    let tree_info = generate_tree_info(Path::new(root_directory), &mut inode_counter);

    let tree_info_json = serde_json::to_string_pretty(&tree_info).unwrap(); 

    let mut file = fs::File::create(output_file_path)?;
    file.write_all(tree_info_json.as_bytes())?;

    println!("Directory tree information written to directory_tree.json");
    Ok(())
}
