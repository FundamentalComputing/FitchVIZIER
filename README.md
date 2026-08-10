# What is this?

FitchVIZIER is a formal proof checker, which determines correctness of First-Order Logic proofs written in Fitch-style natural deduction. 
For example, as in *Language, Proof and Logic*, by Dave Barker-Plummer, Jon Barwise and John Etchemendy.

The aim of the tool is to both be useful as a checker, and an interactive tool helping with developing formal proofs in the Introduction to Logic course at the University of Groningen.


FitchVIZIER is originally written by Aron Hardeman and maintained by Dan Frumin.

# Web Version

The web version is available at: <https://fitch.rug.themisjudge.nl/>

# Building FitchVIZIER

If you want to run FitchVIZIER as a CLI or locally, you will need to install Rust, wasm-pack, pnpm, and follow the instructions below.

## Compiling the simple web interface

If you want to build and run the web interface locally, then use `wasm-pack` (to compile Rust to WebAssembly):

```
wasm-pack build --target web --out-dir ../webui/pkg fitch-proof
```

Once you have it compiled, open a server in the `webui` directory of the repository:
```
python3 -m http.server 8080
```

And then open [http://localhost:8080/](http://localhost:8080/) in your favorite web browser.

## CLI

To build the CLI version run the following in the `cli` directory:
```
cargo build --release
```
The resulting binary is then found in `target/release/` directory.


The CLI interface can be used as follows: 
```
./cli file1.txt [file2.txt ...] [--no-template]
```
will run the checker on all the proof filees, checking them against
the template (i.e. matching certain premises and a certain conclusion)
which is read from STDIN. If the `--no-template` option is selected,
than the template is skipped.
