import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

/** quill-next ships quill.snow.css with a sourceMappingURL but no map file. */
function stripBrokenCssSourceMaps(): Plugin {
  return {
    name: 'strip-broken-css-sourcemaps',
    enforce: 'pre',
    transform(code, id) {
      if (!id.includes('node_modules') || !id.endsWith('.css')) return;
      if (!code.includes('sourceMappingURL')) return;
      return {
        code: code.replace(/\n?\/\*#\s*sourceMappingURL=[^*]+\*\/\s*$/m, ''),
        map: null,
      };
    },
  };
}

// Tauri serves the frontend from a fixed port and watches Rust separately.
export default defineConfig({
  plugins: [stripBrokenCssSourceMaps(), svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/launcher/**'] },
  },
  build: { target: 'chrome110', sourcemap: true },
});
