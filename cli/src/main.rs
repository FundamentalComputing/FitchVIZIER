extern crate fitch_proof;

/// by default we use a,b,c for constants and x,y,z for variables
const DEFAULT_ALLOWED_VARIABLE_NAMES: &str = "x,y,z,u,v,w";

/// The *proof* itself (what the student wrote) should be given as a command line argument.
///
/// The *proof template* should be given via `stdin`.
///
/// Currently, there is NO SUPPORT for a custom set of allowed variable names over the command
/// line (it is only in the web GUI).
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        print_instructions_and_quit(args);
    }

    let proof_file = &args[1];
    let no_template = if args.len() == 3 {
        if args[2] == "--no-template" {
            true
        } else {
            print_instructions_and_quit(args);
        }
    } else {
        false
    };

    let Ok(proof) = std::fs::read_to_string(proof_file) else {
        println!(
            "Oops, it seems like the file {} could not be opened. Aborting.\n",
            proof_file
        );
        print_instructions_and_quit(args);
    };
    let variables = DEFAULT_ALLOWED_VARIABLE_NAMES.to_string();

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

fn print_instructions_and_quit(args: Vec<String>) -> ! {
    println!("Usage: {} <proof-file> [--no-template]", args[0]);
    std::process::exit(1);
}
