import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import SectionTitle from "./SectionTitle";
import type { Experience } from "../types";

function fmtDate(y: number, m: number) {
  return `${y}-${String(m).padStart(2, "0")}`;
}

function dateRange(e: Experience) {
  const hasStart = e.start_year != null;
  const hasEnd = e.end_year != null;
  if (hasStart && hasEnd)
    return `${fmtDate(e.start_year!, e.start_month!)} - ${fmtDate(e.end_year!, e.end_month!)}`;
  if (hasStart) return `${fmtDate(e.start_year!, e.start_month!)} - Present`;
  if (hasEnd) return fmtDate(e.end_year!, e.end_month!);
  return null;
}

type GroupBy = "category" | "start" | "end";

const CATEGORY_LABELS: Record<Experience["category"], string> = {
  work: "Work",
  education: "Education",
  community: "Community",
};

const CATEGORY_ORDER: Experience["category"][] = ["work", "education", "community"];

const OPTIONS: { value: GroupBy; label: string }[] = [
  { value: "category", label: "Category" },
  { value: "start", label: "Start" },
  { value: "end", label: "End" },
];

/** Most recent first, within a group. */
function itemSortKey(e: Experience): [number, number] {
  const y = e.start_year ?? e.end_year ?? 0;
  const m = e.start_month ?? e.end_month ?? 0;
  return [y, m];
}

/**
 * Wraps a group (heading + entries) with a one-time reveal transition.
 * When the group scrolls into view, the accent line grows top-to-bottom
 * while the entries reveal in sync (staggered top-to-bottom).
 */
function RevealGroup({
  header,
  items,
  renderItem,
}: {
  header: ReactNode;
  items: Experience[];
  renderItem: (item: Experience, index: number) => ReactNode;
}) {
  const groupRef = useRef<HTMLDivElement>(null);
  const [shown, setShown] = useState(false);

  useEffect(() => {
    const el = groupRef.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setShown(true);
          io.disconnect();
        }
      },
      { threshold: 0.2, rootMargin: "0px 0px -10% 0px" },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div ref={groupRef} className="relative mb-5 pl-8">
      <div
        aria-hidden
        className="bg-accent absolute top-0 bottom-0 left-0 w-0.5"
        style={{
          transform: shown ? "scaleY(1)" : "scaleY(0)",
          transformOrigin: "top",
          transition: "transform 700ms cubic-bezier(0.4, 0, 0.2, 1)",
        }}
      />
      {header}
      {items.map((item, i) => (
        <div
          key={`${item.name}-${item.organization}-${i}`}
          className="will-change-transform mb-3 py-3.5"
          style={{
            opacity: shown ? 1 : 0,
            transform: shown ? "translateY(0)" : "translateY(16px)",
            transition:
              "opacity 500ms ease, transform 500ms cubic-bezier(0.4, 0, 0.2, 1)",
            transitionDelay: `${i * 120}ms`,
          }}
        >
          {renderItem(item, i)}
        </div>
      ))}
    </div>
  );
}

export default function Experiences({ experiences }: { experiences: Experience[] }) {
  const [groupBy, setGroupBy] = useState<GroupBy>("category");

  const groupRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const [indicator, setIndicator] = useState({ x: 0, w: 0 });

  useLayoutEffect(() => {
    const measure = () => {
      const group = groupRef.current;
      const item = itemRefs.current[groupBy];
      if (!group || !item) return;
      const groupRect = group.getBoundingClientRect();
      const itemRect = item.getBoundingClientRect();
      setIndicator({ x: itemRect.left - groupRect.left, w: itemRect.width });
    };
    measure();
    const ro = new ResizeObserver(measure);
    if (groupRef.current) ro.observe(groupRef.current);
    window.addEventListener("resize", measure);
    return () => {
      ro.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, [groupBy]);

  const keyOf = (e: Experience): string => {
    if (groupBy === "category") return e.category;
    const year =
      groupBy === "start" ? (e.start_year ?? e.end_year) : (e.end_year ?? e.start_year);
    return String(year);
  };

  const groups = new Map<string, Experience[]>();
  for (const e of experiences) {
    const key = keyOf(e);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(e);
  }

  let keys: string[];
  if (groupBy === "category") {
    keys = CATEGORY_ORDER.filter((k) => groups.has(k));
  } else {
    keys = [...groups.keys()].sort((a, b) => Number(b) - Number(a));
  }

  const headerOf = (key: string): string =>
    groupBy === "category"
      ? CATEGORY_LABELS[key as Experience["category"]]
      : key;

  return (
    <section id="experiences">
      <SectionTitle id="experiences">Experiences</SectionTitle>
      <div className="section-div" />
      <div className="section-bd pb-24">
        <p className="content-bd mb-8">
          My professional and educational experiences, as well as my contributions to the community. I have been fortunate to work with some amazing organizations and people, and I am grateful for the opportunities I have had to learn and grow.
        </p>
        <div
          ref={groupRef}
          className="relative mb-6 flex flex-row gap-4"
          role="group"
          aria-label="Group experiences by"
        >
          Group by:
          {OPTIONS.map((o) => {
            const active = groupBy === o.value;
            return (
              <div
                key={o.value}
                ref={(el) => {
                  itemRefs.current[o.value] = el;
                }}
                onClick={() => setGroupBy(o.value)}
                className="group/link-btn cursor-pointer relative"
              >
                <div className="absolute inset-0 link-btn-shadow" />
                <button
                  type="button"
                  aria-pressed={active}
                  className="link-btn cursor-pointer"
                >
                  {o.label}
                </button>
              </div>
            );
          })}
          <div
            aria-hidden
            className="pointer-events-none absolute top-0 bottom-0 left-0 transition-all ease-in-out"
            style={{
              transform: `translateX(${indicator.x}px)`,
              width: `${indicator.w}px`,
            }}
          >
            <span className="experiences-toggle-corner -top-1.5 -left-1.5 border-t border-l" />
            <span className="experiences-toggle-corner -top-1.5 -right-1.5 border-t border-r" />
            <span className="experiences-toggle-corner -bottom-1.5 -left-1.5 border-b border-l" />
            <span className="experiences-toggle-corner -bottom-1.5 -right-1.5 border-b border-r" />
          </div>
        </div>
        {keys.map((key) => {
          const items = groups.get(key)!;
          items.sort((a, b) => itemSortKey(b)[0] - itemSortKey(a)[0] || itemSortKey(b)[1] - itemSortKey(a)[1]);
          return (
            <RevealGroup
              key={`${groupBy}-${key}`}
              header={
                <h3 className="mb-4 text-2xl font-light text-content-muted tracking-wider uppercase">
                  {headerOf(key)}
                </h3>
              }
              items={items}
              renderItem={(e) => (
                <>
                  <div className="font-bold text-xl">
                    {e.name}
                    {groupBy !== "category" && (
                      <span className="tag font-normal ml-2">
                        {e.category.charAt(0).toUpperCase() + e.category.slice(1)}
                      </span>
                    )}
                  </div>
                  <div className="mt-1 text-sm text-content-muted">{e.organization}</div>
                  {dateRange(e) && (
                    <div className="text-sm text-content-muted">{dateRange(e)}</div>
                  )}
                </>
              )}
            />
          );
        })}
      </div>
    </section>
  );
}
