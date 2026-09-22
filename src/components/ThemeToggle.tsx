import { useTheme, type Theme } from "./ThemeProvider";

const ICONS: Record<Theme, string> = {
  auto: "◐",
  light: "☀",
  dark: "☾",
};

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
      aria-label={`Theme: ${LABELS[theme]}`}
      title={`Theme: ${LABELS[theme]} - click for ${LABELS[NEXT[theme]]}`}
    >
      {ICONS[theme]}
    </button>
  );
}