import { defineConfig } from 'orval'

export default defineConfig({
  realmforge: {
    input: {
      // Generated from: cargo run -p control-api --bin generate_schema > openapi.json
      target: '../openapi.json',
    },
    output: {
      mode: 'tags-split',
      target: 'src/api/generated',
      client: 'react-query',
      httpClient: 'fetch',
      baseUrl: '',  // ADR-0005: same-origin, VITE_API_BASE_URL=""
      override: {
        mutator: {
          path: 'src/api/client.ts',
          name: 'customFetch',
        },
      },
    },
  },
})
