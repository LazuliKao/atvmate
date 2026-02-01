import { defineConfig } from 'vite'
import preactCompat from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss(), preactCompat()],
})
