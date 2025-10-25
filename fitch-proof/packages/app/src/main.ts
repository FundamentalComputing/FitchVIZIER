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

import Alpine from 'alpinejs';
import { languagedef, theme } from "./languagedef.ts";
import { getEditorLineNumber, getFile, getLineByMonacoNumber, getLineDepth, getLineType, isFitchBar, replaceWithSymbols } from "./helpers.ts";
import { confettiConfig } from "./confetti.ts";
// window.Alpine = Alpine;

interface TabsStore {
  files: { model: monaco.editor.IModel, name: string }[];
  current: number;
}

// Extend Alpine's Stores interface
declare module 'alpinejs' {
  interface Stores {
    tabs: TabsStore;
  }
}

const initContent = `1 | A
  |----
2 | A           Reit: 1`;
const uri = monaco.Uri.parse("inmemory://test");
const initModel = monaco.editor.createModel(initContent, "fitch", uri);

Alpine.store('tabs', {
  current: 0,
  files: [{ model: initModel, name: 'new.fitch' }],
});


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
  mimetypes: ["text/plain"],
});

// Define the syntax highlighting rules and color theme
monaco.languages.setMonarchTokensProvider("fitch", languagedef);
monaco.editor.defineTheme("fitch-theme", theme);

const editor = monaco.editor.create(document.getElementById("editor"), {
  model: initModel,
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


async function loadFileIntoMonaco(file) {
  // Read the file content
  const content = await file.text();

  // Create a Monaco URI
  const uri = monaco.Uri.parse(`file:///${file.name}`);

  // Check if a model already exists for this URI
  let model = monaco.editor.getModel(uri);

  if (model) {
    // Update existing model
    model.setValue(content);
  } else {
    // Create new model
    model = monaco.editor.createModel(
      content,
      undefined, // language (auto-detected from file extension)
      uri
    );
  }

  return model;
}


async function openFile() {
  const file = await getFile();
  if (file) {
    const model = await loadFileIntoMonaco(file);
    // editor.setModel(model);
    const len = Alpine.store("tabs").files.push({ model, name: file.name });
    Alpine.store("tabs").current = len - 1;
  }
}


let newFileCounter = 1;
function newFile(content?: string) {
  const uri = monaco.Uri.parse(`inmemory://new-${newFileCounter}.fitch`);
  const model = monaco.editor.createModel(content || initContent, "fitch", uri);
  const len = Alpine.store("tabs").files.push({ model, name: `new-${newFileCounter}.fitch` });
  Alpine.store("tabs").current = len - 1;
  newFileCounter++;
}


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
  newFile(assumptionsCompiled);
}

function insertPipe(editor: monaco.editor.IStandaloneCodeEditor) {
  const selection = editor.getSelection();
  // const model = editor.getModel();

  if (selection.isEmpty()) {
    const position = editor.getPosition();
    const lineContent = editor.getModel().getLineContent(position.lineNumber);

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
    const line = getLineByMonacoNumber(editor.getValue(), pos.lineNumber);
    const depth = getLineDepth(line);
    const lineType = getLineType(editor.getValue(), pos.lineNumber);

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

export function process_user_input(firstRun = false) {
  const model = editor.getModel();
  const editorValue = editor.getValue();
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
    const editorLine = getEditorLineNumber(editorValue, Number(matches[1]));
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
    const line = getLineByMonacoNumber(editorValue, lineNr);
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
      tsParticles.load(confettiConfig);
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
  newFile(examples[index]);
}
window.load_example = load_example;

let proof_is_upside_down = false;
function upside_down() {
  proof_is_upside_down = !proof_is_upside_down;
  if (proof_is_upside_down) {
    document.getElementById("editor_container").classList.add("rotate-180");
  } else {
    document.getElementById("editor_container").classList.remove("rotate-180");
  }
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
    const line = editor.getModel().getValue().split("\n")[monacoLineNumber2 - 1];
    lineNumber = parseInt(line.split(" ")[0]);
    monacoLineNumber2 -= 1;
  }

  if (isNaN(lineNumber)) return null;
  return lineNumber;
}

// Listen to content changes (fires on every keystroke)
editor.onDidChangeModelContent((_event: monaco.editor.IModelContentChangedEvent) => {
  process_user_input();
});
editor.onDidChangeModel((_event) => {
  process_user_input();
});

// bottom bar
document.getElementById("format-button").onclick = format;
document.getElementById("latex-button").onclick = to_latex;
document.getElementById("load-example-button").onclick = show_examples;
document.getElementById("load-excercise-button").onclick = load_random_excercise;
document.getElementById("download-button").onclick = download_proof;
document.getElementById("upside-down-button").onclick = upside_down;
document.getElementById("fix-line-numbers-button").onclick = fix_line_numbers;
document.getElementById("allowed-variable-names").onkeyup = () => process_user_input();
document.getElementById("settings-button").onclick = toggle_show_advanced_settings;

// tab actions
document.getElementById("file_open").onclick = openFile;
document.getElementById("file_new").onclick = newFile;

proofTargetEl.addEventListener("keyup", function(e) {
  const raw = (e.target as HTMLInputElement).value;
  const replaced = replaceWithSymbols(raw).result;
  proofTarget = replaced;
  proofTargetEl.value = replaced;
  confettiPlayed = false;
});


Alpine.start();


Alpine.effect(() => {
  const storeData = Alpine.store("tabs");
  console.log("tab change", storeData.current);
  editor.setModel(monaco.editor.getModels()[storeData.current]);
});

await init();
process_user_input(true);
