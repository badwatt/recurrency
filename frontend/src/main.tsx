import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

// Toaster must be rendered inside Router but outside StrictMode to avoid Sonner warning
import { Toaster } from "sonner";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.Fragment>
    <Toaster 
      position="top-right" 
      richColors 
      closeButton
      toastOptions={{
        className: "sonner-toast",
      }}
    />
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.Fragment>
);
