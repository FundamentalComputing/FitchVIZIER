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
    let mut failed: Vec<String> = Vec::new();
    run_integration_tests_dir(Path::new("tests/test_cases"), &mut failed);
    run_integration_tests_dir(Path::new("tests/private"), &mut failed);

    if !failed.is_empty() {
        panic!(
            "Some integration tests failed:\n\n{}",
            failed.join("\n\n--------------------\n\n")
        );
    }
}

fn run_integration_tests_dir(dir : &Path, failed: &mut Vec<String>) {
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
                let template_content = fs::read_to_string(&template_file).
                    expect(&format!("Failed to read template file: {:?}", template_file));
                let mut stdin = child.stdin.take().expect("Failed to open stdin");
                stdin
                    .write_all(template_content.as_bytes())
                    .expect("Failed to write to stdin");
            }

            let output = child.wait_with_output().expect("Failed to read stdout");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let expected_output = fs::read_to_string(&expected_file)
                .expect(&format!("Failed to read expected file: {:?}", expected_file));

            if !stderr.is_empty() || stdout.trim() != expected_output.trim() {
                failed.push(format!(
                    "Test {} failed.\nstdout:\n{}\nstderr:\n{}\nexpected:\n{}\n",
                    stem,
                    stdout,
                    stderr,
                    expected_output.trim()
                ));
            }
        }
    }
}


#[test]
fn reuses_template_for_multiple_proofs_2() {
    let cli_path = env!("CARGO_BIN_EXE_cli");
    let test_cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_cases");
    let proof_file = test_cases_dir.join("test_socrates.txt");
    let proof_file2 = test_cases_dir.join("test_eq_elim1.txt");
    let template = fs::read_to_string(test_cases_dir.join("test_socrates.template"))
        .expect("Failed to read template");
    let expected = fs::read_to_string(test_cases_dir.join("test_socrates.expected"))
        .expect("Failed to read expected output");

    let mut child = Command::new(cli_path)
        .arg(&proof_file)
        .arg(&proof_file)
        .arg(&proof_file2)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    child
        .stdin
        .take()
        .expect("Failed to open stdin")
        .write_all(template.as_bytes())
        .expect("Failed to write template");

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("{}\n{}\nline 1:1: The premises of your proof do not match the premises in the proof template.\n\nline 4:1: The conclusion of your proof does not match the conclusion in the proof template.", expected.trim(), expected.trim())
    );
}

#[test]
fn reuses_template_for_multiple_proofs() {
    let cli_path = env!("CARGO_BIN_EXE_cli");
    let test_cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_cases");
    let proof_file = test_cases_dir.join("test_socrates.txt");
    let template = fs::read_to_string(test_cases_dir.join("test_socrates.template"))
        .expect("Failed to read template");
    let expected = fs::read_to_string(test_cases_dir.join("test_socrates.expected"))
        .expect("Failed to read expected output");

    let mut child = Command::new(cli_path)
        .arg(&proof_file)
        .arg(&proof_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    child
        .stdin
        .take()
        .expect("Failed to open stdin")
        .write_all(template.as_bytes())
        .expect("Failed to write template");

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("{}\n{}", expected.trim(), expected.trim())
    );
}

