// Static server for the epher a11y suites: serves the built PWA
// (crates/web/dist) at /pwa/ with correct MIME (wasm needs
// application/wasm). Used by audit.mjs and voiceover-audit.mjs.
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';

const MIME = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript',
  '.css': 'text/css', '.wasm': 'application/wasm', '.svg': 'image/svg+xml',
  '.json': 'application/json', '.webmanifest': 'application/manifest+json',
  '.md': 'text/markdown', '.png': 'image/png', '.ico': 'image/x-icon',
};

export async function startPwaServer({ port = Number(process.env.A11Y_PORT ?? 8741) } = {}) {
  const DIST = join(import.meta.dirname, '..', '..', 'crates', 'web', 'dist');
  const server = createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    const rel = path.startsWith('/pwa/') ? path.slice(5) : 'index.html';
    let file = join(DIST, rel);
    if (!file.startsWith(DIST) || !existsSync(file) || statSync(file).isDirectory()) {
      file = join(DIST, rel, 'index.html');
      if (/\.[a-z0-9]+$/i.test(rel) && !existsSync(join(DIST, rel))) {
        res.writeHead(404).end('not found');
        return;
      }
    }
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
  });
  await new Promise((r) => server.listen(port, '127.0.0.1', r));
  return { server, base: `http://127.0.0.1:${port}/pwa/` };
}
