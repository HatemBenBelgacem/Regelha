// Push-Benachrichtigungen für Regelha.
//
// Wird im Seitenkopf geladen. Die Seitenleiste ruft diese Funktionen auf und wertet ihre Antwort aus.
// Die Browser-Schnittstellen (Benachrichtigungen, Push-Abo) gibt es nur im Browser, nicht auf dem Server.

(function () {
  // Der öffentliche VAPID-Schlüssel kommt als Text (base64url). Der Browser braucht ihn als Bytes.
  function base64ZuBytes(text) {
    const pad = '='.repeat((4 - (text.length % 4)) % 4);
    const roh = atob((text + pad).replace(/-/g, '+').replace(/_/g, '/'));
    const bytes = new Uint8Array(roh.length);
    for (let i = 0; i < roh.length; i++) {
      bytes[i] = roh.charCodeAt(i);
    }
    return bytes;
  }

  // Zeigt die Antwort des Servers, wenn etwas fehlschlägt, damit der Grund sichtbar ist.
  async function serverFehler(antwort) {
    let text = '';
    try {
      text = (await antwort.text()).trim();
    } catch (e) {
      text = '';
    }
    if (text.startsWith('<')) {
      return 'fehler:Unerwartete Antwort des Servers (Status ' + antwort.status + ').';
    }
    // Leptos stellt Server-Fehlern ein Kennzeichen voran (z.B. "ServerError|"). Das ist für Menschen nicht nötig.
    text = text.replace(/^ServerError\|/, '');
    if (text.length > 300) {
      text = text.slice(0, 300) + '…';
    }
    return 'fehler:' + (text || 'Der Server hat keinen Fehlertext geschickt') + ' (Status ' + antwort.status + ')';
  }

  function unterstuetzt() {
    return 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
  }

  // Status: 'aktiv' | 'inaktiv' | 'verweigert' | 'nicht_unterstuetzt'
  window.regelhaPushStatus = async function () {
    if (!unterstuetzt()) return 'nicht_unterstuetzt';
    if (Notification.permission === 'denied') return 'verweigert';
    try {
      const reg = await navigator.serviceWorker.ready;
      const abo = await reg.pushManager.getSubscription();
      return abo && Notification.permission === 'granted' ? 'aktiv' : 'inaktiv';
    } catch (e) {
      return 'inaktiv';
    }
  };

  // Gerät anmelden. Ergebnis: 'ok' | 'verweigert' | 'nicht_unterstuetzt' | 'fehler:<Text>'
  window.regelhaPushAbonnieren = async function () {
    if (!unterstuetzt()) return 'nicht_unterstuetzt';
    try {
      const erlaubnis = await Notification.requestPermission();
      if (erlaubnis !== 'granted') return 'verweigert';

      const schluesselAntwort = await fetch('/api/get_push_public_key', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: '{}',
      });
      if (!schluesselAntwort.ok) {
        return await serverFehler(schluesselAntwort);
      }
      const schluessel = await schluesselAntwort.json();

      const reg = await navigator.serviceWorker.ready;
      let abo = await reg.pushManager.getSubscription();
      if (!abo) {
        abo = await reg.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: base64ZuBytes(schluessel),
        });
      }

      const daten = abo.toJSON();
      const antwort = await fetch('/api/speichere_push_abo', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          endpoint: daten.endpoint,
          p256dh: daten.keys.p256dh,
          auth: daten.keys.auth,
        }),
      });
      if (!antwort.ok) {
        return await serverFehler(antwort);
      }
      return 'ok';
    } catch (e) {
      return 'fehler:' + (e && e.message ? e.message : 'Unbekannter Fehler');
    }
  };
})();
