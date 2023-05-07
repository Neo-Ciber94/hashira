const HOST = window.HASHIRA_LIVE_RELOAD_HOST || "127.0.0.1";
const PORT = window.HASHIRA_LIVE_RELOAD_PORT || 5002;
const ADDR = `${HOST}:${PORT}`;

const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
const url = protocol + "//" + ADDR + "/ws";
const pollInterval = 2000;

function showLoadingIndicator() {
  const loadingElement = document.createElement("div");
  loadingElement.innerText = "Loading...";
  loadingElement.style.position = "fixed";
  loadingElement.style.bottom = "10px";
  loadingElement.style.left = "10px";
  loadingElement.style.backgroundColor = "white";
  loadingElement.style.borderRadius = "10px";
  loadingElement.style.padding = "10px";
  loadingElement.style.border = "1px solid black";
  loadingElement.style.zIndex = "9999";
  loadingElement.style.fontFamily = "monospace";
  loadingElement.style.fontSize = "18px";
  loadingElement.style.cursor = "pointer";
  loadingElement.id = "loading-element";

  // Add loading element to the document
  document.body.appendChild(loadingElement);
  loadingElement.animate([{ opacity: 1 }, { opacity: 0.4 }, { opacity: 1 }], {
    duration: 2000,
    iterations: Infinity,
  });
}

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
  let isLoading = false;

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.reload === true) {
      console.log("🔃 Reloading window");
      window.location.reload();
    }

    if (data.loading === true && !isLoading) {
      console.log("🚧 Loading...");
      isLoading = true;
      showLoadingIndicator();
    }
  };

  // FIXME: reconnect on error?
  ws.onerror = () => console.log("📛 connection error");
  ws.onclose = handleReconnect;
}

startWebsocket();
