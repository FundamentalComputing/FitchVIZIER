# What is this?

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
