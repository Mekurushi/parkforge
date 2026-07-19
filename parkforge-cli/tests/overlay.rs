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
