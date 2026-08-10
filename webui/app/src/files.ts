import Alpine from "alpinejs";
import * as monaco from "monaco-editor";
import { getFile, makeUUID } from "./helpers";
import { saveToLocalStorage } from "./persistence";
import { editor } from "./editor";

export const initContent = `1 | A
  |----
2 | A           Reit: 1`;

export function closeTab(index: number) {
  const files = Alpine.store("tabs").files;
  if (files.length < 2) {
    return;
  }
  const removedFile = files[index];
  const current = Alpine.store("tabs").current;
  files.splice(index, 1);
  Alpine.store("tabs").current = current > index ? current - 1 : Math.min(current, files.length - 1);

  setTimeout(() => { // it errors without this and im too tired to fix it properly
    const stillOpen = files.some(file => file.uri.toString() == removedFile.uri.toString());
    if (!stillOpen) {
      monaco.editor.getModel(removedFile.uri)?.dispose();
    }
    const currentFile = files[Alpine.store("tabs").current];
    if (currentFile) {
      editor.setModel(monaco.editor.getModel(currentFile.uri));
    }
  }, 100);
}

export function renameTab(index: number) {
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

export async function loadFileIntoMonaco(file: File) {
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

export async function openFile() {
  const file = await getFile();
  if (file) {
    const uri = await loadFileIntoMonaco(file);
    const existingTab = Alpine.store("tabs").files.findIndex(
      tab => tab.uri.toString() == uri.toString()
    );
    if (existingTab >= 0) {
      Alpine.store("tabs").current = existingTab;
      saveToLocalStorage();
      return;
    }
    const len = Alpine.store("tabs").files.push({ proofTarget: "", name: file.name, uri });
    Alpine.store("tabs").current = len - 1;
  }
  saveToLocalStorage();
}


export function newFile(content?: string) {
  const uri = monaco.Uri.parse(`inmemory://${makeUUID()}`);
  monaco.editor.createModel(content ?? initContent, "fitch", uri);
  const len = Alpine.store("tabs").files.push({
    name: `new-${Alpine.store("newFileCounter").value}.txt`, proofTarget: "", uri
  });
  Alpine.store("tabs").current = len - 1;
  Alpine.store("newFileCounter").inc();
}
