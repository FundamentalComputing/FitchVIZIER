import Alpine from "alpinejs";
import * as monaco from "monaco-editor";
import type { TabsStore } from "./stores";

let hasLoadedLocalStorage = false;
export function saveToLocalStorage() {
  const storeData = Alpine.store("tabs");

  // save to localstorage
  if (!hasLoadedLocalStorage) {
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

export function loadFromLocalStorage() {
  const importedData = JSON.parse(localStorage.getItem("tabs")) as Omit<TabsStore, 'current'>;
  if (!importedData || !importedData.files) {
    hasLoadedLocalStorage = true;
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
  Alpine.store("newFileCounter").value = highestNewFile + 1;
  console.log("loaded tabs");

  hasLoadedLocalStorage = true;
}
