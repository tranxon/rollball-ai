import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./i18n"; // i18n initialization (must run before any useTranslation call)
import "./styles/globals.css";

// Import settingsStore early so theme is applied to DOM before first paint.
// The store initializer calls applyTheme() which toggles the .dark class
// based on the persisted preference from localStorage.
import "./stores/settingsStore";

// Disable native browser context menu to prevent accidental page refresh
// and other browser actions that would restart the entire app.
// Custom context menus (ChatPanel, AgentList) handle their own preventDefault.
document.addEventListener("contextmenu", (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
