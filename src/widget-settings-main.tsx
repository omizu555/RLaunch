import React from "react";
import ReactDOM from "react-dom/client";
import "./themes/window-base.css";
import "./components/WidgetSettingsWindow.css";
import { WidgetSettingsWindow } from "./components/WidgetSettingsWindow";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <WidgetSettingsWindow />
  </React.StrictMode>,
);
