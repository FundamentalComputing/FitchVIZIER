# What is this?

This is a fork of the real thing.

The goal is to enable proving at the speed of thought.

Available at: https://fitch.wireva.eu/ (or https://code-for-groningen.github.io/BetterVIZIER/)

Changes so far:
- Ctrl + S to format
- Actual dark mode
- More sensible template
- Monaco Editor instead of textarea
- Auto new line number
- Syntax highlighting

This is a formal proof validator, which determines the correctness of Fitch-style natural deduction proofs ("Fitch proofs").

The tool was developed by Aron Hardeman and used by students who follow the course Introduction to Logic (for CS) at the University of Groningen (the Netherlands).

The tool also returns a (hopefully useful) error message in case the proof is not correct.

This application takes Fitch proofs as they are defined in *Language, Proof and Logic*, by Dave Barker-Plummer, Jon Barwise and John Etchemendy.

# Installation and Development
## Web Interface
You need to have rust and pnpm installed.

go into the `webui` dir and run `pnpm install`, then `pnpm build` (`pnpm dev` for a live dev server).

the static web files will then be written to `webui/app/dist`

## CLI
The cli can be found in the `cli` directory. To run it, use `cargo run <proof.txt>`, to obtain a production binary use `cargo build --release`
and you will find the binary in `cli/target` 
The CLI interface can be used as follows: the first argument is a file
with the proof; the STDIN contains the "template" (e.g. the statement
that has to be proven). The CLI then checks that the proof is correct
and that it proves the statement that was provided via STDIN.

## Prover core ('fitch-proof')
This is the core library that provides checking and formatting of proofs. Check `webui/library` for an example on how to use it
in a WASM context, check `cli` for usage as a normal rust library.
