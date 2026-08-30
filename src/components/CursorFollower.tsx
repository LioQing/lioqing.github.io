import { useEffect, useRef } from "react";

// Follower parameter.
const SPRING = 40;
const DAMPING = 4;
const RADIUS = 360; // outer edge of the follower's reveal
const FALLOFF = RADIUS * 0.2; // fully revealed within this distance
const FADE_MS = 400;
const FPS = 60;

// Dot grid.
const SPACING = 24; // px between dot centers
const REVEAL_RADIUS = 1.5 // dot radius inside the follower
const HIDDEN_RADIUS = 1; // dot radius outside the follower
const REVEAL_ALPHA = 1.0; // dot opacity inside the follower
const HIDDEN_ALPHA = 0.1; // dot opacity outside the follower
const DECAY = 1.2; // fade-out rate once the follower moves away
const DOT_COLOR_LIGHT = "163, 163, 163"; // neutral-400
const DOT_COLOR_DARK = "64, 64, 64"; // neutral-700
const PRUNE_MARGIN = 6; // cells beyond the viewport to keep alive before dropping

function dotColor(): string {
  return document.documentElement.classList.contains("dark") ? DOT_COLOR_DARK : DOT_COLOR_LIGHT;
}

type Dot = { cx: number; cy: number; r: number; a: number };

export default function CursorFollower() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const hasFinePointer = matchMedia("(pointer: fine)").matches;
    if (!hasFinePointer) return;

    // Dots are keyed by their grid cell in *document* coordinates, so they
    // scroll with the page. Each entry persists while visible (plus a margin)
    // to carry its reveal/fade state as the follower moves through the page.
    const dots = new Map<string, Dot>();
    let width = 0;
    let height = 0;

    let last_frame = Date.now();
    let targetX = -9999;
    let targetY = -9999;
    let px = -9999;
    let py = -9999;
    let vx = 0;
    let vy = 0;
    let raf = 0;
    let visible = false;

    // Smoothed scroll position (document coords) with its own spring, so the
    // follower lags and overshoots when the page scrolls.
    let sx = window.scrollX;
    let sy = window.scrollY;
    let svx = 0;
    let svy = 0;

    // Dots are keyed by their grid cell in *document* coordinates, so they
    // scroll with the page. Each entry persists while visible (plus a margin)
    // to carry its reveal/fade state as the follower moves through the page.
    const layout = () => {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = window.innerWidth;
      height = window.innerHeight;
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const setVisible = (v: boolean) => {
      if (v === visible) return;
      visible = v;
      canvas.style.transition = `opacity ${FADE_MS}ms ease-out`;
      canvas.style.opacity = v ? "1.0" : "0.5";
    };

    const onMove = (e: PointerEvent) => {
      targetX = e.clientX;
      targetY = e.clientY;
      if (!visible) setVisible(true);
    };

    const onLeave = () => {
      setVisible(false);
    };

    const onEnter = (e: PointerEvent) => {
      if (e.pointerType === "mouse") {
        targetX = e.clientX;
        targetY = e.clientY;
        setVisible(true);
      }
    };

    // Reveal strength: 1 inside FALLOFF, 0 at/beyond RADIUS, smooth between.
    const reveal = (dist: number) => {
      if (dist <= FALLOFF) return 1;
      if (dist >= RADIUS) return 0;
      const t = (dist - FALLOFF) / (RADIUS - FALLOFF);
      return 1 - t * t * (3 - 2 * t);
    };

    const tick = () => {
      const curr_frame = Date.now();
      const dt = Math.min((curr_frame - last_frame) / 1000, 0.033);

      if (dt < 1 / FPS) {
        raf = requestAnimationFrame(tick);
        return;
      }
      last_frame = curr_frame;

      // Advance the follower spring toward the pointer (viewport coords).
      const dx = targetX - px;
      const dy = targetY - py;
      const ax = SPRING * dx - DAMPING * vx;
      const ay = SPRING * dy - DAMPING * vy;
      vx += ax * dt;
      vy += ay * dt;
      px += vx * dt;
      py += vy * dt;

      const scrollX = window.scrollX;
      const scrollY = window.scrollY;

      const scrollDx = scrollX - sx;
      const scrollDy = scrollY - sy;
      const sax = SPRING * scrollDx - DAMPING * svx;
      const say = SPRING * scrollDy - DAMPING * svy;
      svx += sax * dt;
      svy += say * dt;
      sx += svx * dt;
      sy += svy * dt;

      const fx = px + sx;
      const fy = py + sy;

      const minCx = Math.floor(scrollX / SPACING);
      const maxCx = Math.floor((scrollX + width) / SPACING);
      const minCy = Math.floor(scrollY / SPACING);
      const maxCy = Math.floor((scrollY + height) / SPACING);

      // Exponential decay toward a smaller/more-transparent target.
      const decay = 1 - Math.exp(-DECAY * dt);

      ctx.clearRect(0, 0, width, height);
      ctx.fillStyle = `rgb(${dotColor()})`;

      for (let cy = minCy; cy <= maxCy; cy++) {
        for (let cx = minCx; cx <= maxCx; cx++) {
          const key = `${cx},${cy}`;
          let dot = dots.get(key);
          if (!dot) {
            dot = { cx, cy, r: HIDDEN_RADIUS, a: HIDDEN_ALPHA };
            dots.set(key, dot);
          }

          const wx = cx * SPACING;
          const wy = cy * SPACING;
          const t = reveal(Math.hypot(wx - fx, wy - fy));

          const targetR = HIDDEN_RADIUS + (REVEAL_RADIUS - HIDDEN_RADIUS) * t;
          const targetA = HIDDEN_ALPHA + (REVEAL_ALPHA - HIDDEN_ALPHA) * t;

          // Asymmetric easing: snap up when revealing, decay slowly when fading.
          if (targetR > dot.r) dot.r = targetR;
          else dot.r += (targetR - dot.r) * decay;
          if (targetA > dot.a) dot.a = targetA;
          else dot.a += (targetA - dot.a) * decay;

          if (dot.a <= 0.004) continue;

          ctx.globalAlpha = dot.a;
          ctx.beginPath();
          ctx.arc(wx - scrollX, wy - scrollY, dot.r, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      ctx.globalAlpha = 1;

      // Drop dots that have scrolled well beyond the viewport.
      const pruneMinCx = minCx - PRUNE_MARGIN;
      const pruneMaxCx = maxCx + PRUNE_MARGIN;
      const pruneMinCy = minCy - PRUNE_MARGIN;
      const pruneMaxCy = maxCy + PRUNE_MARGIN;
      for (const [key, dot] of dots) {
        if (
          dot.cx < pruneMinCx ||
          dot.cx > pruneMaxCx ||
          dot.cy < pruneMinCy ||
          dot.cy > pruneMaxCy
        ) {
          dots.delete(key);
        }
      }

      raf = requestAnimationFrame(tick);
    };

    document.addEventListener("pointermove", onMove, { passive: true });
    document.addEventListener("pointerleave", onLeave);
    document.addEventListener("pointerenter", onEnter);
    window.addEventListener("resize", layout);

    layout();
    raf = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(raf);
      document.removeEventListener("pointermove", onMove);
      document.removeEventListener("pointerleave", onLeave);
      document.removeEventListener("pointerenter", onEnter);
      window.removeEventListener("resize", layout);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden
      className="pointer-events-none fixed left-0 top-0 -z-20"
      style={{ opacity: 0 }}
    />
  );
}
