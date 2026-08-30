import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

export type Theme = "light" | "dark" | "auto";

const STORAGE_KEY = "theme";

function resolveAuto(): "light" | "dark" {
  return matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyClass(resolved: "light" | "dark") {
  document.documentElement.classList.toggle("dark", resolved === "dark");
}

function readStored(): Theme {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark" || v === "auto") return v;
  } catch { /* localStorage unavailable */ }
  return "auto";
}

function writeStored(theme: Theme) {
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch { /* localStorage unavailable */ }
}

interface Ctx {
  theme: Theme;
  setTheme: (t: Theme) => void;
}

const ThemeCtx = createContext<Ctx | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(readStored);

  useEffect(() => {
    if (theme === "auto") {
      applyClass(resolveAuto());
      const mq = matchMedia("(prefers-color-scheme: dark)");
      const onChange = () => applyClass(resolveAuto());
      mq.addEventListener("change", onChange);
      return () => mq.removeEventListener("change", onChange);
    }
    applyClass(theme);
  }, [theme]);

  const setTheme = (t: Theme) => {
    writeStored(t);
    setThemeState(t);
  };

  return (
    <ThemeCtx.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeCtx.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeCtx);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}