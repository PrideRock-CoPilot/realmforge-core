import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// ADR-0005: Same-origin hosting. API_BASE_URL is empty string.
// Dev proxy forwards /v1/* to local control-api.
const CONTROL_API_PORT = process.env.CONTROL_API_PORT ?? '8080'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    proxy: {
      '/v1': {
        target: `http://localhost:${CONTROL_API_PORT}`,
        changeOrigin: false,
      },
      '/health': {
        target: `http://localhost:${CONTROL_API_PORT}`,
        changeOrigin: false,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
})
