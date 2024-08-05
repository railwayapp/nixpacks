use anyhow::Result;
use ignore::WalkBuilder;
use std::{fs, io, os::unix::fs::PermissionsExt, path::Path};

fn is_writable<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();
    Ok(permissions.mode() & 0o200 != 0)
}

/// Copies a directory and all its contents to the destination path, recursively.
pub fn recursive_copy_dir<T: AsRef<Path>, Q: AsRef<Path>>(source: T, dest: Q) -> Result<()> {
    let walker = WalkBuilder::new(&source)
        .follow_links(false)
        // this includes hidden directories & files
        .standard_filters(false)
        .hidden(false)
        .build();

    for entry in walker {
        let entry = entry?;

        if let Some(file_type) = entry.file_type() {
            let from = entry.path();
            let to = dest.as_ref().join(from.strip_prefix(&source)?);

            // create directories
            if file_type.is_dir() {
                if let Err(e) = fs::create_dir(to) {
                    match e.kind() {
                        io::ErrorKind::AlreadyExists => {}
                        _ => return Err(e.into()),
                    }
                }
            }
            // copy files
            else if file_type.is_file() {
                fs::copy(from, &to)?;

                if is_writable(&to)? {
                    // replace CRLF with LF
                    if let Ok(data) = fs::read_to_string(from) {
                        fs::write(&to, data.replace("\r\n", "\n"))?;
                    }
                }
            }
        }
    }
    Ok(())
}
