import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  base: '',
  plugins: [tailwindcss()],
  build: {
    rollupOptions: {
      input: { main: "index.html", latex_export: "latex_export.html" }
    }
  }
})

