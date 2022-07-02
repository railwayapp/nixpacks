use anyhow::{Ok, Result};
use std::{fs, io, path::Path};
use walkdir::WalkDir;

pub fn recursive_copy_dir<T: AsRef<Path>, Q: AsRef<Path>>(source: T, dest: Q) -> Result<()> {
    let walker = WalkDir::new(&source).follow_links(false);
    for entry in walker {
        let entry = entry?;

        let from = entry.path();
        let to = dest.as_ref().join(from.strip_prefix(&source)?);

        // create directories
        if entry.file_type().is_dir() {
            if let Err(e) = fs::create_dir(to) {
                match e.kind() {
                    io::ErrorKind::AlreadyExists => {}
                    _ => return Err(e.into()),
                }
            }
        }
        // copy files
        else if entry.file_type().is_file() {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}
