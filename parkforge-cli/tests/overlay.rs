use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEST_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

fn run(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(env!("CARGO_BIN_EXE_parkforge-cli"))
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: parkforge-cli {args:?}").into())
    }
}

fn run_failure(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(env!("CARGO_BIN_EXE_parkforge-cli"))
        .args(args)
        .status()?;
    if status.success() {
        Err(format!("command unexpectedly succeeded: parkforge-cli {args:?}").into())
    } else {
        Ok(())
    }
}

fn add_copy_rule(
    project: &std::path::Path,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = project.join("project.toml");
    let existing = fs::read_to_string(&config)?;
    fs::write(
        config,
        format!(
            "{existing}\n[[build.rule]]\nname = \"copy-{source}\"\nsource = \"{source}\"\ncompiler = \"copy\"\n"
        ),
    )?;
    Ok(())
}

#[test]
fn overlay_creation_uses_the_manifest_directory_shape_and_is_idempotent()
-> Result<(), Box<dyn std::error::Error>> {
    let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let workspace = TestWorkspace(std::env::temp_dir().join(format!(
        "parkforge-overlay-test-{}-{sequence}",
        std::process::id()
    )));
    let project = workspace.0.join("project");
    let project_arg = project.to_string_lossy();

    run(&[
        "project",
        "create",
        &project_arg,
        "--name",
        "test project",
        "--version",
        "0.1.0",
    ])?;
    run(&["project", "game", "add", &project_arg, "R8AJ01"])?;

    let original = project.join("original/R8AJ01");
    fs::create_dir_all(&original)?;
    fs::write(original.join("manifest.json"), manifest_fixture())?;

    run(&["overlay", "create", &project_arg, "R8AJ01"])?;
    run(&["overlay", "create", &project_arg, "R8AJ01"])?;

    let source = project.join("src/R8AJ01");
    assert!(source.join("DATA/files").is_dir());
    assert!(source.join("DATA/files/player.dan/nested").is_dir());
    assert!(!source.join("DATA/files/keep.bin").exists());
    assert!(
        !source
            .join("DATA/files/player.dan/nested/model.bin")
            .exists()
    );

    Ok(())
}

#[test]
fn build_materializes_the_baseline_and_applies_raw_source_assets()
-> Result<(), Box<dyn std::error::Error>> {
    let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let workspace = TestWorkspace(std::env::temp_dir().join(format!(
        "parkforge-build-test-{}-{sequence}",
        std::process::id()
    )));
    let project = workspace.0.join("project");
    let project_arg = project.to_string_lossy();

    run(&[
        "project",
        "create",
        &project_arg,
        "--name",
        "test project",
        "--version",
        "0.1.0",
    ])?;
    run(&["project", "game", "add", &project_arg, "R8AJ01"])?;
    add_copy_rule(&project, "**/*.bin")?;

    let original = project.join("original/R8AJ01");
    fs::create_dir_all(original.join("DATA/files/player.dan/nested"))?;
    fs::write(original.join("DATA/files/keep.bin"), b"baseline")?;
    fs::write(
        original.join("DATA/files/player.dan/nested/model.bin"),
        b"nested baseline",
    )?;
    fs::write(original.join("manifest.json"), manifest_fixture())?;

    let source = project.join("src/R8AJ01/DATA/files/player.dan/nested");
    fs::create_dir_all(&source)?;
    fs::write(
        project.join("src/R8AJ01/DATA/files/keep.bin"),
        b"replacement",
    )?;
    fs::write(source.join("added.bin"), b"added")?;

    run(&["build", &project_arg, "R8AJ01"])?;

    let build = project.join("build/R8AJ01");
    assert_eq!(fs::read(build.join("DATA/files/keep.bin"))?, b"replacement");
    assert_eq!(
        fs::read(build.join("DATA/files/player.dan/nested/model.bin"))?,
        b"nested baseline"
    );
    assert_eq!(
        fs::read(build.join("DATA/files/player.dan/nested/added.bin"))?,
        b"added"
    );
    assert!(!build.join("manifest.json").exists());

    Ok(())
}

#[test]
fn build_compiles_fsc_source_deterministically() -> Result<(), Box<dyn std::error::Error>> {
    let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let workspace = TestWorkspace(std::env::temp_dir().join(format!(
        "parkforge-fsc-build-test-{}-{sequence}",
        std::process::id()
    )));
    let project = workspace.0.join("project");
    let project_arg = project.to_string_lossy();

    run(&[
        "project",
        "create",
        &project_arg,
        "--name",
        "test project",
        "--version",
        "0.1.0",
    ])?;
    run(&["project", "game", "add", &project_arg, "R8AJ01"])?;
    fs::create_dir_all(project.join("original/R8AJ01/DATA/files"))?;

    let source = project.join("src/R8AJ01/DATA/files/test.fsc");
    fs::create_dir_all(source.parent().ok_or("source has no parent")?)?;
    fs::write(&source, "void test(int value) { return; }\n")?;

    run(&["build", &project_arg, "R8AJ01"])?;
    let output = project.join("build/R8AJ01/DATA/files/test.fsb");
    let first = fs::read(&output)?;
    assert!(!first.is_empty());

    run(&["build", &project_arg, "R8AJ01"])?;
    assert_eq!(fs::read(output)?, first);

    Ok(())
}

#[test]
fn build_rejects_raw_and_compiled_output_collisions() -> Result<(), Box<dyn std::error::Error>> {
    let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let workspace = TestWorkspace(std::env::temp_dir().join(format!(
        "parkforge-build-collision-test-{}-{sequence}",
        std::process::id()
    )));
    let project = workspace.0.join("project");
    let project_arg = project.to_string_lossy();

    run(&[
        "project",
        "create",
        &project_arg,
        "--name",
        "test project",
        "--version",
        "0.1.0",
    ])?;
    run(&["project", "game", "add", &project_arg, "R8AJ01"])?;
    add_copy_rule(&project, "**/*.fsb")?;
    fs::create_dir_all(project.join("original/R8AJ01/DATA/files"))?;

    let source = project.join("src/R8AJ01/DATA/files");
    fs::create_dir_all(&source)?;
    fs::write(
        source.join("script.fsc"),
        "void test(int value) { return; }\n",
    )?;
    fs::write(source.join("script.fsb"), b"raw replacement")?;

    run_failure(&["build", &project_arg, "R8AJ01"])?;
    assert!(!project.join("build/R8AJ01").exists());

    Ok(())
}

struct TestWorkspace(PathBuf);

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest_fixture() -> &'static str {
    r#"{
  "game_id": "R8AJ01",
  "containers": [
    {
      "id": 0,
      "virtual_path": "DATA/files/player.dan",
      "format": "U8",
      "compression": "None",
      "parent": null,
      "internal_path": null
    }
  ],
  "files": [
    {
      "virtual_path": "DATA/files/keep.bin",
      "container": null,
      "internal_path": null,
      "hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "size": 0
    },
    {
      "virtual_path": "DATA/files/player.dan/nested/model.bin",
      "container": 0,
      "internal_path": "nested/model.bin",
      "hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "size": 0
    }
  ]
}"#
}
