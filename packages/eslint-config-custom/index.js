module.exports = {
  extends: [
    "eslint:recommended",
    // "plugin:import/recommended",
    // "plugin:import/typescript",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "plugin:@typescript-eslint/strict",
    "plugin:prettier/recommended",
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
  },
  ignorePatterns: ["node_modules/**/*", "build/**/*", "dist/**/*"],
  rules: {
    // "import/order": [
    //   "error",
    //   {
    //     alphabetize: {
    //       caseInsensitive: true,
    //       order: "asc",
    //     },
    //     groups: [["builtin", "external"], "parent", "sibling", "index"],
    //   },
    // ],
    "no-alert": "error",
    "sort-imports": [
      "error",
      {
        ignoreCase: true,
        ignoreDeclarationSort: true,
        allowSeparatedGroups: true,
      },
    ],
  },
};
