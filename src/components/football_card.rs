use crate::i18n::{Locale, t, use_i18n};
use crate::models::Football;
use crate::utils::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, BADGE_GREEN, BADGE_RED, FLEX_BETWEEN, HOVER_SHADOW, ITALIC,
    ITALIC_XS, TEXT_MUTED, TEXT_SUBTLE, TEXT_XS_MUTED,
};
use leptos::either::Either;
use leptos::prelude::*;

fn status_class(status: i8) -> &'static str {
    match status {
        4 => "fc-status-4",
        3 => "fc-status-3",
        2 => "fc-status-2",
        1 => "fc-status-1",
        _ => "fc-status-0",
    }
}

fn status_badge(status: i8) -> &'static str {
    match status {
        4 => "⭐🔥",
        3 => "⭐ Pick",
        2 => "🔥 Hot",
        1 => "Published",
        0 => "Draft",
        _ => "—",
    }
}

#[component]
fn CatBadge(#[prop(into)] name: Signal<String>) -> impl IntoView {
    move || {
        let n = name.get();
        if n.is_empty() {
            Either::Left(())
        } else {
            Either::Right(view! { <span class=BADGE_GRAY>{n}</span> })
        }
    }
}

#[component]
fn OddsSection(odds: Vec<crate::models::FootballLine>) -> impl IntoView {
    let i18n = use_i18n();
    if odds.is_empty() {
        return Either::Left(view! {
            <p class=format!("{} {} mb-2", TEXT_XS_MUTED, ITALIC)>
                {move || t!(i18n, not_calc)}
            </p>
        });
    }
    let init = odds.first().cloned();
    let last = odds.last().cloned();
    Either::Right(view! {
        <div class="text-xs space-y-1 mb-2">
            {init.map(|o| view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss}</span>
                </div>
            })}
            {last.and_then(|o| if odds.len() > 1 { Some(view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss}</span>
                </div>
            }) } else { None })}
        </div>
    })
}

#[component]
fn CalcsSection(calcs: Vec<crate::models::FootballOver>) -> impl IntoView {
    let i18n = use_i18n();
    if calcs.is_empty() {
        return Either::Left(());
    }
    let init = calcs.first().cloned();
    let last = calcs.last().cloned();
    Either::Right(view! {
        <div class="text-xs space-y-1 mb-2 border-t border-gray-100 dark:border-gray-700 pt-2">
            {init.map(|c| view! {
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_calc)}</span>
                    <span class=TEXT_MUTED>
                        {move || t!(i18n, football_s)} ": " {c.s}
                        " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                        " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                        " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                    </span>
                </div>
            })}
            {last.and_then(|c| if calcs.len() > 1 { Some(view! {
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_calc)}</span>
                    <span class=TEXT_MUTED>
                        {move || t!(i18n, football_s)} ": " {c.s}
                        " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                        " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                        " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                    </span>
                </div>
            }) } else { None })}
        </div>
    })
}

#[component]
fn OverSection(over: Option<crate::models::FootballOver>) -> impl IntoView {
    let i18n = use_i18n();
    match over {
        None => Either::Left(view! {
            <p class=ITALIC_XS>{move || t!(i18n, not_full)}</p>
        }),
        Some(ov) => Either::Right(view! {
            <div class="text-xs flex items-center gap-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                <span class="text-gray-400">{move || t!(i18n, football_over)}</span>
                <span class="font-semibold text-blue-700 dark:text-blue-300">
                    {move || t!(i18n, football_s)} ": " {ov.s}
                    " | " {move || t!(i18n, football_wdl)} ": " {ov.wdl}
                    " | " {move || t!(i18n, football_tg)} ": " {ov.tg}
                </span>
            </div>
        }),
    }
}

#[component]
pub fn FootballCard(football: Football) -> impl IntoView {
    let i18n = use_i18n();
    let card_class = format!(
        "card p-4 {} {}",
        HOVER_SHADOW,
        status_class(football.status)
    );
    let cat_name = Memo::new(move |_| {
        football
            .category
            .as_ref()
            .map(|c| {
                if i18n.get_locale() == Locale::zh {
                    c.name_zh.clone()
                } else {
                    c.name_en.clone()
                }
            })
            .unwrap_or_default()
    });
    let topics = football.topics;

    view! {
        <div class=card_class>
            <div class=format!("{} mb-2", FLEX_BETWEEN)>
                <a href=format!("/footballs/{}", football.id) class="font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 no-underline text-base leading-tight">
                    {football.home_team} " vs " {football.away_team}
                </a>
                <span class="text-sm text-gray-400 ml-2 whitespace-nowrap">{status_badge(football.status)}</span>
            </div>

            <div class=format!("text-sm {} mb-3 space-x-2", TEXT_SUBTLE)>
                <span>{football.season}</span>
                <CatBadge name=cat_name/>
                <span class="text-blue-500">{football.kick_off_at_mdhm8}</span>
            </div>

            <OddsSection odds=football.il_odds/>
            <CalcsSection calcs=football.il_calc_over/>
            <OverSection over=football.football_over/>

            <div class=format!("{} mt-3", FLEX_BETWEEN)>
                <div class="flex flex-wrap gap-1">
                    {topics.iter().map(|t| view! {
                        <a href=format!("/footballs?filter=topic&fid={}", t.id) class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                    }).collect::<Vec<_>>()}
                </div>
                <span class="text-sm text-gray-400">{move || t!(i18n, football_hits)} {football.hits}</span>
            </div>
        </div>
    }
}
