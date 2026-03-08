import React from "react";
import ReactDOM from "react-dom/client";
import "./themes/window-base.css";
import "./components/WidgetSelectWindow.css";
import { WidgetSelectWindow } from "./components/WidgetSelectWindow";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <WidgetSelectWindow />
  </React.StrictMode>,
);
