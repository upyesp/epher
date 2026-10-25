// Static server for the epher a11y suites: serves a directory tree with
// correct MIME (wasm needs application/wasm) and an index.html fallback
// for directories and extensionless deep links; true misses 404 so the
// audits notice broken references.
//
// Used by audit.mjs (the PWA suite) and site-audit.mjs (the website).
// Root/mount can be overridden for CI, where the assembled _site tree
// already contains the built PWA under pwa/:
//   A11Y_DIST=_site/pwa node scripts/a11y/audit.mjs   (PWA from _site)
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';

const MIME = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript',
  '.mjs': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml', '.json': 'application/json',
  '.webmanifest': 'application/manifest+json', '.md': 'text/markdown',
  '.txt': 'text/plain', '.xml': 'application/xml',
  '.png': 'image/png', '.ico': 'image/x-icon', '.woff2': 'font/woff2',
};

export async function startPwaServer({
  port = Number(process.env.A11Y_PORT ?? 8741),
  root = process.env.A11Y_DIST ?? join(import.meta.dirname, '..', '..', 'crates', 'web', 'dist'),
  mount = '/pwa/',
} = {}) {
  const ROOT = root.endsWith('/') ? root.slice(0, -1) : root;
  const MOUNT = mount.endsWith('/') ? mount : `${mount}/`;
  const server = createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    const rel = path.startsWith(MOUNT) && MOUNT !== '/'
      ? path.slice(MOUNT.length)
      : MOUNT === '/' ? path.replace(/^\//, '') : null;
    if (rel === null) {
      // Outside the mount: the PWA suite only ever asks for /pwa/*, the
      // site suite serves everything from /, so this is a broken ref.
      res.writeHead(404).end('not found');
      return;
    }
    let file = join(ROOT, rel);
    if (!file.startsWith(ROOT) || (existsSync(file) && statSync(file).isDirectory())) {
      file = join(ROOT, rel, 'index.html');
    }
    if (!existsSync(file)) {
      // Extensionless paths fall back to <path>.html (site pages are
      // linked both ways); everything else is a true miss.
      const html = join(ROOT, `${rel}.html`);
      if (existsSync(html)) { file = html; } else { res.writeHead(404).end('not found'); return; }
    }
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
  });
  await new Promise((r) => server.listen(port, '127.0.0.1', r));
  return { server, base: `http://127.0.0.1:${port}${MOUNT === '/' ? '/' : MOUNT}` };
}
