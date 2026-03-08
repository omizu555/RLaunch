import React from "react";
import ReactDOM from "react-dom/client";
import "./themes/window-base.css";
import "./components/SettingsWindow.css";
import { SettingsWindow } from "./components/SettingsWindow";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsWindow />
  </React.StrictMode>,
);
