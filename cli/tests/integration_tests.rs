use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};


// Integration tests are set up as follows:
//  in the test_cases directory each test corresponds to 2 or 3 files:
//      - test_X.txt           the proof itself
//      - test_X.template      template to check against  [optional]
//      - test_X.expected      the expected output of the test


#[test]
fn run_integration_tests() {
    run_integration_tests_dir(Path::new("tests/test_cases"));
    run_integration_tests_dir(Path::new("tests/private"));
}

fn run_integration_tests_dir(dir : &Path) {
    let cli_path = env!("CARGO_BIN_EXE_cli");
    let test_cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);

    for entry in fs::read_dir(test_cases_dir).expect("Failed to read test_cases directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
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
                    .expect(&format!("Failed to read template file: {:?}", template_file));
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
                .expect(&format!("Failed to read expected file: {:?}", expected_file));

            assert_eq!(stdout.trim(), expected_output.trim(), "Test failed for {}", stem);
        }
    }
}
