import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import UnoCSS from 'unocss/vite'

export default defineConfig(async () => ({
  plugins: [vue(), vueJsx(), UnoCSS()],
  clearScreen: false,
  server: {
    port: 5005,
    strictPort: true,
    host: false,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
}))
