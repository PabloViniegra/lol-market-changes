import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig, type Plugin } from "vite";

// The fonts are only discovered after the CSS downloads; on slow mobile connections they swap in
// late and shift the board.
function preloadFonts(names: string[]): Plugin {
  return {
    name: "preload-fonts",
    apply: "build",
    transformIndexHtml: {
      order: "post",
      handler: (_html, { bundle }) =>
        Object.keys(bundle ?? {})
          .filter((file) => file.endsWith(".woff2") && names.some((name) => file.includes(name)))
          .map((file) => ({
            tag: "link",
            attrs: {
              rel: "preload",
              href: `/${file}`,
              as: "font",
              type: "font/woff2",
              crossorigin: "",
            },
            injectTo: "head" as const,
          })),
    },
  };
}

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    preloadFonts([
      "big-shoulders-display-latin-wght-normal",
      "atkinson-hyperlegible-next-latin-wght-normal",
    ]),
  ],
  server: {
    proxy: { "/api": process.env.API_ORIGIN ?? "https://lol-market-changes.vercel.app" },
  },
});
