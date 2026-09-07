export const editorTextSizeStorageKey = "editor-text-size";
export const defaultEditorTextSize = 14;
export const minEditorTextSize = 12;
export const maxEditorTextSize = 24;

type TextSizeChangeHandler = (fontSize: number) => void;

let currentEditorTextSize = loadEditorTextSize();
let editorTextSizeControl: HTMLInputElement | null = null;
let editorTextSizeValue: HTMLOutputElement | null = null;
const changeHandlers = new Set<TextSizeChangeHandler>();

function normalizeEditorTextSize(fontSize: number): number {
  if (!Number.isFinite(fontSize)) {
    return defaultEditorTextSize;
  }
  return Math.min(maxEditorTextSize, Math.max(minEditorTextSize, Math.round(fontSize)));
}

function parseEditorTextSize(value: string | null): number {
  if (value === null) {
    return defaultEditorTextSize;
  }
  return normalizeEditorTextSize(Number(value));
}

function loadEditorTextSize(): number {
  try {
    return parseEditorTextSize(localStorage.getItem(editorTextSizeStorageKey));
  } catch {
    return defaultEditorTextSize;
  }
}

function saveEditorTextSize(fontSize: number) {
  try {
    localStorage.setItem(editorTextSizeStorageKey, fontSize.toString());
  } catch {
    // The preference still applies for this page when storage is unavailable.
  }
}

function syncControls() {
  if (editorTextSizeControl) {
    editorTextSizeControl.value = currentEditorTextSize.toString();
  }
  if (editorTextSizeValue) {
    editorTextSizeValue.value = `${currentEditorTextSize}px`;
  }
}

function notifyChangeHandlers() {
  for (const handler of changeHandlers) {
    handler(currentEditorTextSize);
  }
}

export function updateEditorTextSize(fontSize: number): number {
  currentEditorTextSize = normalizeEditorTextSize(fontSize);
  saveEditorTextSize(currentEditorTextSize);
  syncControls();
  notifyChangeHandlers();
  return currentEditorTextSize;
}

export function initializeEditorTextSize(onChange: TextSizeChangeHandler): number {
  changeHandlers.add(onChange);
  editorTextSizeControl = document.getElementById("editor-text-size") as HTMLInputElement | null;
  editorTextSizeValue = document.getElementById("editor-text-size-value") as HTMLOutputElement | null;

  if (editorTextSizeControl) {
    editorTextSizeControl.min = minEditorTextSize.toString();
    editorTextSizeControl.max = maxEditorTextSize.toString();
    editorTextSizeControl.step = "1";
    editorTextSizeControl.addEventListener("input", () => {
      updateEditorTextSize(Number(editorTextSizeControl!.value));
    });
  }

  window.addEventListener("storage", (event) => {
    if (event.key !== editorTextSizeStorageKey) {
      return;
    }
    currentEditorTextSize = parseEditorTextSize(event.newValue);
    syncControls();
    notifyChangeHandlers();
  });

  syncControls();
  onChange(currentEditorTextSize);
  return currentEditorTextSize;
}
