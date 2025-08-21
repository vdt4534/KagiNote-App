import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import ErrorBoundary from "./components/ErrorBoundary";
import "./styles/globals.css";

console.log("Main.tsx loaded");
const rootElement = document.getElementById("root");
console.log("Root element:", rootElement);

if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
  console.log("React app rendered");
} else {
  console.error("Root element not found!");
}
