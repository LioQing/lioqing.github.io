import { useSyncExternalStore } from "react";

const QUERY = "(prefers-reduced-motion: reduce)";

function subscribe(onChange: () => void) {
  const media = matchMedia(QUERY);
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}

function getSnapshot() {
  return matchMedia(QUERY).matches;
}

export default function useReducedMotion() {
  return useSyncExternalStore(subscribe, getSnapshot);
}
