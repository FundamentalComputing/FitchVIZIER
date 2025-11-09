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
import examples from "./examples.ts";
import excercises from "./excercises.ts";

import Alpine from 'alpinejs';
import { languagedef, theme, lightTheme } from "./languagedef.ts";
import {
  getEditorLineNumber, getFile, getLineByMonacoNumber, getLineDepth,
  getLineType, isFitchBar, makeUUID, replaceWithSymbols
} from "./helpers.ts";
import { confettiConfig } from "./confetti.ts";


declare global {
  interface Window {
    editor: monaco.editor.IStandaloneCodeEditor;
    load_example: (index: number) => void;
    closeTab: (index: number) => void;
    renameTab: (index: number) => void;
    Alpine: Alpine.Alpine
  }
}

window.Alpine = Alpine;

interface TabsStore {
  files: { name: string, uri: monaco.Uri, proofTarget: string, confettiPlayed: boolean }[];
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
const uri = monaco.Uri.parse("inmemory://" + makeUUID());
const initModel = monaco.editor.createModel(initContent, "fitch", uri);

Alpine.store('tabs', {
  current: 0,
  files: [{ name: 'new.txt', proofTarget: "", confettiPlayed: false, uri }],
});


loadConfettiPreset(tsParticles);

monaco.languages.register({
  id: "fitch",
  extensions: [".fitch", ".txt"],
  aliases: ["Fitch Proof", "fitch"],
  mimetypes: ["text/plain"],
});

// Define the syntax highlighting rules and color theme
monaco.languages.setMonarchTokensProvider("fitch", languagedef);
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)');
monaco.editor.defineTheme("fitch-theme", prefersDark.matches ? theme : lightTheme);


window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (ev: MediaQueryListEvent) => {
  const isDark = ev.matches;
  monaco.editor.defineTheme("fitch-theme", isDark ? theme : lightTheme);
});

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
  saveToLocalStorage();
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


function saveToLocalStorage() {
  const storeData = Alpine.store("tabs");

  // save to localstorage
  if (!hasLoadedLocalStorage) {
    hasLoadedLocalStorage = true;
    return;
  }
  const data = storeData.files.map((file) => {
    const content = monaco.editor.getModel(file.uri).getValue();

    return { ...file, content, uri: file.uri.toString() };
  });

  localStorage.setItem("tabs", JSON.stringify({ files: data }));
  // editor.setModel(monaco.editor.getModels()[storeData.current]);
  console.log("saved!");
}

function loadFromLocalStorage() {
  const importedData = JSON.parse(localStorage.getItem("tabs")) as Omit<TabsStore, 'current'>;
  if (!importedData.files) {
    return;
  }
  console.log("data is in right format");
  monaco.editor.getModels().forEach(m => m.dispose());
  const newTabsData: TabsStore = { current: 0, files: [] };
  let highestNewFile = 1;
  for (const tab of importedData.files) {
    // @ts-ignore
    const uri = monaco.Uri.parse(tab.uri);
    // @ts-ignore 
    monaco.editor.createModel(tab.content, "fitch", uri);
    newTabsData.files.push({
      name: tab.name,
      confettiPlayed: tab.confettiPlayed,
      proofTarget: tab.proofTarget,
      uri
    });

    if (tab.name.startsWith("new-")) {
      const n = parseInt(tab.name.slice(4));
      if (n > highestNewFile) highestNewFile = n;
    }
  }

  if (newTabsData.current >= newTabsData.files.length - 1) {
    Alpine.store("tabs").current = 0;

  }
  Alpine.store("tabs").files = newTabsData.files;
  newFileCounter = highestNewFile + 1;
  console.log("loaded tabs");
}

function closeTab(index: number) {
  const files = Alpine.store("tabs").files;
  if (files.length < 2) {
    return;
  }
  Alpine.store("tabs").files.splice(index, 1);
  Alpine.store("tabs").current = 0;

  setTimeout(() => { // it errors without this and im too tired to fix it properly
    monaco.editor.getModels()[index].dispose();
    editor.setModel(monaco.editor.getModels()[0]);
  }, 100);
}
window.closeTab = closeTab;

function renameTab(index: number) {
  const files = Alpine.store("tabs").files;
  const oldName = files[index].name;
  const newName = prompt('enter new name', oldName);
  if (!newName) return;
  if (newName == oldName) return;
  if (files.find(f => f.name == newName)) {
    alert("File already exists");
    return;
  }
  files[index].name = newName;
}
window.renameTab = renameTab;

async function loadFileIntoMonaco(file: File) {
  const content = await file.text();
  const uri = monaco.Uri.parse(`file:///${file.name}`);
  let model = monaco.editor.getModel(uri);
  if (model) {
    model.setValue(content);
  } else {
    model = monaco.editor.createModel(
      content,
      "fitch",
      uri
    );
  }
  return uri;
}

async function openFile() {
  const file = await getFile();
  if (file) {
    const uri = await loadFileIntoMonaco(file);
    const len = Alpine.store("tabs").files.push({ proofTarget: "", confettiPlayed: false, name: file.name, uri });
    Alpine.store("tabs").current = len - 1;
  }
  saveToLocalStorage();
}


let newFileCounter = 1;
function newFile(content?: string) {
  const uri = monaco.Uri.parse(`inmemory://${makeUUID()}`);
  monaco.editor.createModel(content ?? initContent, "fitch", uri);
  const len = Alpine.store("tabs").files.push({
    name: `new-${newFileCounter}.txt`, proofTarget: "", confettiPlayed: false, uri
  });
  Alpine.store("tabs").current = len - 1;
  newFileCounter++;
}


function load_random_excercise() {
  const excercise = excercises[(Math.random() * excercises.length) | 0];

  let assumptionsCompiled = "";
  for (let i = 0; i < excercise.assumptions.length; i++) {
    assumptionsCompiled += `${i + 1} | ${excercise.assumptions[i]}\n`;
  }
  assumptionsCompiled += "  |----";
  newFile(assumptionsCompiled);
  Alpine.store("tabs").files[Alpine.store("tabs").current].proofTarget = excercise.conclusion;
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
  }
}

export function process_user_input(firstRun = false) {
  let model = editor.getModel();
  if (!model) {
    editor.setModel(monaco.editor.getModels()[Alpine.store("tabs").current]);
    model = editor.getModel();
  };
  const editorValue = editor.getValue();
  replace_words_by_fancy_symbols();

  const allowedVariableNamesField = document.getElementById("allowed-variable-names");
  if (!(allowedVariableNamesField instanceof HTMLInputElement)) {
    throw new Error(`allowed variable names field is of wrong node type`);
  }

  const res = check_proof(editor.getValue(), allowedVariableNamesField.value);
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
  const tab = Alpine.store("tabs").files[Alpine.store("tabs").current];
  const checkRes: string = check_proof_with_template(
    model.getValue(),
    [...premises, tab.proofTarget],
    allowedVariableNamesField.value
  );

  if (checkRes.includes('correct') && !tab.confettiPlayed) {
    tab.confettiPlayed = true;
    if (!firstRun) {
      tsParticles.load(confettiConfig);
    }
  }
}

function format() {
  const formatted = format_proof(editor.getValue());
  if (formatted == "invalid") {
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
    editor.setPosition(pos.with(undefined, pos.column - (offset)));
  }, 1);
}

// Download proof as .txt file
function download_proof() {
  const element = document.createElement("a");
  const blob = new Blob([editor.getValue()], { type: "plain/text" });
  const fileUrl = URL.createObjectURL(blob);
  element.setAttribute("href", fileUrl);
  element.setAttribute("download", Alpine.store("tabs").files[Alpine.store("tabs").current].name);
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
editor.onDidChangeModelContent(() => {
  process_user_input();
  saveToLocalStorage();

});
editor.onDidChangeModel(() => editor.getModel() && process_user_input());

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
document.getElementById("file_new").onclick = () => newFile();

const proofTargetEl = document.getElementById("proof_target") as HTMLInputElement;

proofTargetEl.addEventListener("keyup", function(e) {
  const input = e.target as HTMLInputElement;
  const cursorPos = input.selectionStart;
  const cursorPosEnd = input.selectionEnd;
  if (cursorPosEnd != cursorPos) {
    return;
  }
  const raw = input.value;
  const x = replaceWithSymbols(raw);

  proofTargetEl.value = x.result;

  const currentTab = Alpine.store("tabs").current;
  Alpine.store("tabs").files[currentTab].proofTarget = x.result;
  Alpine.store("tabs").files[currentTab].confettiPlayed = false;

  // Restore cursor position
  const newPos = x.offset == -1 ? cursorPos : cursorPos - x.offset;
  input.setSelectionRange(newPos, newPos);
});

let hasLoadedLocalStorage = false;
Alpine.start();


Alpine.effect(() => {
  Alpine.store("tabs").files;
  saveToLocalStorage();
});


Alpine.effect(() => {
  const current = Alpine.store("tabs").current;
  editor.setModel(monaco.editor.getModels()[current]);
});


await init();
window.addEventListener('storage', () => loadFromLocalStorage(), false);
loadFromLocalStorage();
const current = Alpine.store("tabs").current;
editor.setModel(monaco.editor.getModels()[current]);

process_user_input(true);
