use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs
use std::{fs::File, io::Write};
use tempfile::TempDir;

fn setup_config() -> Result<TempDir, Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;

    let file_path = tmp_dir.path().join("kdot.json");
    let mut tmp_file = File::create(&file_path)?;

    writeln!(
        tmp_file,
        "
          {{
            \"modules\": [
              {{ \"name\": \"bash\", \"location\": \"stuff/bash\" }},
              {{ \"name\": \"vim\", \"location\": \"stuff/vimrc\" }}
            ]
          }}    
        "
    )?;

    Ok(tmp_dir)
}

#[test]
fn links_module() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = setup_config()?;

    let mut cmd = Command::cargo_bin("kdot")?;
    cmd.current_dir(tmp_dir.path().as_os_str().to_str().unwrap())
        .arg("link")
        .arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Linked \"bash\" module."));

    // TODO: check that it's in fact linked to the correct location

    Ok(())
}
