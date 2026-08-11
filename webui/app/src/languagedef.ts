import * as monaco from "monaco-editor";

export const languagedef: monaco.languages.IMonarchLanguage = {
  tokenizer: {
    root: [

      // Justifications - rule names
      [
        /((?:\S\s)?\b(?:Intro|Elim|Reit)\b)/,
        "justification.rule-name",
      ],

      // Line references in justifications (numbers, ranges)
      [/:\s*\d+.*/, "justification.reference"],

      // Line numbers at start
      [/^\s*\d+/, "line-number"],

      // Proof structure - vertical bars and dashes
      [/\s*\|\s*/, "proof-structure"],
      [/\s*-+\s*/, "proof-structure"],

      // Logical operators and symbols
      [/∧/, "operator.conjunction"],
      [/∨/, "operator.disjunction"],
      [/¬/, "operator.negation"],
      [/→/, "operator.implication"],
      [/↔/, "operator.biconditional"],
      [/∀/, "quantifier.universal"],
      [/∃/, "quantifier.existential"],
      [/⊥/, "operator.falsum"],
      [/=/, "operator.equality"],

      // Parentheses and brackets
      [/[()[\]{}]/, "delimiter"],

      // Predicate/function names (capital letters)
      [/[A-Z][a-zA-Z0-9]*/, "predicate"],

      // Variables and constants (lowercase)
      [/[a-z][a-zA-Z0-9]*/, "variable"],

      // Function applications like f(a), g(x,y)
      [/[a-z]+(?=\()/, "function"],

      // Comma separator
      [/,/, "delimiter"],

      // Whitespace
      [/\s+/, "white"],
    ],
  },
};


export const theme: monaco.editor.IStandaloneThemeData = {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "line-number", foreground: "888888", fontStyle: "bold" },
    { token: "proof-structure", foreground: "888888" },
    { token: "operator.conjunction", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.disjunction", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.negation", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.implication", foreground: "CC6666", fontStyle: "bold" },
    {
      token: "operator.biconditional",
      foreground: "CC6666",
      fontStyle: "bold",
    },
    { token: "operator.falsum", foreground: "FF4444", fontStyle: "bold" },
    { token: "operator.equality", foreground: "CC6666", fontStyle: "bold" },
    { token: "quantifier.universal", foreground: "9966CC", fontStyle: "bold" },
    {
      token: "quantifier.existential",
      foreground: "9966CC",
      fontStyle: "bold",
    },
    { token: "justification.rule-name", foreground: "FF9966", fontStyle: "italic" },
    { token: "justification.reference", foreground: "FF9966", fontStyle: "italic" },
    { token: "predicate", foreground: "99CC99" },
    { token: "function", foreground: "CCCC66" },
    { token: "variable", foreground: "99CCFF" },
    { token: "delimiter", foreground: "#87875f" },
  ],
  colors: {
    "editor.background": "#1e1e1e",
  },
};


export const lightTheme: monaco.editor.IStandaloneThemeData = {
  base: "vs",
  inherit: true,
  rules: [
    { token: "line-number", foreground: "888888", fontStyle: "bold" },
    { token: "proof-structure", foreground: "888888" },
    { token: "operator.conjunction", foreground: "CC3333", fontStyle: "bold" },
    { token: "operator.disjunction", foreground: "CC3333", fontStyle: "bold" },
    { token: "operator.negation", foreground: "CC3333", fontStyle: "bold" },
    { token: "operator.implication", foreground: "CC3333", fontStyle: "bold" },
    {
      token: "operator.biconditional",
      foreground: "CC3333",
      fontStyle: "bold",
    },
    { token: "operator.falsum", foreground: "DD0000", fontStyle: "bold" },
    { token: "operator.verum", foreground: "00AA00", fontStyle: "bold" },
    { token: "operator.equality", foreground: "CC3333", fontStyle: "bold" },
    { token: "operator.inequality", foreground: "CC3333", fontStyle: "bold" },
    { token: "quantifier.universal", foreground: "7744AA", fontStyle: "bold" },
    {
      token: "quantifier.existential",
      foreground: "7744AA",
      fontStyle: "bold",
    },
    { token: "predicate", foreground: "338833" },
    { token: "function", foreground: "997700" },
    { token: "variable", foreground: "0066CC" },
    { token: "justification.rule-name", foreground: "666666", fontStyle: "italic" },
    { token: "justification.reference", foreground: "666666" },
    { token: "delimiter", foreground: "#6b6b4a" },
  ],
  colors: {
    "editor.background": "#f5f5f5",
  }
};
