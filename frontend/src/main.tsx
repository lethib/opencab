import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/dm-sans";
import "@fontsource-variable/geist-mono";
import "./index.css";
import { QueryClientProvider } from "@tanstack/react-query";
import App from "./App.tsx";
import { queryClient } from "./api/api.ts";
import "./i18n";
import { Toaster } from "@/components/ui/sonner.tsx";

const mq = window.matchMedia("(prefers-color-scheme: dark)");
const applyTheme = (e: MediaQueryList | MediaQueryListEvent) =>
  document.documentElement.classList.toggle("dark", e.matches);
mq.addEventListener("change", applyTheme);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
      <Toaster position="top-right" />
    </QueryClientProvider>
  </StrictMode>,
);
