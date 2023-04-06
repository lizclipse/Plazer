module.exports = {
  root: true,
  extends: "custom",
  parserOptions: {
    project: "tsconfig.json",
    tsconfigRootDir: __dirname,
  },
  ignorePatterns: [
    ".eslintrc.cjs",
    "vite.config.ts",
    "vitest.config.ts",
    "*.gql.ts",
  ],
};
