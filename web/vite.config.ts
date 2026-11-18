import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  server: {
    proxy: { "/api": process.env.API_ORIGIN ?? "https://lol-market-changes.vercel.app" },
  },
});
