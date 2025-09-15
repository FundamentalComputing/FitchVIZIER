// @ts-nocheck
import '@fontsource-variable/fira-code';
import './style.css'
// @ts-ignore
import { init, check_proof, format_proof, fix_line_numbers_in_proof, export_to_latex } from '@workspace/library'
import * as monaco from 'monaco-editor';
import MonacoErrorLens from '@ym-han/monaco-error-lens';

monaco.languages.register({
  id: 'fitch',
  extensions: ['.fitch'],
  aliases: ['Fitch Proof', 'fitch'],
  mimetypes: ['text/fitch']
});

// Define the syntax highlighting rules
monaco.languages.setMonarchTokensProvider('fitch', {
  tokenizer: {
    root: [
      // Line numbers at start
      [/^\s*\d+/, 'line-number'],

      // Proof structure - vertical bars and dashes
      [/\s*\|\s*/, 'proof-structure'],
      [/\s*-+\s*/, 'proof-structure'],

      // Logical operators and symbols
      [/∧/, 'operator.conjunction'],
      [/∨/, 'operator.disjunction'],
      [/¬/, 'operator.negation'],
      [/→/, 'operator.implication'],
      [/↔/, 'operator.biconditional'],
      [/∀/, 'quantifier.universal'],
      [/∃/, 'quantifier.existential'],
      [/⊥/, 'operator.falsum'],
      [/⊤/, 'operator.verum'],
      [/=/, 'operator.equality'],
      [/≠/, 'operator.inequality'],

      // Parentheses and brackets
      [/[()[\]{}]/, 'delimiter'],

      // Predicate/function names (capital letters)
      [/[A-Z][a-zA-Z0-9]*/, 'predicate'],

      // Variables and constants (lowercase)
      [/[a-z][a-zA-Z0-9]*/, 'variable'],

      // Function applications like f(a), g(x,y)
      [/[a-z]+(?=\()/, 'function'],

      // Justifications - rule names
      [/\b(Reit|∧\s*Elim|∨\s*Elim|∧\s*Intro|∨\s*Intro|→\s*Elim|→\s*Intro|¬\s*Elim|¬\s*Intro|=\s*Elim|=\s*Intro|∀\s*Elim|∀\s*Intro|∃\s*Elim|∃\s*Intro|⊥\s*Elim|RAA|MT|DS|HS|Add|Simp|Conj|MP|DeM|DN|Com|Assoc|Dist|Exp|Equiv|Impl|Taut|Contra)\b/, 'rule-name'],

      // Line references in justifications (numbers, ranges)
      [/:\s*\d+/, 'justification.reference'],
      [/\d+-\d+/, 'justification.reference'],
      [/,\s*\d+/, 'justification.reference'],

      // Comma separator
      [/,/, 'delimiter'],

      // Whitespace
      [/\s+/, 'white']
    ]
  }
});

// Define the color theme
monaco.editor.defineTheme('fitch-theme', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'line-number', foreground: '888888', fontStyle: 'bold' },
    { token: 'proof-structure', foreground: '888888' },
    { token: 'operator.conjunction', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.disjunction', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.negation', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.implication', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.biconditional', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.falsum', foreground: 'FF4444', fontStyle: 'bold' },
    { token: 'operator.verum', foreground: '44FF44', fontStyle: 'bold' },
    { token: 'operator.equality', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'operator.inequality', foreground: 'CC6666', fontStyle: 'bold' },
    { token: 'quantifier.universal', foreground: '9966CC', fontStyle: 'bold' },
    { token: 'quantifier.existential', foreground: '9966CC', fontStyle: 'bold' },
    { token: 'predicate', foreground: '99CC99' },
    { token: 'function', foreground: 'CCCC66' },
    { token: 'variable', foreground: '99CCFF' },
    { token: 'rule-name', foreground: 'FF9966', fontStyle: 'italic' },
    { token: 'justification.reference', foreground: 'CCCCCC' },
    { token: 'delimiter', foreground: '#87875f' }
  ],
  colors: {
    'editor.background': '#1e1e1e'
  }
});

const value = `1 | A
  |----
2 | A           Reit: 1`;
const uri = monaco.Uri.parse("inmemory://test");
const model = monaco.editor.createModel(value, "fitch", uri);

const editor = monaco.editor.create(document.getElementById('editor'), {
  //   value: `1 | A
  //   |----
  // 2 | A           Reit: 1`,
  model,
  language: 'fitch',
  theme: 'fitch-theme',
  lineNumbers: false,
  automaticLayout: true,
  fontFamily: 'Fira Code Variable',
  fontLigatures: true,
  glyphMargin: true,
  minimap: { enabled: false },
  unicodeHighlight: {
    ambiguousCharacters: false,
    invisibleCharacters: true
  }
});


editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, function() {
  format()
});

// Create Error Lens instance
const errorLens = new MonacoErrorLens(editor, monaco, {
  enabled: false,
  enableInlineMessages: true,
  enableLineHighlights: true,
  enableGutterIcons: true,
  followCursor: 'allLines', // Only show diagnostics for current line
  // messageTemplate: 'hi [{source}] {message}', // Custom message format
  messageTemplate: 'hi',
  maxMessageLength: 1000, // Truncate long messages
  updateDelay: 200, // Debounce delay in ms
  colors: {
    error: {
      background: 'rgba(255, 0, 0, 0.1)',
      foreground: '#ff4444',
    },
    warning: {
      background: 'rgba(255, 165, 0, 0.1)',
      foreground: '#ff8800',
    }
  }
});


function getEditorLineNumber(fitchLine: number) {
  return editor.getValue().split("\n").findIndex(l => l.startsWith(fitchLine.toString())) + 1
}

export function process_user_input() {
  replace_words_by_fancy_symbols();

  console.log(editor.getValue())
  let res = check_proof(editor.getValue(), document.getElementById("allowed-variable-names").value);
  if (res.startsWith("The proof is correct!")) {
    document.getElementById("feedback").style.color = "green";
  } else if (res.startsWith("Fatal error")) {
    document.getElementById("feedback").style.color = "red";
  } else {
    document.getElementById("feedback").style.color = "#f05a1f";
  }
  document.getElementById("feedback").innerText = res;
  const matches = res.match(/(?:line\s+)(\d+)/i)
  if (matches) {
    const editorLine = getEditorLineNumber(Number(matches[1]))
    console.log(res, matches[1], editorLine);
    const markers: monaco.editor.IMarkerData[] = [
      {
        message: res,
        severity: monaco.MarkerSeverity.Error,
        startLineNumber: editorLine,
        startColumn: 1,
        endLineNumber: editorLine,
        endColumn: 100,
      }
    ]
    monaco.editor.setModelMarkers(model, "owner", markers);
    // editor.setModelMarkers()
  } else {
    monaco.editor.setModelMarkers(model, "owner", [])
  }
}

function format() {
  let formatted = format_proof(editor.getValue());

  let selection = editor.getSelection()
  let pos = editor.getPosition()

  editor.setValue(formatted)
  // selection.
  editor.setSelection(selection)
  // Move to end of current line
  editor.setPosition({
    lineNumber: pos.lineNumber,
    column: editor.getModel().getLineMaxColumn(pos.lineNumber)
  });
  // document.getElementById("proof-field").value = formatted;
  process_user_input();
}

function fix_line_numbers() {
  let fixed = fix_line_numbers_in_proof(editor.getValue());
  editor.setValue(fixed)
  // document.getElementById("proof-field").value = fixed;
  process_user_input();
}

function to_latex() {
  let latex = export_to_latex(editor.getValue());
  sessionStorage.setItem("latex-exported-proof", latex);
  window.open("latex_export", "_blank");
}

let advanced_settings_are_visible = false;
function toggle_show_advanced_settings() {
  advanced_settings_are_visible = !advanced_settings_are_visible;
  document.getElementById("advanced-settings").hidden = !advanced_settings_are_visible;
};

let examples_are_visible = false;
function show_examples() {
  examples_are_visible = !examples_are_visible;
  document.getElementById("additional-examples").hidden = !examples_are_visible;
}
export function load_example(ex) {
  let rdy = editor.getValue() == "";
  if (!rdy) {
    rdy = confirm("Your proof area is not empty. Loading an example will overwrite your current proof. Are you sure you want to continue?");
  }
  if (rdy) {
    console.log(ex);
    editor.setValue(ex);
    process_user_input();
  }
}
window.load_example = load_example;

let proof_is_upside_down = false;
function upside_down() {
  proof_is_upside_down = !proof_is_upside_down;
  if (proof_is_upside_down) {
    document.getElementById("proof-field").style.setProperty("-webkit-transform", "rotate(180deg)", null);
  } else {
    document.getElementById("proof-field").style.setProperty("-webkit-transform", "rotate(0deg)", null);
  }
}

// when user types e.g. 'forall', replace it instantly with the proper forall unicode symbol, and 
// keep the user's cursor at the correct position so that user can continue typing.

// this is horrible and should be rewritten based on string length i think
function replace_words_by_fancy_symbols() {
  let proofstr = editor.getValue();
  let offset = -1;
  if (proofstr.includes("fa")) {
    offset = 1;
  } else if (proofstr.includes("ex")) {
    offset = 1;
  } else if (proofstr.includes("not")) {
    offset = 2;
  } else if (proofstr.includes("neg")) {
    offset = 2;
  } else if (proofstr.includes("impl")) {
    offset = 3;
  } else if (proofstr.includes("bic")) {
    offset = 2;
  } else if (proofstr.includes("and")) {
    offset = 2;
  } else if (proofstr.includes("or")) {
    offset = 1;
  } else if (proofstr.includes("bot")) {
    offset = 2;
  }

  if (offset == -1) {
    return;
  }

  proofstr = proofstr.replace("fa", "∀").replace("ex", "∃").replace("not", "¬").replace("neg", "¬").replace("or", "∨")
    .replace("bot", "⊥").replace("bic", "↔").replace("impl", "→").replace("and", "∧");
  let oldSelectionIndex = editor.getSelection()
  editor.setValue(proofstr)
  editor.setSelection(oldSelectionIndex)
};

// Download proof as .txt file
function download_proof() {
  const element = document.createElement('a');
  const blob = new Blob([editor.getValue()], { type: 'plain/text' }); const fileUrl = URL.createObjectURL(blob);
  element.setAttribute("href", fileUrl);
  element.setAttribute("download", "proof.txt");
  element.style.display = 'none';
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
};

function findNumberedLineUp(monacoLineNumber: int) {
  let lineNumber = null;
  let monacoLineNumber2 = monacoLineNumber;
  while (!lineNumber && monacoLineNumber2 > 0) {
    const line = model.getValue().split("\n")[monacoLineNumber2 - 1]
    lineNumber = parseInt(line.split(" ")[0])
    monacoLineNumber2 -= 1;
  }

  return lineNumber
}


// Listen to content changes (fires on every keystroke)
model.onDidChangeContent((event) => {
  process_user_input()

  // Check if the change includes a new line
  event.changes.forEach(change => {
    if (change.text.includes('\n') && change.text.length < 3) {
      // A new line was inserted
      const position = editor.getPosition();

      // Insert something after the new line
      setTimeout(() => {
        const currentPosition = editor.getPosition();
        // const line = model.getValue().split("\n")[currentPosition.lineNumber - 2]
        // const lineNumber = parseInt(line.split("|")[0])
        const lineNumber = findNumberedLineUp(currentPosition.lineNumber - 1)
        const line = model.getValue().split("\n")[currentPosition.lineNumber - 2]
        const depth = line.split("|").length

        editor.executeEdits('insert-after-newline', [{
          range: new monaco.Range(
            currentPosition.lineNumber,
            currentPosition.column,
            currentPosition.lineNumber,
            currentPosition.column
          ),
          text: `${lineNumber + 1} ${Array.from(Array(depth)).join("| ")}`
        }]);
      }, 0);
    }
  });
});

document.getElementById("format-button").onclick = format;
document.getElementById("latex-button").onclick = to_latex;
document.getElementById("load-example-button").onclick = show_examples;
document.getElementById("download-button").onclick = download_proof;
document.getElementById("upside-down-button").onclick = upside_down;
document.getElementById("fix-line-numbers-button").onclick = fix_line_numbers;
document.getElementById("allowed-variable-names").onkeyup = process_user_input;
document.getElementById("settings-button").onclick = toggle_show_advanced_settings;




await init();
process_user_input();




