// Service Worker für Regelha (PWA).
//
// Absichtlich schlank: Er speichert keine App-Dateien (WASM, JavaScript, CSS) und keine Daten.
// So zeigt das Handy nach einem Update immer den neuen Stand. Zwischengespeichert werden nur
// die Icons, das Manifest und das Offline-Fenster.

const VERSION = 'regelha-pwa-v1';
const STATISCH = [
  '/offline.html',
  '/manifest.json',
  '/icons/icon-192.png',
  '/icons/icon-512.png',
  '/icons/icon-512-maskable.png',
  '/icons/apple-touch-icon.png',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(VERSION)
      .then((cache) => cache.addAll(STATISCH))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((namen) => Promise.all(namen.filter((n) => n !== VERSION).map((n) => caches.delete(n))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const anfrage = event.request;

  // Nur GET abfangen. Server-Funktionen (POST) und der Live-Chat laufen immer übers Netz.
  if (anfrage.method !== 'GET') return;

  const url = new URL(anfrage.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname.startsWith('/api/')) return;

  // Seitenaufrufe: wenn das Netz fehlt, das Offline-Fenster zeigen
  if (anfrage.mode === 'navigate') {
    event.respondWith(fetch(anfrage).catch(() => caches.match('/offline.html')));
    return;
  }

  // Icons und Manifest: zuerst aus dem Netz, sonst aus dem Speicher
  if (STATISCH.includes(url.pathname)) {
    event.respondWith(fetch(anfrage).catch(() => caches.match(anfrage)));
    return;
  }

  // Alles andere (App-Code, WASM, CSS) geht direkt ans Netz und wird nicht gespeichert.
});
