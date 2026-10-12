import { useTheme, type Theme } from "./ThemeProvider";

const NEXT: Record<Theme, Theme> = {
  auto: "light",
  light: "dark",
  dark: "auto",
};

const LABELS: Record<Theme, string> = {
  auto: "Auto",
  light: "Light",
  dark: "Dark",
};

export default function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <button
      type="button"
      onClick={() => setTheme(NEXT[theme])}
      className="theme-toggle"
      aria-label={`Theme: ${LABELS[theme]}. Switch to ${LABELS[NEXT[theme]]}`}
      title={`Theme: ${LABELS[theme]} - click for ${LABELS[NEXT[theme]]}`}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" fill="none">
        {theme === "auto" ? (
          <>
            <circle cx="12" cy="12" r="8" />
            <path d="M12 4a8 8 0 0 1 0 16Z" fill="currentColor" stroke="none" />
          </>
        ) : theme === "light" ? (
          <>
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5" />
          </>
        ) : (
          <path d="M20.5 14A8.5 8.5 0 0 1 10 3.5 8.5 8.5 0 1 0 20.5 14Z" />
        )}
      </svg>
      <span className="theme-toggle-label">{LABELS[theme]}</span>
    </button>
  );
}
