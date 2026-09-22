import { useState } from "react";
import SectionTitle from "./SectionTitle";
import ProjectCard from "./ProjectCard";
import type { Project } from "../types";

export default function Projects({ projects }: { projects: Project[] }) {
  const [expanded, setExpanded] = useState<number | null>(null);

  return (
    <section id="projects" className="section-div">
      <SectionTitle id="projects">Projects</SectionTitle>
      <div className="section-div" />
      <div className="section-bd">
        <p className="content-bd mb-8">
         A non-exhaustive list of software engineering projects I've done ever since I started learning programming in secondary school. Click on the cards below to see more details on each one!
        </p>
        {projects.map((p, i) => (
          <ProjectCard
            key={p.title}
            p={p}
            expanded={expanded === i}
            onToggle={() => setExpanded(expanded === i ? null : i)}
          />
        ))}
      </div>
    </section>
  );
}
