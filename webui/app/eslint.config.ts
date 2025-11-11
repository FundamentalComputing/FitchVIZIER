// eslint.config.js
import { defineConfig } from "eslint/config";
import stylistic from "@stylistic/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import tsEslint from "@typescript-eslint/eslint-plugin";

export default defineConfig([
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parser: tsParser, // ← important
      parserOptions: {
        project: "./tsconfig.json", // optional but good for type-aware rules
        sourceType: "module",
      },
    },
    plugins: {
      "@stylistic": stylistic, // ← register the plugin
      "@typescript-eslint": tsEslint,
    },
    rules: {
      semi: "error",
      "prefer-const": "error",
      "@stylistic/max-len": ["warn", { code: 120 }],
      "@typescript-eslint/no-unused-vars": "warn",
    },
  },
]);
