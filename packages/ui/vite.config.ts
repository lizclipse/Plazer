import solid from "solid-start/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [solid()],
  server: {
    proxy: {
      '/api': 'http://localhost:8080'
    }
  }
});
