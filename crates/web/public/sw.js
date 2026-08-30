// Offline-first service worker for the epher web app.
//
// Navigation requests are network-first (so new deploys reach users) with a
// cache fallback when offline. Same-origin asset requests are runtime-cached
// (cache-first after the first load), so the app is fully usable offline
// once it has been loaded online — the shell (index.html + the hashed wasm
// and js) is what the fetch handler sees on the first visit.
//
// Bump CACHE when the precache set or the caching strategy changes; the
// activate handler clears older caches.
const CACHE = "epher-v4";

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(["./", "./index.html", "./manifest.webmanifest"]))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;

  // Navigations: try the network first so updated builds propagate, then
  // fall back to the cached shell for offline use.
  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(request, copy));
          return response;
        })
        .catch(() =>
          caches.match(request).then((hit) => hit || caches.match("./index.html"))
        )
    );
    return;
  }

  // Assets (wasm, js, icon): cache-first with runtime caching.
  event.respondWith(
    caches.match(request).then((hit) => {
      return (
        hit ||
        fetch(request).then((response) => {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(request, copy));
          return response;
        })
      );
    })
  );
});
