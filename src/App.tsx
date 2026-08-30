import { ThemeProvider } from "./components/ThemeProvider";
import ThemeToggle from "./components/ThemeToggle";
import experiencesJson from "./data/experiences.json";
import projectsJson from "./data/projects.json";
import Experiences from "./components/Experiences";
import Projects from "./components/Projects";
import Skills from "./components/Skills";
import CursorFollower from "./components/CursorFollower";
import type { Experience, Project } from "./types";
import SectionTitle from "./components/SectionTitle";
import SocialLinkButton from "./components/SocialLinkButton";

const projects = projectsJson as Project[];
const experiences = experiencesJson as Experience[];
export default function App() {
  return (
      <ThemeProvider>
        <CursorFollower />
        <div className="relative min-h-screen leading-relaxed content-bd">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-y-0 left-1/2 w-full max-w-section -translate-x-1/2 section-div border-x"
          />
          <section id="lioqing" className="section-div">
            <SectionTitle id="lioqing">Lio Qing</SectionTitle>
            <div className="section-div" />
            <div className="section-bd">
              <p className="content-bd">
                Hi! I&rsquo;m Lio - a software engineer from Hong Kong, currently working at The Trade Desk, and am interested in web technology, computer graphics, AI, and in general enjoy building
                tools that help people learn and create.
              </p>
              <div className="flex flex-row gap-4 mt-8">
                <SocialLinkButton url="https://www.linkedin.com/in/lioqyz/">LinkedIn</SocialLinkButton>
                <SocialLinkButton url="https://github.com/lioqing">GitHub</SocialLinkButton>
              </div>
            </div>
          </section>

          <Projects projects={projects} />
          <Skills />
          <Experiences experiences={experiences} />
          <ThemeToggle />
        </div>
      </ThemeProvider>
  );
}