import { defineConfig } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const configDir = path.dirname(fileURLToPath(import.meta.url));
const tauriDir = path.join(configDir, 'src-tauri');
const dataDir = path.join(tauriDir, '.e2e-data');

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 120_000,
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'cargo run --features web --bin fincept-server',
    cwd: tauriDir,
    port: 3000,
    reuseExistingServer: !process.env.CI,
    env: {
      FINCEPT_HOST: '127.0.0.1',
      FINCEPT_PORT: '3000',
      FINCEPT_DATA_DIR: dataDir,
      FINCEPT_SCRIPTS_PATH: path.join(tauriDir, 'resources', 'scripts'),
    },
  },
});
