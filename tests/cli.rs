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

fn setup_config_multiple() -> Result<
    (
        TempDir,
        (PathBuf, String),
        (PathBuf, String),
        (PathBuf, String),
    ),
    Box<dyn std::error::Error>,
> {
    let tmp_dir = TempDir::new()?;

    // Create first from folder
    let bash_path = tmp_dir.path().join("from/bash");
    let bash_path_string = bash_path.clone().as_os_str().to_str().unwrap().to_owned();
    fs::create_dir_all(&bash_path)?;

    // Create second from folder
    let zsh_path = tmp_dir.path().join("from/zsh");
    let zsh_path_string = zsh_path.clone().as_os_str().to_str().unwrap().to_owned();
    fs::create_dir_all(&zsh_path)?;

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
                "from": bash_path_string,
                "to": to_path_string
              }
            },
            {
                "name": "zsh",
                "location": {
                  "from": zsh_path_string,
                  "to": to_path_string
                }
              }
          ]
        })
        .to_string()
    )?;

    Ok((
        tmp_dir,
        (bash_path, bash_path_string),
        (zsh_path, zsh_path_string),
        (to_path, to_path_string),
    ))
}

#[test]
fn links_module() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    let bashrc_location = from_path.join("bashrc");
    let mut file = File::create(&bashrc_location)?;
    file.write_all(b"this is the bashrc!")?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash");

    cmd.assert().success();

    let exists_and_symlink = predicate::path::exists().and(predicate::path::is_symlink());

    let final_file = to_path.join("bashrc");
    assert_eq!(true, exists_and_symlink.eval(&final_file));
    assert_eq!(fs::canonicalize(&final_file)?, bashrc_location);

    Ok(())
}

// TODO: test about from: "string"

#[test]
fn links_multiple_modules() -> Result<(), Box<dyn std::error::Error>> {
    let (
        tmp_dir,
        (first_path, _first_path_string),
        (second_path, _second_path_string),
        (to_path, _to_path_string),
    ) = setup_config_multiple()?;

    let mut file = File::create(first_path.join("bashrc"))?;
    file.write_all(b"this is the bashrc!")?;

    let mut file = File::create(second_path.join("zshrc"))?;
    file.write_all(b"this is the zshrc!")?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash")
        .arg("zsh");

    cmd.assert().success();

    let exists_and_symlink = predicate::path::exists().and(predicate::path::is_symlink());

    let first_file = to_path.join("bashrc");
    let second_file = to_path.join("zshrc");

    // TODO: CHECK IT IS LINKED CORRECTLY!!!!
    assert_eq!(true, exists_and_symlink.eval(first_file.as_path()));
    assert_eq!(true, exists_and_symlink.eval(second_file.as_path()));

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

#[test]
fn links_module_linked_file() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    let bashrc_location = from_path.join("bashrc");
    File::create(&bashrc_location)?.write_all(b"this is the bashrc!")?;

    // Create a module link - aka link `zshrc` to `bashrc`
    let zshrc_location = from_path.join("zshrc");
    std::os::unix::fs::symlink(&bashrc_location, &zshrc_location)?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash");

    cmd.assert().success();

    let exists_and_symlink = predicate::path::exists().and(predicate::path::is_symlink());

    // TODO: CHECK IT IS LINKED CORRECTLY!!!!
    let bashrc_final = to_path.join("bashrc");
    let zshrc_final = to_path.join("zshrc");
    assert_eq!(true, exists_and_symlink.eval(bashrc_final.as_path()));
    assert_eq!(true, exists_and_symlink.eval(zshrc_final.as_path()));
    assert_eq!(fs::canonicalize(&bashrc_final)?, bashrc_location);
    // They should point to same file
    assert_eq!(
        fs::canonicalize(&bashrc_final)?,
        fs::canonicalize(&zshrc_final)?
    );

    Ok(())
}

#[test]
fn unlinks_module() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    // Creates a link: `to/linked.zsh` -> `from/linked.zsh`
    let location = from_path.join("linked.zsh");
    let linked_location = to_path.join("linked.zsh");
    File::create(&location)?;
    std::os::unix::fs::symlink(&location, &linked_location)?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("unlink")
        .arg("bash");

    cmd.assert().success();

    let does_not_exist = predicate::path::exists().not();

    assert_eq!(true, does_not_exist.eval(&linked_location));

    Ok(())
}

// TODO: test about from: "string"

#[test]
fn unlinks_multiple_modules() -> Result<(), Box<dyn std::error::Error>> {
    let (
        tmp_dir,
        (first_path, _first_path_string),
        (second_path, _second_path_string),
        (to_path, _to_path_string),
    ) = setup_config_multiple()?;

    // Creates a link: `to/bash/bashrc` -> `from/bashrc`
    let bashrc = first_path.join("bashrc");
    let bashrc_location = to_path.join("bashrc");
    File::create(&bashrc)?;
    std::os::unix::fs::symlink(&bashrc, &bashrc_location)?;

    // Creates a link: `to/zsh/zshrc` -> `from/zshrc`
    let zshrc = second_path.join("zshrc");
    let zshrc_location = to_path.join("zshrc");
    File::create(&zshrc)?;
    std::os::unix::fs::symlink(&zshrc, &zshrc_location)?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("unlink")
        .arg("bash")
        .arg("zsh");

    cmd.assert().success();

    let does_not_exist = predicate::path::exists().not();

    assert_eq!(true, does_not_exist.eval(&bashrc_location));
    assert_eq!(true, does_not_exist.eval(&zshrc_location));

    Ok(())
}

#[test]
fn unlinks_deeply_nested_module() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    // Creates a link: `to/deeply/nested/linked.zsh` -> `from/deeply/nested/linked.zsh`
    fs::create_dir_all(from_path.join("deeply/nested/"))?;
    fs::create_dir_all(to_path.join("deeply/nested/"))?;

    let location = from_path.join("deeply/nested/linked.zsh");
    let linked_location = to_path.join("deeply/nested/linked.zsh");
    File::create(&location)?;
    std::os::unix::fs::symlink(&location, &linked_location)?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("unlink")
        .arg("bash");

    cmd.assert().success();

    let does_not_exist = predicate::path::exists().not();
    let exists_and_dir = predicate::path::exists().and(predicate::path::is_dir());

    assert_eq!(true, does_not_exist.eval(&linked_location));
    assert_eq!(true, exists_and_dir.eval(&from_path.join("deeply/nested/")));
    assert_eq!(true, exists_and_dir.eval(&to_path.join("deeply/nested/")));

    Ok(())
}

#[test]
fn syncs_module() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, (from_path, _from_path_string), (to_path, _to_path_string)) = setup_config()?;

    // Creates a link: `to/linked.zsh` -> `from/linked.zsh`
    let location = from_path.join("linked.zsh");
    let linked_location = to_path.join("linked.zsh");
    File::create(&location)?;
    std::os::unix::fs::symlink(&location, &linked_location)?;

    // Create the unlinked file (that should be synced)
    let unlinked_location = from_path.join("unlinked.txt");
    File::create(&unlinked_location)?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("sync")
        .arg("bash");

    cmd.assert().success();

    let exists_and_symlink = predicate::path::exists().and(predicate::path::is_symlink());

    // TODO: CHECK IT IS LINKED CORRECTLY!!!!
    assert_eq!(true, exists_and_symlink.eval(&linked_location));
    assert_eq!(true, exists_and_symlink.eval(&to_path.join("unlinked.txt")));

    Ok(())
}

#[test]
fn syncs_multiple_modules() -> Result<(), Box<dyn std::error::Error>> {
    //  TODO
    Ok(())
}
