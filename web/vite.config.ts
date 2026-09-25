import { defineConfig } from 'vite';

// games/ is served as static files, so the replay always shows what the arena saved.
// GAMES_DIR overrides it (e.g. to view smoke-test games in a tmp dir).
export default defineConfig({
  publicDir: process.env.GAMES_DIR ?? '../games',
});
