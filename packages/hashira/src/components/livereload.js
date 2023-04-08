const HOST = window.HASHIRA_LIVE_RELOAD_HOST || "127.0.0.1";
const PORT = window.HASHIRA_LIVE_RELOAD_PORT || 5002;
const ADDR = `${HOST}:${PORT}`;

const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
const url = protocol + "//" + ADDR + "/ws";
const pollInterval = 2000;

function handleReconnect() {
  setTimeout(() => {
    console.log("🕗 Reconnecting...");
    startWebsocket();

    // If we are reconnecting we should reload
    window.location.reload();
  }, pollInterval);
}

function startWebsocket() {
  console.log("⚡ Starting websocket...");

  const ws = new WebSocket(url);
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.reload === true) {
      console.log("🔃 Reloading window");
      window.location.reload();
    }
  };

  // FIXME: reconnect on error?
  ws.onerror = () => console.log("📛 connection error");
  ws.onclose = handleReconnect;
}

startWebsocket();
