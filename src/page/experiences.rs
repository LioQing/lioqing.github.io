use itertools::Itertools;
use leptos::prelude::*;

use crate::page::{Content, Spacer};

macro_rules! experience {
    (
        $position:expr,
        $organization:expr,
        $expr_ty:ident,
        $start_year:expr,
        $start_month:expr,
        $end_year:expr,
        $end_month:expr $(,)?
    ) => {
        Experience {
            position: $position,
            organization: $organization,
            ty: Type::$expr_ty,
            start: chrono::NaiveDate::from_ymd_opt($start_year, $start_month, 1).unwrap(),
            end: Some(chrono::NaiveDate::from_ymd_opt($end_year, $end_month, 1).unwrap()),
        }
    };
    (
        $position:expr,
        $organization:expr,
        $expr_ty:ident,
        $start_year:expr,
        $start_month:expr $(,)?
    ) => {
        Experience {
            position: $position,
            organization: $organization,
            ty: Type::$expr_ty,
            start: chrono::NaiveDate::from_ymd_opt($start_year, $start_month, 1).unwrap(),
            end: None,
        }
    };
}

struct Experience {
    position: &'static str,
    organization: &'static str,
    ty: Type,
    start: chrono::NaiveDate,
    end: Option<chrono::NaiveDate>,
}

const EXPERIENCES: &[Experience] = &[
    experience! {
        "Simple Reinforcement Learning Driver Workshop Instructor",
        "St. Paul's College",
        ProfessionalWork,
        2025,
        3,
    },
    experience! {
        "AI Student Research Assistant",
        "University of Hong Kong Innovation Wing AI Team",
        ProfessionalWork,
        2024,
        10,
    },
    experience! {
        "Google Developer Group Organizer",
        "Google Developer Group on Campus University of Hong Kong",
        CommunityService,
        2024,
        10,
    },
    experience! {
        "Software Engineer Intern",
        "The Trade Desk",
        ProfessionalWork,
        2024,
        6,
        2024,
        9,
    },
    experience! {
        "Student Interest Group Leader",
        "University of Hong Kong GenNarrator AI Student Interest Group",
        CommunityService,
        2023,
        12,
        2024,
        8,
    },
    experience! {
        "Student Ambassador",
        "University of Hong Kong",
        CommunityService,
        2023,
        10,
    },
    experience! {
        "Data Science Intern",
        "Asklora",
        ProfessionalWork,
        2023,
        6,
        2023,
        8,
    },
    experience! {
        "Data Structure & Algorithm Workshop Tutor",
        "St. Paul's College",
        ProfessionalWork,
        2023,
        3,
        2023,
        4,
    },
    experience! {
        "Student Tutor Assistant",
        "University of Hong Kong Computer Science Department",
        CommunityService,
        2022,
        9,
        2023,
        11,
    },
    experience! {
        "IT Strategic Support Intern",
        "HKSAR Electrical & Mechanical Service Department",
        ProfessionalWork,
        2022,
        6,
        2022,
        8,
    },
    experience! {
        "University Undergraduate Student",
        "University of Hong Kong Bachelor of Science in Computer Science",
        Education,
        2021,
        9,
        2025,
        6,
    },
    experience! {
        "Biology Society & Computer Society Committee Member",
        "St. Paul's College",
        CommunityService,
        2018,
        9,
        2021,
        7,
    },
    experience! {
        "Rubik's Cube Club Committee Member",
        "St. Paul's College",
        CommunityService,
        2018,
        9,
        2019,
        7,
    },
    experience! {
        "Gao Hu Member",
        "Hong Kong Young Chinese Orchestra",
        CommunityService,
        2017,
        7,
        2021,
        7,
    },
    experience! {
        "IGCSE Spanish Student",
        "St. Paul's College joint with HKU School of Modern Languages and Cultures",
        Education,
        2015,
        9,
        2019,
        6,
    },
    experience! {
        "DSE Secondary School Student",
        "St. Paul's College",
        Education,
        2015,
        9,
        2021,
        7,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Type {
    ProfessionalWork,
    CommunityService,
    Education,
}

impl Type {
    pub const fn name(self) -> &'static str {
        match self {
            Type::ProfessionalWork => "Professional Work",
            Type::CommunityService => "Community Service",
            Type::Education => "Education",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Type::ProfessionalWork => "ph-briefcase",
            Type::CommunityService => "ph-users-three",
            Type::Education => "ph-graduation-cap",
        }
    }
}

#[component]
pub fn Experiences() -> impl IntoView {
    view! {
        <Content
            icon="ph-handshake"
            title="Experiences"
            subtitle="Journey of professional work and education."
        >
            <Spacer/>
            {Itertools::intersperse_with(
                EXPERIENCES.iter().map(|experience| {
                    view! {
                        <Items
                            position={experience.position}
                            ty={experience.ty}
                            organization={experience.organization}
                            start={experience.start}
                            end={experience.end}
                        />
                    }.into_any()
                }),
                || view! { <Spacer/> }.into_any(),
            ).collect_view()}
        </Content>
    }
}

#[component]
fn Items(
    position: &'static str,
    ty: Type,
    organization: &'static str,
    start: chrono::NaiveDate,
    end: Option<chrono::NaiveDate>,
) -> impl IntoView {
    view! {
        <div style="\
            display: flex; \
            flex-direction: row; \
            justify-content: start; \
            align-items: center; \
            gap: 1em; \
            padding-left: 0.4em; \
            margin-bottom: 0.4em; \
        ">
            <h3>{position}</h3>
        </div>
        <div style="\
            display: flex; \
            flex-direction: row; \
            flex-wrap: wrap; \
            justify-content: start; \
            align-items: center; \
            gap: 0.4em; \
            padding-left: 0.4em; \
        ">
            <i class="ph ph-building"/>
            <p>{organization}</p>
            |
            <i class=format!("ph {}", ty.icon())/>
            <p>{ty.name()}</p>
            |
            <i class="ph ph-calendar"/>
            <p>
                {format!(
                    "{} - {}",
                    start.format("%b %Y"),
                    end.map_or("Present".to_string(), |end| end.format("%b %Y").to_string())
                )}
            </p>
            |
            <i class="ph ph-timer"/>
            <p>
                {duration_string(start, end)}
            </p>
        </div>
    }
}

fn duration_string(start: chrono::NaiveDate, end: Option<chrono::NaiveDate>) -> String {
    let end_date = end.unwrap_or_else(|| chrono::Local::now().naive_local().date());
    let duration = end_date.signed_duration_since(start);

    let total_days = duration.num_days();
    let years = total_days / 365;
    let months = (total_days % 365) / 30;

    match (years, months) {
        (0, m) => format!("{} month{}", m, if m == 1 { "" } else { "s" }),
        (y, 0) => format!("{} year{}", y, if y == 1 { "" } else { "s" }),
        (y, m) => format!(
            "{} year{} {} month{}",
            y,
            if y == 1 { "" } else { "s" },
            m,
            if m == 1 { "" } else { "s" }
        ),
    }
}
