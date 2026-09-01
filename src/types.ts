export interface Project {
  title: string;
  tags: string[];
  year: number;
  month: number;
  subtitle: string;
  description: string;
  image_url: string;
  links: { label: string; url: string }[];
}

export interface Skill {
  name: string;
  tags: string[];
}

export interface Experience {
  name: string;
  organization: string;
  category: "work" | "education" | "community";
  start_year?: number;
  start_month?: number;
  end_year?: number;
  end_month?: number;
}
