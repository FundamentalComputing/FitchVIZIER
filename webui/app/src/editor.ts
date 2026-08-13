import * as monaco from "monaco-editor";
import Alpine from "alpinejs";
import {
  findNumberedLineUp,
  getLineByMonacoNumber,
  getLineDepth,
  getLineType,
  replaceWithSymbols,
  wiggleFeedback
} from "./helpers";
import { saveToLocalStorage } from "./persistence";
import {
  check_proof_diagnostics,
  format_proof,
  fix_line_numbers_in_proof
} from "@workspace/library";

export let editor: monaco.editor.IStandaloneCodeEditor;


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
    let lineNumber = findNumberedLineUp(editor.getValue(), pos.lineNumber);
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
        text = `\n${" ".repeat(lineNumber.toString().length)} ${"| ".repeat(depth)}`;
      }
    }

    editor.pushUndoStop();
    editor.executeEdits("insert-after-newline", [{
      range: new monaco.Range(pos.lineNumber, 999 | pos.column, pos.lineNumber, 999 | pos.column),
      text,
    }]);
    editor.pushUndoStop();

    editor.setPosition(
      new monaco.Position(pos.lineNumber + lineNumberOffset, 999 | pos.column),
    );
  }
}

export function format() {
  const formatted = format_proof(editor.getValue());
  if (formatted == "invalid") {
    wiggleFeedback();
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



export function fix_line_numbers() {
  const fixed = fix_line_numbers_in_proof(editor.getValue());
  editor.executeEdits("format-source", [{
    range: editor.getModel().getFullModelRange(),
    text: fixed,
  }]);
  process_user_input();
}


// when user types e.g. 'forall', replace it instantly with the proper forall unicode symbol, and
// keep the user's cursor at the correct position so that user can continue typing.
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


export function process_user_input() {
  let model = editor.getModel();
  if (!model) {
    editor.setModel(monaco.editor.getModels()[Alpine.store("tabs").current]);
    model = editor.getModel();
  };
  replace_words_by_fancy_symbols();

  const allowedVariableNamesField = document.getElementById("allowed-variable-names");
  if (!(allowedVariableNamesField instanceof HTMLInputElement)) {
    throw new Error(`allowed variable names field is of wrong node type`);
  }

  const result = check_proof_diagnostics(editor.getValue(), allowedVariableNamesField.value);
  const res = result.status === "correct"
    ? "The proof is correct!"
    : result.diagnostics.map(({ message }) => message).join("\n\n");
  if (result.status === "correct") {
    document.getElementById("feedback").style.background = "green";
  } else if (result.status === "fatal") {
    document.getElementById("feedback").style.background = "red";
  } else {
    document.getElementById("feedback").style.background = "#f05a1f";
  }
  document.getElementById("feedback").innerText =
    result.status === "fatal" ? `Fatal error: ${res}` : res;
  const markers: monaco.editor.IMarkerData[] = result.diagnostics
    .filter(({ location }) => location !== null)
    .map(({ message, location }) => {
      const lineNumber = location!.line;
      const colNumber = location!.column;
      console.log(colNumber);
      return {
        message,
        severity: monaco.MarkerSeverity.Error,
        startLineNumber: lineNumber,
        startColumn: colNumber,
        endLineNumber: lineNumber,
        endColumn: model.getLineMaxColumn(lineNumber),
      };
    });
  monaco.editor.setModelMarkers(model, "owner", markers);

}

export function initEditor(theme: string) {
  editor = monaco.editor.create(document.getElementById("editor"), {
    // model: initModel,
    language: "fitch",
    theme,
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


  // Add zoom in/out shortcuts to Monaco
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Equal, () => {
    editor.updateOptions({ fontSize: editor.getOption(monaco.editor.EditorOption.fontSize) + 1 });
  });

  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Minus, () => {
    const currentSize = editor.getOption(monaco.editor.EditorOption.fontSize);
    if (currentSize > 1) {
      editor.updateOptions({ fontSize: currentSize - 1 });
    }
  });

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
}
