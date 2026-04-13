import React from "react";
import ReactDOM from "react-dom/client";
import "./themes/window-base.css";
import "./components/GroupPopup.css";
import "./components/LauncherGrid.css";
import "./components/ContextMenu.css";
import "./components/ItemEditDialog.css";
import { GroupPopupWindow } from "./components/GroupPopupWindow";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <GroupPopupWindow />
  </React.StrictMode>,
);
