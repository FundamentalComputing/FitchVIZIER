export function makeUUID() {
  var S4 = function() {
    return (((1 + Math.random()) * 0x10000) | 0).toString(16).substring(1);
  };
  return (S4() + S4() + "-" + S4() + "-" + S4() + "-" + S4() + "-" + S4() + S4() + S4());
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
  "\u200B": " ",
};


export function replaceWithSymbols(input: string) {
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



export function findNumberedLineUp(editorValue: string, monacoLineNumber: number): number | null {
  let lineNumber = null;
  let monacoLineNumber2 = monacoLineNumber;
  while (!lineNumber && monacoLineNumber2 > 0) {
    const line = editorValue.split("\n")[monacoLineNumber2 - 1];
    lineNumber = parseInt(line.split(" ")[0]);
    monacoLineNumber2 -= 1;
  }

  if (isNaN(lineNumber)) return null;
  return lineNumber;
}

export function getEditorLineNumber(editorValue: string, fitchLine: number) {
  return editorValue.split("\n").findIndex((l) =>
    l.startsWith(fitchLine.toString())
  ) + 1;
}


export function getLineByMonacoNumber(editorValue: string, monacoLineNr: number) {
  return editorValue.split("\n")[monacoLineNr - 1];
}

export function getLineDepth(line: string) {
  return line ? line.split("|").length - 1 : 1;
}

export function isFitchBar(line: string) {
  return line.includes("|-");
}

export function getLineType(editorValue: string, moncaoLineNr: number) {
  let currentLineNr = moncaoLineNr;
  const initialLine = getLineByMonacoNumber(editorValue, currentLineNr);
  const initialDepth = getLineDepth(initialLine);
  if (isFitchBar(initialLine)) return "fitchbar";
  while (currentLineNr > 0) {
    const line = getLineByMonacoNumber(editorValue, currentLineNr);
    const depth = getLineDepth(line);
    if (depth < initialDepth) return "premise";
    if (depth > initialDepth) return "conclusion";
    if (isFitchBar(line) && depth == initialDepth) return "conclusion";
    currentLineNr--;
  }
  return "premise";
}

export async function getFile(): Promise<File> {
  // Check if File System Access API is available
  if ('showOpenFilePicker' in window) {
    console.log("using modern api");
    try {
      const [fileHandle] = await window.showOpenFilePicker(
        { types: [{ accept: { "text/plain": ['.txt', '.fitch'] } }] }
      );
      const file = await fileHandle.getFile();
      return file;
    } catch (err) {
      if (err.name === 'AbortError') {
        return null; // User cancelled
      }
      throw err;
    }
  } else {
    // Fallback for Firefox and other browsers
    return new Promise((resolve) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = ".txt,.fitch";
      input.onchange = (e) => {
        const target = e.target as HTMLInputElement;
        const file = target.files[0];
        resolve(file || null);
      };
      input.click();
    });
  }
}

export function wiggleFeedback() {
  const feedbackEl = document.getElementById("feedback");
  feedbackEl.classList.remove("wiggle");
  feedbackEl.offsetHeight;
  feedbackEl.classList.add("wiggle");
}
