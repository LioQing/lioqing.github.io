import { ThemeProvider } from "./components/ThemeProvider";
import FloatingNavigation from "./components/FloatingNavigation";
import experiencesJson from "./data/experiences.json";
import projectsJson from "./data/projects.json";
import Experiences from "./components/Experiences";
import Projects from "./components/Projects";
import Skills from "./components/Skills";
import CursorFollower from "./components/CursorFollower";
import type { Experience, Project } from "./types";
import SectionTitle from "./components/SectionTitle";
import SocialLinkButton from "./components/SocialLinkButton";
import LioQing3D from "./data/lio-qing-3d.svg?react";

const projects = projectsJson as Project[];
const experiences = experiencesJson as Experience[];
export default function App() {
  return (
    <ThemeProvider>
      <CursorFollower />
      <FloatingNavigation />
      <div className="content-bd relative min-h-screen leading-relaxed">
        <div
          aria-hidden
          className="max-w-section section-div pointer-events-none absolute inset-y-0 left-1/2 w-full -translate-x-1/2 border-x"
        />
        <section aria-label="Lio Qing 3D illustration" className="section-div hero-section">
          <div className="section-bd overflow-visible">
            <LioQing3D aria-hidden className="lioqing-3d block h-auto w-fit" />
          </div>
        </section>
        <section id="about" className="section-div">
          <SectionTitle id="about">Lio Qing</SectionTitle>
          <div className="section-div" />
          <div className="section-bd">
            <p className="content-bd">
              Hi! I&rsquo;m Lio - a software engineer from Hong Kong, currently working at The Trade
              Desk, and am interested in web technology, computer graphics, AI, and in general enjoy
              building tools that help people learn and create.
            </p>
            <div className="mt-8 flex flex-row gap-4">
              <SocialLinkButton url="https://www.linkedin.com/in/lioqyz/">
                LinkedIn
              </SocialLinkButton>
              <SocialLinkButton url="https://github.com/lioqing">GitHub</SocialLinkButton>
            </div>
          </div>
        </section>

        <Projects projects={projects} />
        <Skills />
        <Experiences experiences={experiences} />
        <div className="bg-border h-px" />
        <footer className="section-div">
          <div className="section-bd py-8">
            <p className="content-bd text-content-muted text-center text-sm">
              &copy; {new Date().getFullYear()} Lio Qing
            </p>
          </div>
        </footer>
        <section aria-hidden="true" className="section-div">
          <div className="section-hd">
            <div className="section-striped h-48" />
          </div>
        </section>
      </div>
    </ThemeProvider>
  );
}
