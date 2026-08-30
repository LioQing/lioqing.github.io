import { useEffect, useRef, useState, type CSSProperties } from "react";
import SectionTitle from "./SectionTitle";
import skillsJson from "../data/skills.json";
import type { Skill } from "../types";

const skills = skillsJson as Skill[];
const TYPING_STEP_MS = 55;

export default function Skills() {
  const [typed, setTyped] = useState<number[]>(() => skills.map(() => 0));
  const itemsRef = useRef<(HTMLDivElement | null)[]>([]);
  const countsRef = useRef<number[]>(skills.map(() => 0));
  const timersRef = useRef<Record<number, ReturnType<typeof setInterval>>>({});

  useEffect(() => {
    const items = itemsRef.current.filter(Boolean) as HTMLDivElement[];
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          const el = entry.target as HTMLDivElement;
          const i = Number(el.dataset.index);
          observer.unobserve(el);
          if (timersRef.current[i] != null) return;

          el.classList.add("is-typing");
          timersRef.current[i] = setInterval(() => {
            const count = countsRef.current[i];
            if (count < skills[i].name.length) {
              countsRef.current[i] = count + 1;
              setTyped((prev) => {
                const next = [...prev];
                next[i] = count + 1;
                return next;
              });
            } else {
              clearInterval(timersRef.current[i]);
              delete timersRef.current[i];
              el.classList.remove("is-typing");
              el.classList.add("typing-done");
            }
          }, TYPING_STEP_MS);
        });
      },
      { threshold: 0.35, rootMargin: "0px 0px -12% 0px" }
    );
    items.forEach((el) => observer.observe(el));
    return () => {
      observer.disconnect();
      Object.values(timersRef.current).forEach((t) => clearInterval(t));
      timersRef.current = {};
      countsRef.current = skills.map(() => 0);
    };
  }, []);

  return (
    <section id="skills" className="section-div">
      <SectionTitle id="skills">Skills</SectionTitle>
      <div className="section-div" />
      <div className="section-bd">
        <p className="content-bd mb-8">
         Some computer skills that I am productive in, these include the programming languages, frameworks, ecosystems, softwares, technical skills that I have been working with.
        </p>
        <div className="flex flex-col items-center gap-12 pt-2">
          {skills.map((s, i) => {
            const left = i % 2 === 0;
            return (
              <div
                key={s.name}
                ref={(el) => {
                  itemsRef.current[i] = el;
                }}
                data-index={i}
                className={`skill-item flex flex-col items-center gap-4 ${
                  left ? "md:-translate-x-24 md:-rotate-1" : "md:translate-x-24 md:rotate-1"
                }`}
              >
                <h3 className="skill-name" aria-label={s.name}>
                  <span className="skill-name-text">{s.name.slice(0, typed[i])}</span>
                </h3>
                <ul className="flex flex-row flex-wrap justify-center gap-2">
                  {s.tags.map((t, j) => {
                    const tilt = (-(j - (s.tags.length - 1) / 2)) * 12 + (left ? -2 : 2);
                    return (
                      <li
                        key={t}
                        className="skill-tag"
                        style={
                          {
                            "--tag-tilt": `${tilt}deg`,
                            "--tag-delay": `${j * 90}ms`,
                          } as CSSProperties
                        }
                      >
                        {t}
                      </li>
                    );
                  })}
                </ul>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
