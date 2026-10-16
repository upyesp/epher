// Copies the shared editor assets from clients/shared into the places
// the extension packages them. Run before compiling or packaging
// (`npm run compile` does it); the copies under syntaxes/ and
// snippets/ are checked in so packaging never needs a network.
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const shared = join(root, "..", "shared");

mkdirSync(join(root, "syntaxes"), { recursive: true });
mkdirSync(join(root, "snippets"), { recursive: true });
copyFileSync(
  join(shared, "epher.tmLanguage.json"),
  join(root, "syntaxes", "epher.tmLanguage.json"),
);
copyFileSync(
  join(shared, "epher-snippets.json"),
  join(root, "snippets", "epher.json"),
);
console.log("synced clients/shared into clients/vscode");
