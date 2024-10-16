extern crate prost_build;

use std::{fs, path::Path};

fn main() {
    let proto_dir = "src/";
    let mut proto_files: Vec<String> = Vec::new();
    let path = Path::new(proto_dir);
    walk_dir(&path, &mut proto_files);
    println!(
        "Compile files size={}, {:?}",
        proto_files.len(),
        &proto_files
    );
    let mut config = prost_build::Config::new();
    config
        .compile_protos(&proto_files[..], &[proto_dir])
        .unwrap();
}

fn walk_dir(entry: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(entry) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        println!("File: {}", entry.path().display());
                        if let Some(ext) = entry.path().as_path().extension() {
                            let ext = ext.to_str().unwrap();
                            if ext == "proto" {
                                files.push(String::from(entry.path().to_str().unwrap()));
                            }
                        }
                    } else if file_type.is_dir() {
                        println!("Directory: {}", entry.path().display());
                        walk_dir(entry.path().as_path(), files);
                    } else {
                        println!("Other: {}", entry.path().display());
                    }
                }
            }
        }
    } else {
        println!("Failed to read directory.");
    }
}
