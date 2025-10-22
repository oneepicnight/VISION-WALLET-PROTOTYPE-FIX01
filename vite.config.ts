import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  define: {
    global: 'globalThis',
  },
  optimizeDeps: {
    include: ['buffer']
  }
  ,
  server: {
    host: '127.0.0.1',
    port: 4173,
    proxy: {
      '/api/market': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/api\/market/, '')
      }
    }
  }
})