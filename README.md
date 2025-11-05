# What is this?

This is a formal proof validator, which determines the correctness of Fitch-style natural deduction proofs ("Fitch proofs").

The tool was developed by Aron Hardeman and used by students who follow the course Introduction to Logic (for CS) at the University of Groningen (the Netherlands).

The tool also returns a (hopefully useful) error message in case the proof is not correct.

This application takes Fitch proofs as they are defined in *Language, Proof and Logic*, by Dave Barker-Plummer, Jon Barwise and John Etchemendy.

# Building and running FitchVIZIERE

## Web version

The web interface is accessible here: <https://fitch.rug.themisjudge.nl>

If you want to build and run the application locally, then clone the repository, install Cargo if you haven't already and install `wasm-pack` (to compile Rust to WebAssembly) and run the following in the `fitch-proof` directory:
```
wasm-pack build --target web
```

Once you have it compiled, open a server in the `fitch-proof` directory of the repository:
```
python3 -m http.server 8080
```

And then open [http://localhost:8080/](http://localhost:8080/) in your favorite web browser.

## CLI version

To build the CLI version run the following in the `cli` directory:
```
cargo build --release
```
The resulting binary is then found in `target/release/` directory.


The CLI interface can be used as follows: 
```
./cli file1.txt [file2.txt ...] [--no-template] [--summary]
```
will run the checker on all the proof filees, checking them against
the template (i.e. matching certain premises and a certain conclusion)
which is read from STDIN. If the `--no-template` option is selected,
than the template is skipped. If the `--summary` option is selected it
will further print the summary of the rule coverage in the proofs.
