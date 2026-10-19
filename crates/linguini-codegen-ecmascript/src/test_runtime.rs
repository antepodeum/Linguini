//! Execute generated projects as native ESM, including their actual module graph.

use crate::EcmaGeneratedFile;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) fn execute_project(files: &[EcmaGeneratedFile], script: &str) -> String {
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/codegen-execution")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&directory).expect("create execution directory");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    fs::write(directory.join("package.json"), "{\"type\":\"module\"}").expect("ESM package");
    for file in files {
        let path = directory.join(&file.path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("module directory");
        fs::write(path, &file.contents).expect("generated module");
    }
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .current_dir(&directory)
        .output()
        .expect("Node.js must be available");
    assert!(
        output.status.success(),
        "generated project failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 output")
        .trim()
        .to_owned()
}
