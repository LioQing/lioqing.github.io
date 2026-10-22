import { GraduationCap, NotebookPen, Sparkle, SquareSparkles, type LucideIcon } from "lucide-react";
import SectionTitle from "./SectionTitle";
import ExternalLinkIcon from "../data/ext-lnk.svg?react";
import links from "../data/explore.json";

const ICONS: Record<string, LucideIcon> = {
  "square-sparkles": SquareSparkles,
  sparkle: Sparkle,
  "graduation-cap": GraduationCap,
  "notebook-pen": NotebookPen,
};

export default function Explore() {
  return (
    <section id="explore" className="section-div">
      <SectionTitle id="explore">Explore</SectionTitle>
      <div className="section-div" />
      <div className="section-bd pb-24">
        <p className="content-bd mb-8">
          A few collections of things I&rsquo;ve built, used, and captured.
        </p>
        <ul className="flex flex-col gap-6">
          {links.map((link) => {
            const Icon = ICONS[link.icon];
            return (
              <li key={link.url} className="relative">
                <div aria-hidden="true" className="link-btn-shadow absolute inset-0" />
                <a
                  href={link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="group/explore explore-link"
                >
                  <div className="flex items-center gap-3 sm:gap-6">
                    <Icon
                      aria-hidden="true"
                      strokeWidth={1.5}
                      className="group-focus-visible/explore:text-accent group-hover/explore:pointer-fine:text-accent size-8 shrink-0 transition-colors duration-200 sm:size-16"
                    />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-start justify-between gap-4">
                        <h3 className="text-xl font-black tracking-tight wrap-anywhere sm:text-3xl">
                          {link.title}
                        </h3>
                        <ExternalLinkIcon
                          aria-hidden="true"
                          className="mt-1 hidden size-6 shrink-0 sm:block"
                        />
                      </div>
                      <p className="text-content-muted mt-3 font-mono text-xs leading-relaxed wrap-anywhere sm:text-sm">
                        {link.url}
                      </p>
                    </div>
                  </div>
                </a>
              </li>
            );
          })}
        </ul>
      </div>
    </section>
  );
}
