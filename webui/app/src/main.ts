// @ts-ignore
import "@fontsource-variable/fira-code";
// import "./style.css";
import {
  export_to_latex,
} from "@workspace/library";
import * as monaco from "monaco-editor";
import { tsParticles } from "@tsparticles/engine";
import "@tsparticles/preset-confetti";
import { loadConfettiPreset } from '@tsparticles/preset-confetti';
import examples from "./examples.ts";
import excercises from "./excercises.ts";

import Alpine from 'alpinejs';
import { languagedef, theme, lightTheme } from "./languagedef.ts";
import * as helpers from "./helpers.ts";
import { initStores } from "./stores.ts";
import { loadFromLocalStorage, saveToLocalStorage } from "./persistence.ts";
import { closeTab, initContent, newFile, openFile, renameTab } from "./files.ts";
import { editor, fix_line_numbers, format, initEditor, process_user_input } from "./editor.ts";

initStores();

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
window.editor = editor;
window.closeTab = closeTab;
window.renameTab = renameTab;

const uri = monaco.Uri.parse("inmemory://" + helpers.makeUUID());
const initModel = monaco.editor.createModel(initContent, "fitch", uri);

Alpine.store('tabs').current = 0;
Alpine.store('tabs').files = [{ name: 'new.txt', proofTarget: "", confettiPlayed: false, uri }];

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

initEditor();
editor.setModel(initModel);

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
  const x = helpers.replaceWithSymbols(raw);

  proofTargetEl.value = x.result;

  const currentTab = Alpine.store("tabs").current;
  Alpine.store("tabs").files[currentTab].proofTarget = x.result;
  Alpine.store("tabs").files[currentTab].confettiPlayed = false;

  // Restore cursor position
  const newPos = x.offset == -1 ? cursorPos : cursorPos - x.offset;
  input.setSelectionRange(newPos, newPos);
});

Alpine.start();
Alpine.effect(() => {
  Alpine.store("tabs").files;
  saveToLocalStorage();
});

Alpine.effect(() => {
  const current = Alpine.store("tabs").current;
  editor.setModel(monaco.editor.getModels()[current]);
});

window.addEventListener('storage', () => loadFromLocalStorage(), false); // listen for changes from other tabs
loadFromLocalStorage();
const current = Alpine.store("tabs").current;
editor.setModel(monaco.editor.getModels()[current]);

process_user_input(true);
