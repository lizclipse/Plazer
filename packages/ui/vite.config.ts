import solid from "solid-start/vite";
import { babel } from "@rollup/plugin-babel";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [
    babel({
      extensions: [".ts", ".tsx"],
      babelrc: false,
      configFile: false,
      plugins: ["babel-plugin-graphql-tag"],
    }),
    solid(),
  ],
  server: {
    proxy: {
      "/api": "http://localhost:8080",
    },
  },
});
