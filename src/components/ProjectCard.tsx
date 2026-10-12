import { useId, type CSSProperties } from "react";
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
  const detailsId = useId();

  return (
    <div className={`group relative ${expanded ? "group-expanded" : ""}`}>
      <div className="project-card-shadow" />
      <article
        className="project-card mb-8 flex flex-col gap-8 overflow-hidden py-0 pb-8"
        onClick={() => {
          const selection = window.getSelection();
          if (selection && selection.toString().trim().length > 0) {
            return;
          }
          onToggle();
        }}
      >
        <div className="border-border flex w-full flex-row items-center border-b px-2 py-1">
          <div
            className={`mr-1.5 h-[0.4em] w-[0.4em] rounded-full transition-colors ${
              expanded
                ? "bg-accent-contrast group-hover:bg-accent-contrast"
                : "bg-border group-hover:bg-content-muted"
            }`}
          />
          <pre className="text-border group-hover:text-content-muted group-expanded:text-content-muted cursor-text text-xs transition-all">
            project=
            {`${p.title
              .replaceAll(/[^a-zA-Z0-9\s]/g, "")
              .trim()
              .replaceAll(" ", "-")
              .toLowerCase()}`}
          </pre>
        </div>
        <div className="px-8">
          <div className="flex flex-col gap-8 lg:flex-row">
            <div className="order-2 min-w-0 flex-1 lg:order-1">
              <div className="text-content-muted mt-1 cursor-text text-sm">
                {month} {p.year}
              </div>

              <h3 className="pb-4 text-4xl font-black">
                <button
                  type="button"
                  aria-expanded={expanded}
                  aria-controls={detailsId}
                  className="cursor-pointer text-left focus-visible:outline-2 focus-visible:outline-offset-4"
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggle();
                  }}
                >
                  {p.title}
                </button>
              </h3>

              <div className="mt-1 cursor-text">{p.subtitle}</div>

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
                width={p.image_width}
                height={p.image_height}
                loading="lazy"
                style={
                  {
                    "--image-width": p.image_width,
                    "--image-height": p.image_height,
                  } as CSSProperties
                }
                className="border-border h-auto max-h-64 w-[min(calc(var(--image-width)*1px),calc(16rem*var(--image-width)/var(--image-height)))] max-w-full border select-none"
              />
            </div>
          </div>

          <div
            id={detailsId}
            inert={!expanded}
            aria-hidden={!expanded}
            className={`grid transition-[grid-template-rows] duration-300 ${
              expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
            }`}
          >
            <div className="min-h-0">
              <div className="mb-2.5 cursor-text pt-8">{p.description}</div>

              {p.links.length > 0 && (
                <div className="flex flex-wrap gap-4">
                  {p.links.map((l) => (
                    <div key={l.label} className="group/link-btn relative">
                      <div className="link-btn-shadow absolute inset-0" />
                      <a
                        key={l.label}
                        href={l.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="link-btn"
                        onClick={(e) => e.stopPropagation()}
                      >
                        {l.label}
                        <ExternalLinkIcon className="h-3.5 w-3.5" />
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
