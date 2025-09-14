# What is this?

This is a fork of the real thing.
Available at: https://code-for-groningen.github.io/BetterVIZIER/

Changes so far:
- Ctrl + S to format
- Actual dark mode
- More sensible template

This is a formal proof validator, which determines the correctness of Fitch-style natural deduction proofs ("Fitch proofs").

The tool was developed by Aron Hardeman and used by students who follow the course Introduction to Logic (for CS) at the University of Groningen (the Netherlands).

The tool also returns a (hopefully useful) error message in case the proof is not correct.

This application takes Fitch proofs as they are defined in *Language, Proof and Logic*, by Dave Barker-Plummer, Jon Barwise and John Etchemendy.

# How to run it?

It is accessible here: <https://fitch.rug.themisjudge.nl>

If you want to build and run the application locally, then clone the repository, install Cargo if you haven't already and install `wasm-pack` (to compile Rust to WebAssembly) and do:

```
wasm-pack build --target web
```

Once you have it compiled, open a server in the `fitch-proof` directory of the repository:
```
python3 -m http.server 8080
```

And then open [http://localhost:8080/](http://localhost:8080/) in your favorite web browser.

The CLI interface can be used as follows: the first argument is a file
with the proof; the STDIN contains the "template" (e.g. the statement
that has to be proven). The CLI then checks that the proof is correct
and that it proves the statement that was provided via STDIN.
