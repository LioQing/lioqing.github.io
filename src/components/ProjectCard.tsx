import type { Project } from "../types";
import ExternalLinkIcon from "../data/ext-lnk.svg?react";

const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

export default function ProjectCard({
  p,
  expanded,
  onToggle,
}: {
  p: Project;
  expanded: boolean;
  onToggle: () => void;
}) {
  const month = MONTHS[p.month - 1] ?? p.month;

  return (
    <div className={`group relative ${expanded ? "group-expanded" : ""}`}>
      <div className="project-card-shadow" />
      <article
        className="project-card overflow-hidden mb-8 flex flex-col gap-8 py-0 pb-8"
        aria-expanded={expanded}
        onClick={() => {
          const selection = window.getSelection();
          if (selection && selection.toString().trim().length > 0) {
            return;
          }
          onToggle();
        }}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.target !== e.currentTarget) {
            return;
          }
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onToggle();
          }
        }}
      >
        <div className="flex flex-row py-1 px-2 w-full border-b border-border items-center">
          <div
            className={`w-[0.4em] h-[0.4em] rounded-full mr-1.5 transition-colors ${
              expanded ? "bg-accent-contrast group-hover:bg-accent-contrast" : "bg-border group-hover:bg-content-muted"
            }`}
          />
          <pre className="text-border transition-all cursor-text group-hover:text-content-muted group-expanded:text-content-muted text-xs">
            project={`${p.title.replaceAll(/[^a-zA-Z0-9\s]/g, '').trim().replaceAll(' ', '-').toLowerCase()}`}
          </pre>
        </div>
        <div className="px-8">
          <div className="flex flex-col gap-8 lg:flex-row">
            <div className="order-2 min-w-0 flex-1 lg:order-1">
              <div className="mt-1 cursor-text text-sm text-content-muted">
                {month} {p.year}
              </div>

              <h3 className="font-black text-4xl pb-4 cursor-text">{p.title}</h3>

              <div className="mt-1 cursor-text">
                {p.subtitle}
              </div>

              <div className="mt-4 flex flex-wrap gap-1.5">
                {p.tags.map((t) => (
                  <span key={t} className="tag">
                    {t}
                  </span>
                ))}
              </div>
            </div>

            <div className="order-1 w-fit lg:order-2">
              <img
                src={`/projects/${p.image_url}`}
                alt={p.title}
                loading="lazy"
                className="max-h-64 w-auto max-w-full border border-border select-none"
              />
            </div>
          </div>

          <div
            className={`grid transition-[grid-template-rows] duration-300 ${
              expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
            }`}
          >
            <div className="min-h-0">
              <div className="mb-2.5 pt-8 cursor-text">
                {p.description}
              </div>

              {p.links.length > 0 && (
                <div className="flex flex-wrap gap-4">
                  {p.links.map((l) => (
                    <div
                      key={l.label}
                      className="group/link-btn relative"
                    >
                      <div className="absolute inset-0 link-btn-shadow" />
                      <a
                        key={l.label}
                        href={l.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="link-btn"
                        onClick={(e) => e.stopPropagation()}
                      >
                        {l.label}
                        <ExternalLinkIcon className="w-3.5 h-3.5" />
                      </a>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </article>
    </div>
  );
}
