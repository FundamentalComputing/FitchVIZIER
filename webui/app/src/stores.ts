import Alpine from "alpinejs";
import * as monaco from "monaco-editor";

export interface TabsStore {
  files: { name: string, uri: monaco.Uri, proofTarget: string, confettiPlayed: boolean }[];
  current: number;
}


// Extend Alpine's Stores interface
declare module 'alpinejs' {
  interface Stores {
    tabs: TabsStore;
    newFileCounter: { value: number, inc: () => void }
  }
}

export function initStores() {
  Alpine.store('tabs', {
    current: 0,
    files: [],
  });
  Alpine.store('newFileCounter', {
    value: 1,

    inc() {
      this.value++;
    }
  });
}

