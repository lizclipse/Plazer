/** @type {import("@typescript-eslint/utils").TSESLint.Linter.Config} */
module.exports = {
  root: true,
  extends: "custom",
  parserOptions: {
    project: "tsconfig.json",
    tsconfigRootDir: __dirname,
  },
  ignorePatterns: [
    "scripts/**",
    ".eslintrc.cjs",
    "vite.config.ts",
    "vitest.config.ts",
    "__generated__/**",
    "*.gql.ts",
  ],
};
