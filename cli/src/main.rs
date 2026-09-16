extern crate fitch_proof;

use clap::Parser;

mod summary;
use fitch_proof::format_proof;
use summary::*;

#[derive(Parser)]
struct Args {
    #[arg(required = true)]
    path: Vec<String>,

    #[arg(long, action)]
    no_template: bool,

    #[arg(long, action)]
    summary: bool,

    #[arg(long, action)]
    format: bool,

    #[arg(long, action)]
    debug: bool,
}

/// by default we use a,b,c for constants and x,y,z for variables
const DEFAULT_ALLOWED_VARIABLE_NAMES: &str = "x,y,z,u,v,w";

/// The *proof* itself (what the student wrote) should be given as a command line argument.
///
/// The *proof template* should be given via `stdin`.
///
/// Currently, there is NO SUPPORT for a custom set of allowed variable names over the command
/// line (it is only in the web GUI).
fn main() {
    let args = Args::parse();

    let debug = args.debug;
    let summary = args.summary;
    let template = if args.no_template {
        None
    } else {
        Some(
            std::io::stdin()
                .lines()
                .map(|s| s.unwrap().trim().to_string())
                .collect::<Vec<String>>(),
        )
    };

    for proof_file in &args.path {
        if args.format {
            format_file(proof_file);
        }
        check_file(template.as_deref(), proof_file, debug);
    }

    if summary {
        summaries_files(&args.path);
    }
}

fn format_file(proof_file: &String) {
    let Ok(proof) = std::fs::read_to_string(proof_file) else {
        println!(
            "{}: Fatal error: Cannot open the file. Aborting.",
            proof_file
        );
        std::process::exit(1)
    };
    let proof = format_proof(&proof);
    if let Err(e) = std::fs::write(proof_file, proof) {
        println!("{proof_file}: Fatal error: Cannot write back formatted file: {e}");
    }
}

fn check_file(template: Option<&[String]>, proof_file : &String, debug: bool) {
    let variables = DEFAULT_ALLOWED_VARIABLE_NAMES.to_string();

    let Ok(proof) = std::fs::read_to_string(proof_file) else {
        println!(
            "{}: Fatal error: Cannot open the file. Aborting.",
            proof_file
        );
        std::process::exit(1)
    };

    let result: String = match template {
        None => fitch_proof::check_proof(&proof, &variables),
        Some(template) => {
            fitch_proof::check_proof_with_template(&proof, template.to_vec(), &variables)
        }
    };
    println!("{}", result);
    if debug {
        println!("\nDebug proof with locations:\n{}", fitch_proof::debug_proof_with_locations(&proof));
    }
}
