const HOST = window.HASHIRA_LIVE_RELOAD_HOST || "127.0.0.1";
const PORT = window.HASHIRA_LIVE_RELOAD_PORT || 5002;
const ADDR = `${HOST}:${PORT}`;
const pollInterval = window.HASHIRA_LIVE_RELOAD_POLL_INTERVAL || 5000;

const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
const url = protocol + "//" + ADDR + "/ws";

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

  ws.onclose = handleReconnect;
  // FIXME: reconnect on error?
  return ws;
}

startWebsocket();
