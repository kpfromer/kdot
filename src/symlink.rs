use anyhow::{anyhow, bail, Context, Result};
use fs::read_link;
use pathdiff::diff_paths;
use std::{fs, io, os::unix::fs as unixfs};
use std::{fs::canonicalize, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Creates a file symlink.
// pub fn link_file(from: &PathBuf, to: &PathBuf) -> Result<()> {
//     unixfs::symlink(to, from).with_context(|| {
//         format!(
//             "Failed to symlink \"{}\" -> \"{}\"",
//             from.as_os_str().to_str().unwrap(),
//             to.as_os_str().to_str().unwrap()
//         )
//     })?;

//     Ok(())
// }

/// Remove a file symlink.
pub fn unlink_file(path: &PathBuf) -> Result<()> {
    fs::remove_file(path).with_context(|| {
        format!(
            "Failed to remove \"{}\"",
            path.as_os_str().to_str().unwrap()
        )
    })?;

    Ok(())
}

/// Creates a folder symlink.
pub fn link_folder(from: &PathBuf, to: &PathBuf, recursive: bool) -> Result<()> {
    if from.exists() {
        bail!("\"from\" path already exists.");
    }

    if recursive {
        // Walk the files and symlink if file or create directory
        let walker = WalkDir::new(&to).into_iter();

        let full_to_path = canonicalize(&to)?;
        // let full_from_path = canonicalize(&from)?;

        debug!("Recursivly linking.");

        for entry in walker {
            let entry = entry.unwrap();
            let file = entry.path();

            if file.is_dir() && !file.exists() {
                debug!(
                    "\"{}\" directory does not exist. Creating it.",
                    file.as_os_str().to_str().unwrap()
                );

                fs::create_dir_all(file)?;
            } else if file.is_file() {
                let full_file_path = canonicalize(&file)?;

                match diff_paths(&full_file_path, &full_to_path) {
                    Some(path_diff) => {
                        println!("test");

                        let mut relative_from = from.clone();
                        relative_from.push(path_diff);

                        // Create the folder of the file (if it does not already exist)
                        let relative_from_parent = relative_from.parent().unwrap();
                        if !relative_from_parent.exists() {
                            fs::create_dir_all(relative_from_parent)?;
                            info!(
                                "Created \"{}\"",
                                relative_from_parent.as_os_str().to_str().unwrap()
                            );
                        }

                        // TODO: generic impl? (function generic over pathbuf and path?)
                        info!(
                            "Linking {} to {}",
                            relative_from.as_os_str().to_str().unwrap(),
                            full_file_path.as_os_str().to_str().unwrap()
                        );
                        // link_file(&relative_from, file)?;
                        // TODO: remove below infavor of generic impl
                        unixfs::symlink(&full_file_path, &relative_from)
                            .with_context(|| format!("Failed to link file."))?; // .with_context(|| {
                                                                                //     format!(
                                                                                //         "Failed to symlink \"{}\" -> \"{}\"",
                                                                                //         relative_from.as_os_str().to_str().unwrap(),
                                                                                //         file.as_os_str().to_str().unwrap()
                                                                                //     )
                                                                                // })?;
                    }
                    None => bail!("Invalid path difference."),
                }
            }
        }
    } else {
        debug!("Symlinking folder directly.");
        unixfs::symlink(&to, &from).with_context(|| format!("Failed to symlink."))?;
    }

    debug!("Done linking folders.");

    Ok(())
}

/// Remove a folder symlink.
pub fn unlink_folder(from: &PathBuf, to: &PathBuf, recursive: bool) -> Result<()> {
    if !from.exists() {
        warn!("info: from does not exist!");
    } else {
        if let Ok(_) = fs::read_link(from) {
            // Is symoblic

            unlink_file(from)?;
        }
    }

    Ok(())
}
