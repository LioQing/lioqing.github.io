use std::{fmt::Display, str::FromStr};

use chrono::Duration;
use itertools::Itertools;
use leptos::prelude::*;
use leptos_use::{use_element_size, UseElementSizeReturn};

use crate::{
    consts::{DEFAULT_TRANSITION, FRAME_BACKDROP_FILTER, SMALL_SPACER_HEIGHT},
    page::{Content, Spacer},
    query_signal,
};

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
            end: Some(
                chrono::NaiveDate::from_ymd_opt(
                    $end_year,
                    $end_month,
                    match $end_month {
                        2 => 28,
                        4 | 6 | 9 | 11 => 30,
                        _ => 31,
                    },
                )
                .unwrap(),
            ),
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
        "Student Ambassador",
        "University of Hong Kong",
        CommunityService,
        2023,
        10,
    },
    experience! {
        "University Undergraduate Student",
        "University of Hong Kong Bachelor of Engineering in Computer Science",
        Education,
        2021,
        9,
        2025,
        6,
    },
    experience! {
        "Simple Reinforcement Learning Driver Workshop Instructor",
        "St. Paul's College",
        ContractWork,
        2025,
        2,
        2025,
        3,
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
        "Student Teaching Assistant",
        "University of Hong Kong Computer Science Department",
        ContractWork,
        2022,
        9,
        2023,
        11,
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
        "Programming Team Tutor",
        "St. Paul's College",
        ContractWork,
        2023,
        3,
        2023,
        4,
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
        "HKDSE Secondary School Student",
        "St. Paul's College",
        Education,
        2015,
        9,
        2021,
        7,
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
        "IGCSE Spanish Student",
        "St. Paul's College joint with HKU School of Modern Languages and Cultures",
        Education,
        2015,
        9,
        2019,
        6,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Type {
    ProfessionalWork,
    ContractWork,
    CommunityService,
    Education,
}

impl Type {
    pub const fn name(self) -> &'static str {
        match self {
            Type::ProfessionalWork => "Professional Work",
            Type::ContractWork => "Contract Work",
            Type::CommunityService => "Community Service",
            Type::Education => "Education",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Type::ProfessionalWork => "ph-briefcase",
            Type::ContractWork => "ph-hammer",
            Type::CommunityService => "ph-users-three",
            Type::Education => "ph-graduation-cap",
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    struct TypeFlags: u32 {
        const PROFESSIONAL_WORK = 1 << 0;
        const CONTRACT_WORK     = 1 << 1;
        const COMMUNITY_SERVICE = 1 << 2;
        const EDUCATION         = 1 << 3;
    }
}

impl TypeFlags {
    pub fn name(self) -> &'static str {
        match self {
            Self::PROFESSIONAL_WORK => "Professional Work",
            Self::CONTRACT_WORK => "Contract Work",
            Self::COMMUNITY_SERVICE => "Community Service",
            Self::EDUCATION => "Education",
            _ => "Unknown",
        }
    }
}

impl From<Type> for TypeFlags {
    fn from(ty: Type) -> Self {
        match ty {
            Type::ProfessionalWork => TypeFlags::PROFESSIONAL_WORK,
            Type::ContractWork => TypeFlags::CONTRACT_WORK,
            Type::CommunityService => TypeFlags::COMMUNITY_SERVICE,
            Type::Education => TypeFlags::EDUCATION,
        }
    }
}

impl FromStr for TypeFlags {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(TypeFlags::from_bits_truncate)
    }
}

impl Display for TypeFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bits().fmt(f)
    }
}

impl Default for TypeFlags {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Sort {
    #[default]
    EndDate,
    StartDate,
    Duration,
}

impl Sort {
    pub fn name(self) -> &'static str {
        match self {
            Sort::EndDate => "End Date",
            Sort::StartDate => "Start Date",
            Sort::Duration => "Duration",
        }
    }
}

impl FromStr for Sort {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "end_date" => Ok(Sort::EndDate),
            "start_date" => Ok(Sort::StartDate),
            "duration" => Ok(Sort::Duration),
            _ => Err(()),
        }
    }
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::EndDate => write!(f, "end_date"),
            Sort::StartDate => write!(f, "start_date"),
            Sort::Duration => write!(f, "duration"),
        }
    }
}

fn get_experiences(ty: TypeFlags, sort: Sort, asc: bool) -> Vec<usize> {
    let mut experiences = EXPERIENCES
        .iter()
        .enumerate()
        .filter_map(move |(i, experience)| ty.contains(experience.ty.into()).then_some(i))
        .collect_vec();

    experiences.sort_by(|a, b| {
        let a = &EXPERIENCES[*a];
        let b = &EXPERIENCES[*b];

        match sort {
            Sort::EndDate => a.end.cmp(&b.end),
            Sort::StartDate => a.start.cmp(&b.start),
            Sort::Duration => duration(a.start, a.end).cmp(&duration(b.start, b.end)),
        }
    });

    if !asc {
        experiences.reverse();
    }

    experiences
}

#[component]
pub fn Experiences() -> impl IntoView {
    let (ty, _) = query_signal::<TypeFlags>("type");
    let (sort, _) = query_signal::<Sort>("sort");
    let (asc, _) = query_signal::<bool>("asc");
    let experiences = Memo::new(move |_| {
        get_experiences(
            ty.get().unwrap_or_default(),
            sort.get().unwrap_or_default(),
            asc.get().unwrap_or_default(),
        )
    });

    view! {
        <Content
            icon="ph-handshake"
            title="Experiences"
            subtitle="Journey of professional work and education."
        >
            <div style="height: 2.4rem;"/>
            <div style="padding-left: 0.4em">
                <Tools/>
            </div>
            <Spacer/>
            {move || experiences.with(|experiences| Itertools::intersperse_with(
                experiences.iter().map(|i| {
                    let experience = &EXPERIENCES[*i];
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
            ).collect_view())}
        </Content>
    }
}

#[component]
fn Tools() -> impl IntoView {
    let ty_label_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: ty_label_width,
        ..
    } = use_element_size(ty_label_ref);
    let sort_label_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: sort_label_width,
        ..
    } = use_element_size(sort_label_ref);
    let asc_label_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: asc_label_width,
        ..
    } = use_element_size(asc_label_ref);
    let (ty, set_ty) = query_signal::<TypeFlags>("type");
    let (sort, set_sort) = query_signal::<Sort>("sort");
    let (asc, set_asc) = query_signal::<bool>("asc");

    let get_ty = move || ty.get().unwrap_or_default();

    let set_ty = move |ty: TypeFlags| {
        if get_ty() == ty {
            return;
        }

        set_ty.set((ty != TypeFlags::default()).then_some(ty));
    };

    let toggle_ty = move |ty: TypeFlags| {
        set_ty(get_ty() ^ ty);
    };

    let set_sort = move |sort: Sort| {
        set_sort.set((sort != Sort::default()).then_some(sort));
    };

    let toggle_asc = move || {
        set_asc.set((!asc.get().unwrap_or_default()).then_some(true));
    };

    view! {
        <div style="position: relative;">
            <div style=move || {
                let width = ty_label_width.get();
                format!("\
                    display: flex; \
                    flex-direction: column; \
                    gap: 0.4rem; \
                    padding: 1rem 0.8rem; \
                    border: 1px solid; \
                    backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                    transition: {DEFAULT_TRANSITION}; \
                    mask-image: linear-gradient( \
                        to top, \
                        black 0% calc(100% - 1px), \
                        transparent calc(100% - 1px) 100% \
                    ), linear-gradient( \
                        to left, \
                        black 0% calc(100% - 1.5rem - {width}px), \
                        transparent calc(100% - 1.5rem - {width}px) 100% \
                    ), linear-gradient( \
                        to right, \
                        black 0% 0.5rem, \
                        transparent 0.5rem 100% \
                    ) \
                ")
            }>
                <div style="display: flex; flex-direction: column; gap: 0.4rem;">
                    <div style="display: flex; flex-direction: row; gap: 0.4rem;">
                        <button class="lowercase" on:click=move |_| set_ty(TypeFlags::all())>
                            {move || view! {
                                <i class={match get_ty() == TypeFlags::all() {
                                    true => "ph-fill ph-square",
                                    false => "ph ph-square",
                                }}/>
                            }}
                            all
                        </button>
                        <button class="lowercase" on:click=move |_| set_ty(TypeFlags::empty())>
                            {move || view! {
                                <i class={match get_ty() == TypeFlags::empty() {
                                    true => "ph-fill ph-square",
                                    false => "ph ph-square",
                                }}/>
                            }}
                            none
                        </button>
                    </div>
                    <div style="display: flex; flex-direction: row; gap: 0.4rem;">
                        {[
                            TypeFlags::PROFESSIONAL_WORK,
                            TypeFlags::CONTRACT_WORK,
                            TypeFlags::COMMUNITY_SERVICE,
                            TypeFlags::EDUCATION,
                        ]
                            .into_iter()
                            .map(|ty| view! {
                                <button class="lowercase" on:click=move |_| toggle_ty(ty)>
                                    {move || view! {
                                        <i class={match get_ty().contains(ty) {
                                            true => "ph-fill ph-square",
                                            false => "ph ph-square",
                                        }}/>
                                    }}
                                    {ty.name().replace(" ", "_")}
                                </button>
                            })
                            .collect_view()
                        }
                    </div>
                </div>
            </div>
            <p
                node_ref=ty_label_ref
                style="\
                    position: absolute; \
                    left: 1rem; \
                    top: -0.4rem; \
                    font-size: 0.8rem; \
                    max-width: 19rem; \
                    white-space: nowrap; \
                    overflow: hidden; \
                    text-overflow: ellipsis; \
                "
            >
                {move || format!(
                    "type={}",
                    match ty.get() {
                        Some(ty) => leptos_router::location::Url::escape(&ty.to_string()),
                        None => String::new(),
                    }
                )}
            </p>
        </div>
        <Spacer small=true/>
        <div style=format!("display: flex; flex-direction: row; gap: {SMALL_SPACER_HEIGHT};")>
            <div style="position: relative;">
                <div style=move || {
                    let width = sort_label_width.get();
                    format!("\
                        padding: 1rem 0.8rem; \
                        width: fit-content; \
                        border: 1px solid; \
                        backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                        transition: {DEFAULT_TRANSITION}; \
                        mask-image: linear-gradient( \
                            to top, \
                            black 0% calc(100% - 1px), \
                            transparent calc(100% - 1px) 100% \
                        ), linear-gradient( \
                            to left, \
                            black 0% calc(100% - 1.5rem - {width}px), \
                            transparent calc(100% - 1.5rem - {width}px) 100% \
                        ), linear-gradient( \
                            to right, \
                            black 0% 0.5rem, \
                            transparent 0.5rem 100% \
                        ) \
                    ")
                }>
                    <div style=format!("display: flex; flex-direction: row; gap: 0.4rem;")>
                        {[
                            Sort::EndDate,
                            Sort::StartDate,
                            Sort::Duration,
                        ]
                            .into_iter()
                            .map(|curr_sort| view! {
                                <button class="lowercase" on:click=move |_| set_sort(curr_sort)>
                                    {move || view! {
                                        <i class={match sort.get().unwrap_or_default() == curr_sort {
                                            true => "ph-fill ph-square",
                                            false => "ph ph-square",
                                        }}/>
                                    }}
                                    {curr_sort.name().replace(" ", "_")}
                                </button>
                            })
                            .collect_view()
                        }
                    </div>
                </div>
                <p
                    node_ref=sort_label_ref
                    style="\
                        position: absolute; \
                        left: 1rem; \
                        top: -0.4rem; \
                        font-size: 0.8rem; \
                        max-width: 19rem; \
                        white-space: nowrap; \
                        overflow: hidden; \
                        text-overflow: ellipsis; \
                    "
                >
                    {move || format!(
                        "sort={}",
                        match sort.get() {
                            Some(sort) => leptos_router::location::Url::escape(&sort.to_string()),
                            None => String::new(),
                        }
                    )}
                </p>
            </div>
            <div style="position: relative;">
                <div style=move || {
                    let width = asc_label_width.get();
                    format!("\
                        padding: 1rem 0.8rem; \
                        width: fit-content; \
                        border: 1px solid; \
                        backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                        transition: {DEFAULT_TRANSITION}; \
                        mask-image: linear-gradient( \
                            to top, \
                            black 0% calc(100% - 1px), \
                            transparent calc(100% - 1px) 100% \
                        ), linear-gradient( \
                            to left, \
                            black 0% calc(100% - 1.5rem - {width}px), \
                            transparent calc(100% - 1.5rem - {width}px) 100% \
                        ), linear-gradient( \
                            to right, \
                            black 0% 0.5rem, \
                            transparent 0.5rem 100% \
                        ) \
                    ")
                }>
                    <button class="lowercase" style="width: fit-content" on:click=move |_| toggle_asc()>
                        {move || view! {
                            <i class={match asc.get().unwrap_or_default() {
                                true => "ph-fill ph-square",
                                false => "ph ph-square",
                            }}/>
                        }}
                        true
                    </button>
                </div>
                <p
                    node_ref=asc_label_ref
                    style="\
                        position: absolute; \
                        left: 1rem; \
                        top: -0.4rem; \
                        font-size: 0.8rem; \
                        max-width: 19rem; \
                        white-space: nowrap; \
                        overflow: hidden; \
                        text-overflow: ellipsis; \
                    "
                >
                    {move || format!(
                        "asc={}",
                        match asc.get() {
                            Some(asc) => leptos_router::location::Url::escape(&asc.to_string()),
                            None => String::new(),
                        }
                    )}
                </p>
            </div>
        </div>
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
            padding-left: 0.2em; \
            margin-bottom: 0.4em; \
        ">
            <h3><i class=format!("ph {}", ty.icon())/>" "{position}</h3>
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

fn duration(start: chrono::NaiveDate, end: Option<chrono::NaiveDate>) -> Duration {
    let end_date = end.unwrap_or_else(|| chrono::Local::now().naive_local().date());
    end_date.signed_duration_since(start)
}

fn duration_string(start: chrono::NaiveDate, end: Option<chrono::NaiveDate>) -> String {
    let duration = duration(start, end);
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
