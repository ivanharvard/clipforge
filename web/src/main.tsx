import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./styles/theme.css";
import "./styles/app.css";
import "./styles/controls.css";
import "./styles/export.css";
import "./styles/responsive.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("ClipForge root element is missing");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
