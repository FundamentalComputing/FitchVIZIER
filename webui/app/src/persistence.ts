import Alpine from "alpinejs";
import * as monaco from "monaco-editor";
import type { TabsStore } from "./stores";

interface PersistedTab {
  name: string;
  uri: string;
  proofTarget: string;
  content: string;
}

interface PersistedTabs {
  files: PersistedTab[];
}

let hasLoadedLocalStorage = false;
export function saveToLocalStorage() {
  const storeData = Alpine.store("tabs");

  if (!hasLoadedLocalStorage) {
    return;
  }
  const data = storeData.files.map((file) => {
    const content = monaco.editor.getModel(file.uri)?.getValue() ?? "";

    return { ...file, content, uri: file.uri.toString() };
  });

  localStorage.setItem("tabs", JSON.stringify({ files: data }));
}

export function loadFromLocalStorage() {
  const storedTabs = localStorage.getItem("tabs");
  const importedData = storedTabs ? JSON.parse(storedTabs) as PersistedTabs : null;
  if (!importedData || !importedData.files) {
    hasLoadedLocalStorage = true;
    return;
  }
  monaco.editor.getModels().forEach(m => m.dispose());
  const newTabsData: TabsStore = { current: 0, files: [] };
  let highestNewFile = 1;
  const loadedUris = new Set<string>();
  for (const tab of importedData.files) {
    const tabUri = tab.uri;
    if (loadedUris.has(tabUri)) {
      continue;
    }
    loadedUris.add(tabUri);
    const uri = monaco.Uri.parse(tab.uri);
    monaco.editor.createModel(tab.content, "fitch", uri);
    newTabsData.files.push({
      name: tab.name,
      proofTarget: tab.proofTarget,
      uri
    });

    if (tab.name.startsWith("new-")) {
      const n = parseInt(tab.name.slice(4));
      if (n > highestNewFile) highestNewFile = n;
    }
  }

  Alpine.store("tabs").files = newTabsData.files;
  Alpine.store("tabs").current = 0;
  Alpine.store("newFileCounter").value = highestNewFile + 1;

  hasLoadedLocalStorage = true;
}
