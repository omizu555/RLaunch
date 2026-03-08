import React from "react";
import ReactDOM from "react-dom/client";
import "./themes/window-base.css";
import "./components/FolderBrowser.css";
import { FolderBrowserWindow } from "./components/FolderBrowserWindow";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <FolderBrowserWindow />
  </React.StrictMode>,
);
