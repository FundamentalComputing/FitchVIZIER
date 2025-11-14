extern crate fitch_proof;

use clap::Parser;

mod summary;
use summary::*;

#[derive(Parser)]
struct Args {
    #[arg(required = true)]
    path: Vec<String>,

    #[arg(long, action)]
    no_template: bool,

    #[arg(long, action)]
    summary: bool
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

    let no_template = args.no_template;
    let summary = args.summary;

    for proof_file in &args.path {
        check_file(no_template, proof_file)
    }
    
    if summary {
        summaries_files(&args.path);
    }

}

fn check_file(no_template : bool, proof_file : &String) {
    let variables = DEFAULT_ALLOWED_VARIABLE_NAMES.to_string();

    let Ok(proof) = std::fs::read_to_string(proof_file) else {
        println!(
            "{}: Fatal error: Cannot open the file. Aborting.",
            proof_file
        );
        std::process::exit(1)
    };

    let result: String = if no_template {
        fitch_proof::check_proof(&proof, &variables)
    } else {
        let template: Vec<String> = std::io::stdin()
            .lines()
            .map(|s| s.unwrap().trim().to_string())
            .collect();
        fitch_proof::check_proof_with_template(&proof, template, &variables)
    };
    println!("{}", result);

}
