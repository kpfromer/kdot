use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use serde_json::json; // Used for writing assertions
use std::{fs, path::PathBuf, process::Command}; // Run programs
use std::{fs::File, io::Write};
use tempfile::TempDir;

fn setup_config(
) -> Result<(TempDir, (PathBuf, String), (PathBuf, String)), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;

    // Create from folder and file
    let from_path = tmp_dir.path().join("from");
    let from_path_string = from_path.clone().as_os_str().to_str().unwrap().to_owned();
    fs::create_dir(&from_path)?;

    // Create to folder
    let to_path = tmp_dir.path().join("to");
    let to_path_string = to_path.clone().as_os_str().to_str().unwrap().to_owned();
    fs::create_dir(&to_path)?;

    let file_path = tmp_dir.path().join("kdot.json");
    let mut tmp_file = File::create(&file_path)?;

    writeln!(
        tmp_file,
        "{}",
        json!({
          "modules": [
            {
              "name": "bash",
              "location": {
                "from": from_path_string,
                "to": to_path_string
              }
            }
          ]
        })
        .to_string()
    )?;

    Ok((
        tmp_dir,
        (from_path, from_path_string),
        (to_path, to_path_string),
    ))
}

#[test]
fn links_module() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    let mut file = File::create(from_path.join("bashrc"))?;
    file.write_all(b"this is the bashrc!")?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash");

    cmd.assert().success();

    let predicate_fn = predicate::path::exists().and(predicate::path::is_symlink());

    let final_file = to_path.join("bashrc");
    assert_eq!(true, predicate_fn.eval(final_file.as_path()));

    Ok(())
}

#[test]
fn links_deeply_nested_file() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    fs::create_dir_all(from_path.clone().join("deeply/nested"))?;

    let mut file = File::create(from_path.join("deeply/nested/bashrc"))?;
    file.write_all(b"this is the bashrc!")?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash");

    cmd.assert().success();

    let exists_and_directory_and_not_symlinked = predicate::path::exists()
        .and(predicate::path::is_dir())
        .and(predicate::path::is_symlink().not());
    let exists_and_symlink = predicate::path::exists().and(predicate::path::is_symlink());

    assert_eq!(
        true,
        exists_and_symlink.eval(to_path.join("deeply/nested/bashrc").as_path())
    );
    assert_eq!(
        true,
        exists_and_directory_and_not_symlinked.eval(to_path.join("deeply").as_path())
    );
    assert_eq!(
        true,
        exists_and_directory_and_not_symlinked.eval(to_path.join("deeply/nested").as_path())
    );

    Ok(())
}

// todo: recursive folder creation

// #[test]
// fn unlinks_module() -> Result<(), Box<dyn std::error::Error>> {}

// #[test]
// fn syncs_module() -> Result<(), Box<dyn std::error::Error>> {}
