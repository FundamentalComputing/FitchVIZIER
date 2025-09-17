# What is this?

This is a fork of the real thing.

The goal is to enable proving at the speed of thought.

Available at: https://code-for-groningen.github.io/BetterVIZIER/

## ✨ Preview Deployments
This repository supports automatic preview deployments for Pull Requests! When you open a PR, a preview link will be automatically generated so reviewers can test your changes instantly. See [`PREVIEW_DEPLOYMENT.md`](PREVIEW_DEPLOYMENT.md) for details.

## Changes so far:
- Ctrl + S to format
- Actual dark mode
- More sensible template
- Monaco Editor instead of textarea
- Auto new line number
- Syntax highlighting
- **🚀 PR Preview Deployments** - Automatic staging deployments for every pull request

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

## For Repository Maintainers

### Preview Deployments Setup
The PR preview system is ready to use! The first time a preview is created, ensure GitHub Pages is configured to serve from multiple branches in **Settings** → **Pages**. Individual preview branches will be accessible via GitHub Pages subdirectory structure.
