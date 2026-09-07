# About FitchVIZIER

FitchVIZIER is a formal proof checker, which determines correctness of
First-Order Logic proofs written in Fitch-style natural deduction. For
example, as in *Language, Proof and Logic*, by Dave Barker-Plummer,
Jon Barwise and John Etchemendy.

The aim of the tool is to both be useful as a checker, and an
interactive tool helping with developing formal proofs in the
Introduction to Logic course at the University of Groningen.

## CLI

The CLI interface can be used as follows: 
```
./fitchv file1.txt [file2.txt ...] [--no-template] [--summary]
```
will run the checker on all the proof filees, checking them against
the template (i.e. matching certain premises and a certain conclusion)
which is read from STDIN. If multiple input files are selected, then
all of them are checked against the template.

**Options**:

- If the `--no-template` option is selected, than the template is
  skipped and no input on STDIN is required.
- If the `--summary` option is selected, then in addition to the proof
  checking, FitchVIZIER will print a summary of the proof: which rules
  were used and how many times, and what is the maximal depth in the
  proof.
- Finally the `--debug` option dumps the internal representation is dumped onto STDOUT.  


## Web Version

The web version is available at: <https://fitch.rug.themisjudge.nl/>.
It is features an interactive editor
[Monaco](https://github.com/microsoft/monaco-editor) with features
like syntax highlighting, variable font size, auto-formatting on
Ctrl-S, and auto new line numbering.


## Contributors

FitchVIZIER was originally written by Aron Hardeman and is currently
maintained by Dan Frumin. A new WebUI was contributed by Lea Lohrie.


# Building FitchVIZIER

To build FitchVIZIER CLI you will need to have Rust installed. For
building the WebUI you will further need
[wasm-pack](https://github.com/wasm-bindgen/wasm-pack) and
[pnpm](https://github.com/pnpm/pnpm).

## Building the CLI

To build the CLI simply run `cargo build --release` or `cargo install --path .` in the `cli` directory.
If you want to run FitchVIZIER as a CLI or locally, you will need to install Rust, wasm-pack, pnpm, and follow the instructions below.
The resulting binary is called `fitchv`; release builds are found in the `target/release/` directory.


## Building the WebUI

To build the modern webui you will need to `cd` to the `webui` directory and run the following:

```
pnpm install --frozen-lockfile
pnpm build
```

The static web files will then be written to `webui/app/dist`.
You can `cd` to that directory and run the webserver with:
```
python3 -m http.server 8080
```

And then open [http://localhost:8080/](http://localhost:8080/) in your favorite web browser.

For the development, you can also use `pnpm preview` to run a dev web server that comes with vite.

## Compiling the legacy web interface

To use the legacy UI (called simpleui in the project), you can build it with `wasm-pack` (to compile Rust to WebAssembly):

```
wasm-pack build --target web --out-dir ../simpleui/pkg fitch-proof
```

Once you have it compiled, open a server in the `simpleui` directory of the repository:
```
python3 -m http.server 8080
```

And then open [http://localhost:8080/](http://localhost:8080/) in your favorite web browser.
