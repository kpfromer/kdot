use anyhow::{bail, Context, Result};
use pathdiff::diff_paths;
use std::{
    collections::HashSet,
    fs::{self},
    os::unix::fs as unixfs,
};
use std::{fs::canonicalize, path::PathBuf};
use walkdir::WalkDir;

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

/// Returns the absolute path of wanted from file, the to file symbolic file, and (if to file is a symlink) the real to path (resolved module file symlink).
/// ## Assumes
/// - `to_folder` is absolute
/// - `to_file` is absolute (ideally includes `to_folder` as parent otherwise will error)
/// - `from` is relative to the kdot file
fn get_paths(
    to_folder: &PathBuf,
    to_file: &PathBuf,
    from: &PathBuf,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    // Link to real file (since canonicalize follows symlink)
    let to_real_abs = canonicalize(&to_file)?;
    // Link to file that is a symbolic link
    let to_sym_abs = crate::path::absolute_path(&to_file)?;

    let diff = {
        if let Some(diff) = diff_paths(&to_sym_abs, &to_folder) {
            diff
        } else {
            bail!("Invalid path difference.");
        }
    };

    let mut relative_from = from.clone();
    relative_from.push(&diff);

    // Create the folder of the file (if it does not already exist)
    let relative_from_parent = relative_from.parent().unwrap();
    if !relative_from_parent.exists() {
        fs::create_dir_all(relative_from_parent)?;
        info!(
            "Created \"{}\"",
            relative_from_parent.as_os_str().to_str().unwrap()
        );
    }

    Ok((relative_from, to_sym_abs, to_real_abs))
}

/// Creates a folder symlink.
pub fn link_folder(from: &PathBuf, to: &PathBuf, recursive: bool) -> Result<()> {
    if recursive {
        // Walk the files and symlink if file or create directory
        let walker = WalkDir::new(&to).into_iter();

        let full_to_path = canonicalize(&to)?;

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
                // Handles files in module that link to other files in module (module/a.txt -> module/b.txt)
                let (from, to_display, to) =
                    get_paths(&full_to_path, &file.clone().to_path_buf(), &from)?;

                info!(
                    "Linking \"{}\" -> \"{}\"",
                    from.as_os_str().to_str().unwrap(),
                    to_display.as_os_str().to_str().unwrap()
                );

                // link_file(&relative_from, file)?;
                // TODO: remove below infavor of generic impl
                unixfs::symlink(&to, &from).with_context(|| {
                    format!(
                        "Failed to symlink \"{}\" -> \"{}\"{}",
                        from.as_os_str().to_str().unwrap(),
                        to.as_os_str().to_str().unwrap(),
                        if to_display != to {
                            " (symbolic module link)"
                        } else {
                            ""
                        }
                    )
                })?;
            }
        }
    } else {
        debug!("Symlinking folder directly.");

        if from.exists() {
            bail!("\"from\" path already exists.");
        }

        unixfs::symlink(&to, &from).with_context(|| format!("Failed to symlink."))?;
    }

    debug!("Done linking folders.");

    Ok(())
}

/// Gets all files relative from `folder`
fn get_relative_files(folder: &PathBuf) -> Result<HashSet<PathBuf>> {
    let mut relative_files = HashSet::new();

    let walker = WalkDir::new(&folder).into_iter();

    for entry in walker {
        // Ignore invalid permissioned files
        if let Ok(entry) = entry {
            let file = entry.path();

            if file.is_file() {
                let relative_to_folder = diff_paths(file.to_owned(), &folder).unwrap();
                relative_files.insert(relative_to_folder);
            }
        }
    }

    Ok(relative_files)
}

/// Remove a folder symlink.
pub fn unlink_folder(from: &PathBuf, to: &PathBuf, recursive: bool) -> Result<()> {
    if !from.exists() {
        warn!("info: from does not exist!");
    } else if recursive {
        debug!("Unlinking recursivly.");

        let relative_files_to = get_relative_files(&to)?;

        debug!("Unlinking: {:?}", relative_files_to);

        if relative_files_to.is_empty() {
            warn!("There are no files to unlink!");
            return Ok(());
        }

        for file in relative_files_to
            .into_iter()
            // Filter out files that don't exist in `from`
            .filter(|file| {
                let from_file = {
                    let mut f = from.clone();
                    f.push(file);
                    f
                };

                from_file.exists()
            })
            // Make path absolute
            .map(|file| from.join(&file))
        {
            info!("Unlinking \"{}\"", file.as_os_str().to_str().unwrap());
            unlink_file(&file)?;
        }
    } else {
        debug!("Unlinking root folder.");

        if fs::read_link(from).is_ok() {
            // Is symoblic
            unlink_file(from)?;
        } else {
            bail!(
                "Expected \"{}\" to be symobolically linked to \"{}\" but it is not.",
                from.as_os_str().to_str().unwrap(),
                to.as_os_str().to_str().unwrap()
            );
        }
    }

    Ok(())
}
