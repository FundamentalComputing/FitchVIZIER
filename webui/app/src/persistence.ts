import Alpine from "alpinejs";
import * as monaco from "monaco-editor";
import { createFreshUri } from "./helpers";
import type { TabsStore } from "./stores";

interface PersistedTab {
  name: string;
  proofTarget: string;
  content: string;
  // Kept optional so older saved data can still be read. It is never restored.
  uri?: string;
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

    return { name: file.name, proofTarget: file.proofTarget, content };
  });

  localStorage.setItem("tabs", JSON.stringify({ files: data }));
}

function isPersistedTab(value: unknown): value is PersistedTab {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const tab = value as Record<string, unknown>;
  return typeof tab.name === "string"
    && typeof tab.proofTarget === "string"
    && typeof tab.content === "string";
}

export function loadFromLocalStorage() {
  let storedTabs: string | null;
  try {
    storedTabs = localStorage.getItem("tabs");
  } catch {
    hasLoadedLocalStorage = true;
    return;
  }

  if (storedTabs === null) {
    hasLoadedLocalStorage = true;
    return;
  }

  let importedData: unknown;
  try {
    importedData = JSON.parse(storedTabs);
  } catch {
    hasLoadedLocalStorage = true;
    return;
  }

  if (typeof importedData !== "object" || importedData === null
    || !Array.isArray((importedData as { files?: unknown }).files)) {
    hasLoadedLocalStorage = true;
    return;
  }

  const validTabs = (importedData as PersistedTabs).files.filter(isPersistedTab);
  if (validTabs.length === 0) {
    hasLoadedLocalStorage = true;
    return;
  }

  monaco.editor.getModels().forEach(m => m.dispose());
  const newTabsData: TabsStore = { current: 0, files: [] };
  let highestNewFile = 1;
  for (const tab of validTabs) {
    const uri = createFreshUri();
    monaco.editor.createModel(tab.content, "fitch", uri);
    newTabsData.files.push({
      name: tab.name,
      proofTarget: tab.proofTarget,
      uri
    });

    const generatedName = /^new-(\d+)\.txt$/.exec(tab.name);
    if (generatedName) {
      const n = Number(generatedName[1]);
      if (Number.isSafeInteger(n) && n > highestNewFile) {
        highestNewFile = n;
      }
    }
  }

  Alpine.store("tabs").files = newTabsData.files;
  Alpine.store("tabs").current = 0;
  Alpine.store("newFileCounter").value = highestNewFile + 1;

  hasLoadedLocalStorage = true;
}
