import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: [],
    include: ['src/**/*.test.{ts,tsx}'],
  },
  resolve: {
    alias: {
      buffer: 'buffer',
    },
  },
  define: {
    global: 'globalThis',
    'import.meta.env.VITE_CONTRACT_ID': JSON.stringify('CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4'),
    'import.meta.env.VITE_NETWORK_PASSPHRASE': JSON.stringify('Test SDF Network ; September 2015'),
    'import.meta.env.VITE_RPC_URL': JSON.stringify('https://soroban-testnet.stellar.org'),
    'import.meta.env.VITE_HORIZON_URL': JSON.stringify('https://horizon-testnet.stellar.org'),
    'import.meta.env.VITE_EXPLORER_URL': JSON.stringify('https://stellar.expert/explorer/testnet'),
    'import.meta.env.VITE_SIMULATION_ACCOUNT': JSON.stringify('GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H'),
  },
})
