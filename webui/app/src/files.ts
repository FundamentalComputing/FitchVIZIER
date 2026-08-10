import Alpine from "alpinejs";
import * as monaco from "monaco-editor";
import { getFile, makeUUID } from "./helpers";
import { saveToLocalStorage } from "./persistence";

export const initContent = `1 | A
  |----
2 | A           Reit: 1`;

export function closeTab(index: number) {
  const tabs = Alpine.store("tabs");
  const files = tabs.files;
  if (files.length < 2 || index < 0 || index >= files.length) {
    return;
  }

  const [removedFile] = files.splice(index, 1);

  if (tabs.current > index) {
    tabs.current--;
  } else if (tabs.current === index) {
    tabs.current = Math.min(index, files.length - 1);
  }

  const stillOpen = files.some(
    file => file.uri.toString() === removedFile.uri.toString()
  );
  if (!stillOpen) {
    monaco.editor.getModel(removedFile.uri)?.dispose();
  }
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
