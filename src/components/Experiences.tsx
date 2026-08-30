import SectionTitle from "./SectionTitle";
import type { Experience } from "../types";

function fmtDate(y: number, m: number) {
  return `${y}-${String(m).padStart(2, "0")}`;
}

function dateRange(e: Experience) {
  const hasStart = e.start_year != null;
  const hasEnd = e.end_year != null;
  if (hasStart && hasEnd)
    return `${fmtDate(e.start_year!, e.start_month!)} → ${fmtDate(e.end_year!, e.end_month!)}`;
  if (hasStart) return `${fmtDate(e.start_year!, e.start_month!)} → Present`;
  if (hasEnd) return fmtDate(e.end_year!, e.end_month!);
  return null;
}

export default function Experiences({ experiences }: { experiences: Experience[] }) {
  const groups = {
    work: [] as Experience[],
    education: [] as Experience[],
    community: [] as Experience[],
  };
  for (const e of experiences) {
    if (!e.filter) continue;
    groups[e.filter!]?.push(e);
  }

  const order: (keyof typeof groups)[] = ["work", "education", "community"];
  const labels: Record<keyof typeof groups, string> = {
    work: "Work",
    education: "Education",
    community: "Community",
  };

  return (
    <section id="experiences">
      <SectionTitle id="experiences">Experiences</SectionTitle>
      <div className="section-div" />
      <div className="section-bd pb-24">
        {order.map((key) => {
          const items = groups[key];
          if (!items.length) return null;
          return (
            <div key={key} className="mb-5">
              <h3 className="mb-4 text-2xl font-light text-content-muted tracking-wider uppercase">
                {labels[key]}
              </h3>
              {items.map((e, i) => (
                <div
                  key={`${e.name}-${e.organization}-${i}`}
                  className="mb-3 px-5 py-3.5"
                >
                  <div className="font-bold text-xl">{e.name}</div>
                  <div className="mt-1 text-sm text-content-muted">{e.organization}</div>
                  {dateRange(e) && (
                    <div className="text-sm text-content-muted">{dateRange(e)}</div>
                  )}
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </section>
  );
}
