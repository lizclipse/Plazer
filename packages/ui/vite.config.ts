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
      babelHelpers: "bundled",
    }),
    solid(),
  ],
  server: {
    host: true,
    proxy: {
      "/api/graphql/ws": {
        target: "ws://localhost:8080",
        ws: true,
        changeOrigin: true,
      },
      "/api/graphql": {
        target: "htpp://localhost:8080",
        changeOrigin: true,
      },
    },
  },
});
