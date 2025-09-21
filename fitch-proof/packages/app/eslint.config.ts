// eslint.config.js
import { defineConfig } from "eslint/config";
import stylistic from "@stylistic/eslint-plugin";
import tsParser from "@typescript-eslint/parser";


export default defineConfig([
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parser: tsParser,        // ← important
      parserOptions: {
        project: "./tsconfig.json", // optional but good for type-aware rules
        sourceType: "module",
      },
    },
    plugins: {
      "@stylistic": stylistic,          // ← register the plugin
    },
    rules: {
      semi: "error",
      "prefer-const": "error",
      "@stylistic/max-len": ["warn", { code: 120 }],
    },
  },
]);
