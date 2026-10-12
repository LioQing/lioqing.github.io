import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import ThemeToggle from "./ThemeToggle";

const DESKTOP_QUERY = "(min-width: 80rem)";
const SECTIONS = [
  { id: "about", label: "About" },
  { id: "projects", label: "Projects" },
  { id: "skills", label: "Skills" },
  { id: "experiences", label: "Experiences" },
];

function subscribeDesktop(onChange: () => void) {
  const media = matchMedia(DESKTOP_QUERY);
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}

function isDesktop() {
  return matchMedia(DESKTOP_QUERY).matches;
}

export default function FloatingNavigation() {
  const desktop = useSyncExternalStore(subscribeDesktop, isDesktop);
  const [expanded, setExpanded] = useState(false);
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const navRef = useRef<HTMLElement>(null);
  const toggleRef = useRef<HTMLButtonElement>(null);
  const open = !desktop && expanded;

  useEffect(() => {
    let frame = 0;
    const sections = SECTIONS.map(({ id }) => document.getElementById(id));

    const update = () => {
      frame = 0;
      const readingLine = Math.max(96, Math.min(160, window.innerHeight * 0.25));
      let current: string | null = null;
      for (const section of sections) {
        if (section && section.getBoundingClientRect().top <= readingLine) current = section.id;
      }
      setActiveSection(current);
    };

    const scheduleUpdate = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    const closeOnResize = () => setExpanded(false);
    const media = matchMedia(DESKTOP_QUERY);
    const observer = new ResizeObserver(scheduleUpdate);
    observer.observe(document.body);
    window.addEventListener("scroll", scheduleUpdate, { passive: true });
    window.addEventListener("resize", scheduleUpdate);
    media.addEventListener("change", closeOnResize);
    update();

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
      window.removeEventListener("scroll", scheduleUpdate);
      window.removeEventListener("resize", scheduleUpdate);
      media.removeEventListener("change", closeOnResize);
    };
  }, []);

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: PointerEvent) => {
      if (event.target instanceof Node && !navRef.current?.contains(event.target)) {
        setExpanded(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setExpanded(false);
        toggleRef.current?.focus({ preventScroll: true });
      }
    };

    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  const closeMenu = () => {
    if (open) {
      setExpanded(false);
      toggleRef.current?.focus({ preventScroll: true });
    }
  };

  const scrollToTop = () => {
    closeMenu();
    window.scrollTo({ top: 0, behavior: "auto" });
  };

  return (
    <nav
      ref={navRef}
      className="floating-nav"
      aria-label="Page navigation"
      data-scrolled={activeSection !== null}
      data-expanded={open}
    >
      <button
        className="floating-nav-home"
        type="button"
        aria-label="Lio Qing, back to top"
        aria-current={activeSection === null ? "location" : undefined}
        onClick={scrollToTop}
      >
        Lio Qing
      </button>
      <div className="floating-nav-controls">
        <ThemeToggle />
        <button
          ref={toggleRef}
          className="floating-nav-toggle"
          type="button"
          aria-expanded={open}
          aria-controls="navigation-sections"
          onClick={() => setExpanded((value) => !value)}
        >
          <span>Menu</span>
          <svg aria-hidden="true" viewBox="0 0 20 20" fill="none">
            <path d="m5 8 5 5 5-5" />
          </svg>
        </button>
      </div>
      <div id="navigation-sections" className="floating-nav-sections" inert={!desktop && !open}>
        <div className="floating-nav-sections-inner">
          <ul className="floating-nav-links">
            {SECTIONS.map(({ id, label }) => (
              <li key={id}>
                <a
                  className="floating-nav-link"
                  href={`#${id}`}
                  aria-current={activeSection === id ? "location" : undefined}
                  onClick={closeMenu}
                >
                  {label}
                </a>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </nav>
  );
}
