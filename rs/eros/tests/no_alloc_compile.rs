#![cfg(not(feature = "alloc"))]

#[test]
#[cfg_attr(miri, ignore)]
fn allocation_dependent_inputs_are_rejected_during_codegen() {
    use std::{fs, path::Path, process::Command};

    let project = std::env::temp_dir().join(format!("eros-no-alloc-{}", std::process::id()));
    fs::create_dir_all(&project).unwrap();
    let examples = [
        (
            "constructor",
            include_str!("no_alloc/constructor.rs"),
            "T must be zero-sized",
        ),
        (
            "mapping",
            include_str!("no_alloc/mapping.rs"),
            "T must be zero-sized",
        ),
        (
            "conversion",
            include_str!("no_alloc/conversion.rs"),
            "T must be zero-sized",
        ),
        (
            "static_payload",
            include_str!("no_alloc/static_payload.rs"),
            "T must be zero-sized",
        ),
        (
            "message",
            include_str!("no_alloc/message.rs"),
            "string error messages require the eros alloc feature",
        ),
    ];
    let mut manifest = format!(
        "[package]\nname = \"eros-no-alloc-compile-tests\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\neros = {{ path = {:?}, default-features = false }}\n",
        env!("CARGO_MANIFEST_DIR"),
    );
    for (name, source, _) in examples {
        manifest.push_str(&format!("[[bin]]\nname = {name:?}\npath = \"{name}.rs\"\n"));
        fs::write(project.join(format!("{name}.rs")), source).unwrap();
    }
    fs::write(project.join("Cargo.toml"), manifest).unwrap();
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/no-alloc-compile-tests");
    for (name, _, expected) in examples {
        // cargo check does not instantiate the const assertion. Build binaries
        // in an isolated workspace so dev dependencies cannot enable Eros alloc.
        let output = Command::new(env!("CARGO"))
            .args(["build", "--offline", "--manifest-path"])
            .arg(project.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(&target)
            .args(["--bin", name])
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{name} unexpectedly compiled");
        assert!(
            stderr.contains(expected),
            "unexpected diagnostic for {name}:\n{stderr}"
        );
    }
    fs::remove_dir_all(project).unwrap();
}
