#![cfg(not(feature = "alloc"))]

#[test]
#[cfg_attr(miri, ignore)]
fn word_payloads_compile_and_allocation_dependent_inputs_are_rejected() {
    use std::{fs, path::Path, process::Command};

    let project = std::env::temp_dir().join(format!("eros-no-alloc-{}", std::process::id()));
    fs::create_dir_all(&project).unwrap();
    let examples = [
        (
            "word_payload",
            include_str!("no_alloc/word_payload.rs"),
            None,
        ),
        (
            "reference_payload",
            include_str!("no_alloc/reference_payload.rs"),
            None,
        ),
        ("msg_error", include_str!("no_alloc/msg_error.rs"), None),
        (
            "aligned_payload",
            include_str!("no_alloc/aligned_payload.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "erased_payload",
            include_str!("no_alloc/erased_payload.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "constructor",
            include_str!("no_alloc/constructor.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "classification_payload",
            include_str!("no_alloc/classification_payload.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "mapping",
            include_str!("no_alloc/mapping.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "conversion",
            include_str!("no_alloc/conversion.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "static_payload",
            include_str!("no_alloc/static_payload.rs"),
            Some("T must fit in one usize"),
        ),
        (
            "message",
            include_str!("no_alloc/message.rs"),
            Some("formatted error messages require the eros alloc feature"),
        ),
        (
            "captured_message",
            include_str!("no_alloc/captured_message.rs"),
            Some("formatted error messages require the eros alloc feature"),
        ),
        (
            "bail_message",
            include_str!("no_alloc/bail_message.rs"),
            Some("formatted error messages require the eros alloc feature"),
        ),
        (
            "ensure_message",
            include_str!("no_alloc/ensure_message.rs"),
            Some("formatted error messages require the eros alloc feature"),
        ),
    ];
    let mut manifest = format!(
        "[package]\nname = \"eros-no-alloc-compile-tests\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\neros = {{ path = {:?}, default-features = false }}\n",
        env!("CARGO_MANIFEST_DIR"),
    );
    if cfg!(feature = "error_location") {
        manifest.push_str("[features]\nlocation = [\"eros/error_location\"]\n");
    }
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
            .args(if cfg!(feature = "error_location") {
                vec!["--features", "location"]
            } else {
                vec![]
            })
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(expected) = expected {
            assert!(!output.status.success(), "{name} unexpectedly compiled");
            assert!(
                stderr.contains(expected),
                "unexpected diagnostic for {name}:\n{stderr}"
            );
        } else {
            assert!(output.status.success(), "{name} should compile:\n{stderr}");
        }
    }
    fs::remove_dir_all(project).unwrap();
}
