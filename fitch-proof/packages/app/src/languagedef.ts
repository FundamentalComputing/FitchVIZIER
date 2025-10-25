import * as monaco from "monaco-editor";

export const languagedef: monaco.languages.IMonarchLanguage = {
  tokenizer: {
    root: [
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
      [/⊤/, "operator.verum"],
      [/=/, "operator.equality"],
      [/≠/, "operator.inequality"],

      // Parentheses and brackets
      [/[()[\]{}]/, "delimiter"],

      // Predicate/function names (capital letters)
      [/[A-Z][a-zA-Z0-9]*/, "predicate"],

      // Variables and constants (lowercase)
      [/[a-z][a-zA-Z0-9]*/, "variable"],

      // Function applications like f(a), g(x,y)
      [/[a-z]+(?=\()/, "function"],

      // Justifications - rule names
      [
        /\b(Reit|∧\s*Elim|∨\s*Elim|∧\s*Intro|∨\s*Intro|→\s*Elim|→\s*Intro|¬\s*Elim|¬\s*Intro|=\s*Elim|=\s*Intro|∀\s*Elim|∀\s*Intro|∃\s*Elim|∃\s*Intro|⊥\s*Elim|RAA|MT|DS|HS|Add|Simp|Conj|MP|DeM|DN|Com|Assoc|Dist|Exp|Equiv|Impl|Taut|Contra)\b/,
        "rule-name",
      ],

      // Line references in justifications (numbers, ranges)
      [/:\s*\d+/, "justification.reference"],
      [/\d+-\d+/, "justification.reference"],
      [/,\s*\d+/, "justification.reference"],

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
    { token: "operator.verum", foreground: "44FF44", fontStyle: "bold" },
    { token: "operator.equality", foreground: "CC6666", fontStyle: "bold" },
    { token: "operator.inequality", foreground: "CC6666", fontStyle: "bold" },
    { token: "quantifier.universal", foreground: "9966CC", fontStyle: "bold" },
    {
      token: "quantifier.existential",
      foreground: "9966CC",
      fontStyle: "bold",
    },
    { token: "predicate", foreground: "99CC99" },
    { token: "function", foreground: "CCCC66" },
    { token: "variable", foreground: "99CCFF" },
    { token: "rule-name", foreground: "FF9966", fontStyle: "italic" },
    { token: "justification.reference", foreground: "CCCCCC" },
    { token: "delimiter", foreground: "#87875f" },
  ],
  colors: {
    "editor.background": "#1e1e1e",
  },
};
