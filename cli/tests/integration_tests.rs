use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

// Integration tests are set up as follows:
//  in the test_cases directory each test corresponds to 2 or 3 files:
//      - test_X.txt           the proof itself
//      - test_X.template      template to check against  [optional]
//      - test_X.expected      the expected output of the test

fn run_tests_in_dir(cli_path: &str, dir: &Path) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("Failed to read test directory {:?}: {}", dir, e)) {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let proof_file = path.to_str().unwrap();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            println!("Running test for: {}", stem);

            let template_file = path.with_extension("template");
            let expected_file = path.with_extension("expected");

            if !expected_file.exists() {
                println!("Skipping test for {} because no .expected file was found", stem);
                continue;
            }

            let mut command = Command::new(cli_path);
            command.arg(proof_file);

            let use_template = template_file.exists();
            if !use_template {
                command.arg("--no-template");
            }

            if use_template {
                command.stdin(Stdio::piped());
            }
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());

            let mut child = command.spawn().expect("Failed to spawn child process");

            if use_template {
                let template_content = fs::read_to_string(&template_file)
                    .unwrap_or_else(|e| panic!("Failed to read template file: {:?}: {}", template_file, e));
                let mut stdin = child.stdin.take().expect("Failed to open stdin");
                stdin.write_all(template_content.as_bytes()).expect("Failed to write to stdin");
            }

            let output = child.wait_with_output().expect("Failed to read stdout");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stderr.is_empty() {
                panic!("Test for {} failed with stderr:\n{}", stem, stderr);
            }

            let expected_output = fs::read_to_string(&expected_file)
                .unwrap_or_else(|e| panic!("Failed to read expected file: {:?}: {}", expected_file, e));

            assert_eq!(stdout.trim(), expected_output.trim(), "Test failed for {}", stem);
        }
    }
}

#[test]
fn run_integration_tests() {
    let cli_path = env!("CARGO_BIN_EXE_cli");
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_cases");
    
    // Run tests in the main test_cases directory, which contains passing tests
    run_tests_in_dir(cli_path, &base_dir);

    // Run tests in the failing subdirectory
    let failing_dir = base_dir.join("failing");
    if failing_dir.is_dir() {
        run_tests_in_dir(cli_path, &failing_dir);
    }
}
