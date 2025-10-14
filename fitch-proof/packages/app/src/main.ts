// @ts-ignore
import "@fontsource-variable/fira-code";
// import "./style.css";
import {
  check_proof,
  check_proof_with_template,
  export_to_latex,
  fix_line_numbers_in_proof,
  format_proof,
  init,
} from "@workspace/library";
import * as monaco from "monaco-editor";
import { tsParticles } from "@tsparticles/engine";
import "@tsparticles/preset-confetti";
import { loadConfettiPreset } from '@tsparticles/preset-confetti';
// import MonacoErrorLens, { type MonacoEditor } from "@ym-han/monaco-error-lens";
import examples from "./examples.ts";
import excercises from "./excercises.ts";


declare global {
  interface Window {
    editor: monaco.editor.IStandaloneCodeEditor;
    load_example: (index: number) => void;
  }
}

const proofTargetEl = document.getElementById("proof_target") as HTMLInputElement;
let proofTarget = proofTargetEl.value;
let confettiPlayed = false;
loadConfettiPreset(tsParticles);

monaco.languages.register({
  id: "fitch",
  extensions: [".fitch", ".txt"],
  aliases: ["Fitch Proof", "fitch"],
  mimetypes: ["text/fitch"],
});

// Define the syntax highlighting rules
monaco.languages.setMonarchTokensProvider("fitch", {
  tokenizer: {
    root: [
      // Line numbers at start
      [/^\s*\d+/, "line-number"],

      // Proof structure - vertical bars and dashes
      [/\s*\|\s*/, "proof-structure"],
      [/\s*-+\s*/, "proof-structure"],

      // Logical operators and symbols
      [/∧/, "operator.conjunction"],
      [/∨/, "operator.disjunction"],
      [/¬/, "operator.negation"],
      [/→/, "operator.implication"],
      [/↔/, "operator.biconditional"],
      [/∀/, "quantifier.universal"],
      [/∃/, "quantifier.existential"],
      [/⊥/, "operator.falsum"],
      [/⊤/, "operator.verum"],
      [/=/, "operator.equality"],
      [/≠/, "operator.inequality"],

      // Parentheses and brackets
      [/[()[\]{}]/, "delimiter"],

      // Predicate/function names (capital letters)
      [/[A-Z][a-zA-Z0-9]*/, "predicate"],

      // Variables and constants (lowercase)
      [/[a-z][a-zA-Z0-9]*/, "variable"],

      // Function applications like f(a), g(x,y)
      [/[a-z]+(?=\()/, "function"],

      // Justifications - rule names
      [
        /\b(Reit|∧\s*Elim|∨\s*Elim|∧\s*Intro|∨\s*Intro|→\s*Elim|→\s*Intro|¬\s*Elim|¬\s*Intro|=\s*Elim|=\s*Intro|∀\s*Elim|∀\s*Intro|∃\s*Elim|∃\s*Intro|⊥\s*Elim|RAA|MT|DS|HS|Add|Simp|Conj|MP|DeM|DN|Com|Assoc|Dist|Exp|Equiv|Impl|Taut|Contra)\b/,
        "rule-name",
      ],

      // Line references in justifications (numbers, ranges)
      [/:\s*\d+/, "justification.reference"],
      [/\d+-\d+/, "justification.reference"],
      [/,\s*\d+/, "justification.reference"],

      // Comma separator
      [/,/, "delimiter"],

      // Whitespace
      [/\s+/, "white"],
    ],
  },
});

// Define the color theme
monaco.editor.defineTheme("fitch-theme", {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "line-number", foreground: "888888", fontStyle: "bold" },
    { token: "proof-structure", foreground: "888888" },
    { token: "operator.conjunction", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.disjunction", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.negation", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.implication", foreground: "CC6666", fontStyle: "bold" },
    {
      token: "operator.biconditional",
      foreground: "CC6666",
      fontStyle: "bold",
    },
    { token: "operator.falsum", foreground: "FF4444", fontStyle: "bold" },
    { token: "operator.verum", foreground: "44FF44", fontStyle: "bold" },
    { token: "operator.equality", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.inequality", foreground: "CC6666", fontStyle: "bold" },
    { token: "quantifier.universal", foreground: "9966CC", fontStyle: "bold" },
    {
      token: "quantifier.existential",
      foreground: "9966CC",
      fontStyle: "bold",
    },
    { token: "predicate", foreground: "99CC99" },
    { token: "function", foreground: "CCCC66" },
    { token: "variable", foreground: "99CCFF" },
    { token: "rule-name", foreground: "FF9966", fontStyle: "italic" },
    { token: "justification.reference", foreground: "CCCCCC" },
    { token: "delimiter", foreground: "#87875f" },
  ],
  colors: {
    "editor.background": "#1e1e1e",
  },
});

const value = `1 | A
  |----
2 | A           Reit: 1`;
const uri = monaco.Uri.parse("inmemory://test");
const model = monaco.editor.createModel(value, "fitch", uri);

const editor = monaco.editor.create(document.getElementById("editor"), {
  //   value: `1 | A
  //   |----
  // 2 | A           Reit: 1`,
  model,
  language: "fitch",
  theme: "fitch-theme",
  lineNumbers: "off",
  automaticLayout: true,
  fontFamily: "Fira Code Variable",
  fontLigatures: true,
  glyphMargin: true,
  minimap: { enabled: false },
  unicodeHighlight: {
    ambiguousCharacters: false,
    invisibleCharacters: true,
  },
});
window.editor = editor;

editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, function() {
  format();
});

// Add custom key bindings
editor.addCommand(monaco.KeyCode.Tab, function() {
  insertPipe(editor);
});

editor.addCommand(monaco.KeyMod.Shift | monaco.KeyCode.Tab, function() {
  removePipe(editor);
});

editor.addCommand(monaco.KeyCode.Enter, function() {
  insertNewline(editor, false);
});

editor.addCommand(monaco.KeyMod.Shift | monaco.KeyCode.Enter, function() {
  insertNewline(editor, true);
});

function insertPipe(editor: monaco.editor.IStandaloneCodeEditor) {
  const selection = editor.getSelection();
  // const model = editor.getModel();

  if (selection.isEmpty()) {
    const position = editor.getPosition();
    const lineContent = model.getLineContent(position.lineNumber);

    const column = lineContent.length - lineContent.split("").reverse().findIndex((v) => v == '|');

    editor.pushUndoStop();
    editor.executeEdits("insert-pipe", [{
      range: new monaco.Range(
        position.lineNumber,
        column - 1,
        position.lineNumber,
        column,
      ),
      text: " | ",
    }]);
    editor.setPosition(new monaco.Position(position.lineNumber, column + 4));
    editor.pushUndoStop();
  }
}

function removePipe(editor: monaco.editor.IStandaloneCodeEditor) {
  const selection = editor.getSelection();
  const model = editor.getModel();

  if (selection.isEmpty()) {
    const position = editor.getPosition();
    const lineContent = model.getLineContent(position.lineNumber);

    const column = lineContent.length - lineContent.split("").reverse().findIndex((v) => v == '|');
    if (column > 3) {
      editor.pushUndoStop();
      editor.executeEdits("remove-pipe", [{
        range: new monaco.Range(
          position.lineNumber,
          column - 2,
          position.lineNumber,
          column,
        ),
        text: "",
      }]);
      editor.setPosition(new monaco.Position(position.lineNumber, column));
      editor.pushUndoStop();
    }
  }
}

function getLineByMonacoNumber(monacoLineNr: number) {
  return model.getValue().split("\n")[monacoLineNr - 1];
}

function getLineDepth(line: string) {
  return line ? line.split("|").length - 1 : 1;
}

function isFitchBar(line: string) {
  return line.includes("|-");
}

function getLineType(moncaoLineNr: number) {
  let currentLineNr = moncaoLineNr;
  const initialLine = getLineByMonacoNumber(currentLineNr);
  const initialDepth = getLineDepth(initialLine);
  if (isFitchBar(initialLine)) return "fitchbar";
  while (currentLineNr > 0) {
    const line = getLineByMonacoNumber(currentLineNr);
    const depth = getLineDepth(line);
    if (depth < initialDepth) return "premise";
    if (depth > initialDepth) return "conclusion";
    if (isFitchBar(line) && depth == initialDepth) return "conclusion";
    currentLineNr--;
  }
  return "premise";
}

function insertNewline(
  editor: monaco.editor.IStandaloneCodeEditor,
  shiftPressed: boolean,
) {
  const selection = editor.getSelection();
  let lineNumberOffset = 1;

  if (selection.isEmpty()) {
    // Single cursor - insert pipe at cursor position
    const pos = editor.getPosition();
    let lineNumber = findNumberedLineUp(pos.lineNumber);
    if (!lineNumber) lineNumber = 0;
    const line = getLineByMonacoNumber(pos.lineNumber);
    const depth = getLineDepth(line);
    const lineType = getLineType(pos.lineNumber);

    let text = `\n${lineNumber + 1} ${"| ".repeat(depth)}`;

    if (lineType == "premise") {
      if (depth > 1 && !shiftPressed || shiftPressed && depth <= 1) {
        lineNumberOffset = 2;
        text = `\n ${" |".repeat(depth)}---${text}`;
      }
    } else {
      if (shiftPressed) {
        text = `\n${" ".repeat(lineNumber.toString().length)} ${"| ".repeat(depth)
          }`;
      }
    }

    // Push an undo stop before the edit to ensure proper undo behavior
    editor.pushUndoStop();

    editor.executeEdits("insert-after-newline", [{
      range: new monaco.Range(
        pos.lineNumber,
        999 | pos.column,
        pos.lineNumber,
        999 | pos.column,
      ),
      text,
    }]);

    // Push another undo stop after the edit to create a discrete undo operation
    editor.pushUndoStop();

    editor.setPosition(
      new monaco.Position(
        pos.lineNumber + lineNumberOffset,
        999 | pos.column,
      ),
    );
  } else {
    console.log("selection not implemented");
  }
}

// Create Error Lens instance
// const errorLens = new MonacoErrorLens(editor, monaco, {
//   enabled: false,
//   enableInlineMessages: true,
//   enableLineHighlights: true,
//   enableGutterIcons: true,
//   followCursor: "allLines", // Only show diagnostics for current line
//   // messageTemplate: 'hi [{source}] {message}', // Custom message format
//   messageTemplate: "hi",
//   maxMessageLength: 1000, // Truncate long messages
//   updateDelay: 200, // Debounce delay in ms
//   colors: {
//     error: {
//       background: "rgba(255, 0, 0, 0.1)",
//       foreground: "#ff4444",
//     },
//     warning: {
//       background: "rgba(255, 165, 0, 0.1)",
//       foreground: "#ff8800",
//     },
//   },
// });

function getEditorLineNumber(fitchLine: number) {
  return editor.getValue().split("\n").findIndex((l) =>
    l.startsWith(fitchLine.toString())
  ) + 1;
}
export function process_user_input(firstRun = false) {
  replace_words_by_fancy_symbols();

  const allowedVariableNamesField = document.getElementById(
    "allowed-variable-names",
  );
  if (!(allowedVariableNamesField instanceof HTMLInputElement)) {
    throw new Error(`allowed variable names field is of wrong node type`);
  }

  const res = check_proof(
    editor.getValue(),
    allowedVariableNamesField.value,
  );
  if (res.startsWith("The proof is correct!")) {
    document.getElementById("feedback").style.background = "green";
  } else if (res.startsWith("Fatal error")) {
    document.getElementById("feedback").style.background = "red";
  } else {
    document.getElementById("feedback").style.background = "#f05a1f";
  }
  document.getElementById("feedback").innerText = res;
  const matches = res.match(/(?:line\s+)(\d+)/i);
  if (matches) {
    const editorLine = getEditorLineNumber(Number(matches[1]));
    const markers: monaco.editor.IMarkerData[] = [
      {
        message: res,
        severity: monaco.MarkerSeverity.Error,
        startLineNumber: editorLine,
        startColumn: 1,
        endLineNumber: editorLine,
        endColumn: 100,
      },
    ];
    monaco.editor.setModelMarkers(model, "owner", markers);
    // editor.setModelMarkers()
  } else {
    monaco.editor.setModelMarkers(model, "owner", []);
  }


  const lastLineNr = model.getFullModelRange().endLineNumber;

  const premises = [];
  for (let lineNr = 1; lineNr < lastLineNr; lineNr++) {
    const line = getLineByMonacoNumber(lineNr);
    if (isFitchBar(line)) break;
    const content = line.split('|').at(-1).trimStart();
    if (content) premises.push(content);
  }

  // check if proof target was reached
  const checkRes: string = check_proof_with_template(
    model.getValue(),
    [...premises, proofTarget],
    allowedVariableNamesField.value
  );

  if (checkRes.includes('correct') && !confettiPlayed) {
    confettiPlayed = true;
    console.log('yes');
    if (!firstRun) {
      console.log('fr');
      tsParticles.load({
        id: "tsparticles",

        options: {
          emitters: {
            position: { x: 50, y: 100 },
            rate: { quantity: 100, delay: 0.05 },
            life: { duration: 1, count: 1 }
          },
          particles: {
            move: {
              direction: "top",
              enable: true,
              outModes: "destroy",
              speed: { min: 10, max: 20 },
              gravity: { enable: true, acceleration: 9.8 }
            },
            number: { value: 0 },
            opacity: { value: 1 },
            shape: { type: ["square", "circle"] },
            size: { value: { min: 3, max: 8 } },
            color: { value: ["#FF6B6B", "#4ECDC4", "#FFD93D", "#6BCF7F", "#C77DFF"] },
            life: { duration: { value: 3 } },
            rotate: {
              value: { min: 0, max: 360 },
              direction: "random",
              animation: { enable: true, speed: 30 }
            }
          }
        }
      });
    }
  }

}

function format() {
  const formatted = format_proof(editor.getValue());
  if (formatted == "invalid") {
    // alert("proof invalid cannot format");
    const feedbackEl = document.getElementById("feedback");
    feedbackEl.classList.remove("wiggle");
    feedbackEl.offsetHeight;
    feedbackEl.classList.add("wiggle");
    return;
  }

  const selection = editor.getSelection();
  const pos = editor.getPosition();

  editor.executeEdits("format-source", [{
    range: editor.getModel().getFullModelRange(),
    text: formatted,
  }]);
  // selection.
  editor.setSelection(selection);
  // Move to end of current line
  editor.setPosition({
    lineNumber: pos.lineNumber,
    column: editor.getModel().getLineMaxColumn(pos.lineNumber),
  });
  process_user_input();
}

function fix_line_numbers() {
  const fixed = fix_line_numbers_in_proof(editor.getValue());

  editor.executeEdits("format-source", [{
    range: editor.getModel().getFullModelRange(),
    text: fixed,
  }]);
  process_user_input();
}

function to_latex() {
  const latex = export_to_latex(editor.getValue());
  sessionStorage.setItem("latex-exported-proof", latex);
  window.open("latex_export", "_blank");
}

let advanced_settings_are_visible = false;
function toggle_show_advanced_settings() {
  advanced_settings_are_visible = !advanced_settings_are_visible;
  document.getElementById("advanced-settings").hidden =
    !advanced_settings_are_visible;
}

let examples_are_visible = false;
function show_examples() {
  examples_are_visible = !examples_are_visible;
  document.getElementById("additional-examples").hidden = !examples_are_visible;
}
export function load_example(index: number) {
  let rdy = model.getValue() == "";
  if (!rdy) {
    rdy = confirm(
      "Your proof area is not empty. Loading an example will overwrite your current proof. Are you sure you want to continue?",
    );
  }
  if (rdy) {
    editor.setValue(examples[index]);
    process_user_input();
  }
}

window.load_example = load_example;

function load_random_excercise() {
  const excercise = excercises[(Math.random() * excercises.length) | 0];
  proofTarget = excercise.conclusion;
  proofTargetEl.value = proofTarget;
  confettiPlayed = false;

  let assumptionsCompiled = "";
  for (let i = 0; i < excercise.assumptions.length; i++) {
    assumptionsCompiled += `${i + 1} | ${excercise.assumptions[i]}\n`;
  }
  assumptionsCompiled += "  |----";
  model.setValue(assumptionsCompiled);

}

let proof_is_upside_down = false;
function upside_down() {
  proof_is_upside_down = !proof_is_upside_down;
  if (proof_is_upside_down) {
    document.getElementById("editor_container").classList.add("rotate-180");
  } else {
    document.getElementById("editor_container").classList.remove("rotate-180");
  }
}

const replacements = {
  "fa": "∀",
  "ex": "∃",
  "not": "¬",
  "neg": "¬",
  "!": "¬",
  "impl": "→",
  "->": "→",
  "bic": "↔",
  "and": "∧",
  "&": "∧",
  "*": "∧",
  "or": "∨",
  "+": "∨",
  "bot": "⊥",
};


function replaceWithSymbols(input: string) {
  let offset = -1;
  for (const [token, replacement] of Object.entries(replacements)) {
    // we obnly ever have one token at the time (I hope). calculate the offset and replace it
    if (input.includes(token)) {
      offset = token.length - 1;
      return { result: input.replace(token, replacement), offset };
    }
  }
  return { result: input, offset };
}

// when user types e.g. 'forall', replace it instantly with the proper forall unicode symbol, and
// keep the user's cursor at the correct position so that user can continue typing.
//
function replace_words_by_fancy_symbols() {
  let proofstr = editor.getValue();
  let offset: number;
  // eslint-disable-next-line prefer-const
  ({ result: proofstr, offset } = replaceWithSymbols(proofstr));
  if (offset == -1) {
    return;
  }

  const pos = editor.getPosition();

  editor.executeEdits("format-source", [{
    range: editor.getModel().getFullModelRange(),
    text: proofstr,
  }]);

  setTimeout(() => {
    const newPos = pos.with(undefined, pos.column - (offset - 1));
    editor.setPosition(newPos);
  }, 1);
}

// Download proof as .txt file
function download_proof() {
  const element = document.createElement("a");
  const blob = new Blob([editor.getValue()], { type: "plain/text" });
  const fileUrl = URL.createObjectURL(blob);
  element.setAttribute("href", fileUrl);
  element.setAttribute("download", "proof.txt");
  element.style.display = "none";
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
}

function findNumberedLineUp(monacoLineNumber: number): number | null {
  let lineNumber = null;
  let monacoLineNumber2 = monacoLineNumber;
  while (!lineNumber && monacoLineNumber2 > 0) {
    const line = model.getValue().split("\n")[monacoLineNumber2 - 1];
    lineNumber = parseInt(line.split(" ")[0]);
    monacoLineNumber2 -= 1;
  }

  if (isNaN(lineNumber)) return null;
  return lineNumber;
}

// Listen to content changes (fires on every keystroke)
model.onDidChangeContent((_event: monaco.editor.IModelContentChangedEvent) => {
  process_user_input();
});

document.getElementById("format-button").onclick = format;
document.getElementById("latex-button").onclick = to_latex;
document.getElementById("load-example-button").onclick = show_examples;
document.getElementById("load-excercise-button").onclick = load_random_excercise;
document.getElementById("download-button").onclick = download_proof;
document.getElementById("upside-down-button").onclick = upside_down;
document.getElementById("fix-line-numbers-button").onclick = fix_line_numbers;
document.getElementById("allowed-variable-names").onkeyup = () => process_user_input();
document.getElementById("settings-button").onclick =
  toggle_show_advanced_settings;

proofTargetEl.addEventListener("keyup", function(e) {
  const raw = (e.target as HTMLInputElement).value;
  const replaced = replaceWithSymbols(raw).result;
  proofTarget = replaced;
  proofTargetEl.value = replaced;
  confettiPlayed = false;
});

await init();
process_user_input(true);
