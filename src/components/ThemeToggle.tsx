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
      className="fixed top-4 right-4 z-50 flex size-9 items-center justify-center rounded-full border border-neutral-300 bg-white text-base leading-none transition-colors hover:bg-neutral-100 dark:border-neutral-600 dark:bg-neutral-800 dark:hover:bg-neutral-700"
      aria-label={`Theme: ${LABELS[theme]}`}
      title={`Theme: ${LABELS[theme]} — click for ${LABELS[NEXT[theme]]}`}
    >
      {ICONS[theme]}
    </button>
  );
}