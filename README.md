# FitchVIZIER

Available at: https://fitch.rug.themisjudge.nl/

## What is this?
This is a formal proof validator, which determines the correctness of Fitch-style natural deduction proofs ("Fitch proofs").

The tool was developed by Aron Hardeman and used by students who follow the course Introduction to Logic (for CS) at the University of Groningen (the Netherlands).

The tool also returns a (hopefully useful) error message in case the proof is not correct.

This application takes Fitch proofs as they are defined in *Language, Proof and Logic*, by Dave Barker-Plummer, Jon Barwise and John Etchemendy.

# How to run it?
You need to have rust and pnpm installed.

go into the fitch-proof dir and run `pnpm install`, then `pnpm build`.

now you can run the web devserver by going into `packages/app` and running `pnpm dev`

The CLI interface can be used as follows: the first argument is a file
with the proof; the STDIN contains the "template" (e.g. the statement
that has to be proven). The CLI then checks that the proof is correct
and that it proves the statement that was provided via STDIN.

