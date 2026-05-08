use crate::i18n::{t, use_i18n};
use crate::models::Football;
use crate::utils::constant::{BADGE_BLUE_NO_UL, BADGE_GRAY, BADGE_GREEN, BADGE_RED, ITALIC, ITALIC_XS};
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
        3 => "⭐ Rec",
        2 => "🔥 Hot",
        1 => "Published",
        0 => "Draft",
        _ => "—",
    }
}

#[component]
pub fn FootballCard(football: Football) -> impl IntoView {
    let i18n = use_i18n();
    let cat_name = football.category.as_ref().map(|c| c.name_en.clone()).unwrap_or_default();
    let card_class = format!("card p-4 hover:shadow-md transition-shadow {}", status_class(football.status));
    let init_odds = football.il_odds.first().cloned();
    let last_odds = football.il_odds.last().cloned();
    let init_pred = football.il_pred_over.first().cloned();
    let last_pred = football.il_pred_over.last().cloned();
    let football_over = football.football_over;
    let topics = football.topics;

    view! {
        <div class=card_class>
            <div class="flex items-center justify-between mb-2">
                <a href=format!("/footballs/{}", football.id) class="font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 no-underline text-sm leading-tight">
                    {football.home_team} " vs " {football.away_team}
                </a>
                <span class="text-xs text-gray-400 ml-2 whitespace-nowrap">{status_badge(football.status)}</span>
            </div>

            <div class="text-xs text-gray-500 dark:text-gray-400 mb-3 space-x-2">
                <span>{football.season}</span>
                {if !cat_name.is_empty() {
                    view! { <span class=BADGE_GRAY>{cat_name}</span> }.into_any()
                } else { ().into_any() }}
                <span class="text-blue-500">{football.kick_off_at_mdhm8}</span>
            </div>

            {if football.il_odds.is_empty() {
                view! { <p class=format!("text-xs text-gray-400 {} mb-2", ITALIC)>{move || t!(i18n, not_pred)}</p> }.into_any()
            } else {
                view! {
                    <div class="text-xs space-y-1 mb-2">
                        {init_odds.map(|o| view! {
                            <div class="flex items-center gap-2">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_odds)}</span>
                                <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win}</span>
                                <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw}</span>
                                <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss}</span>
                            </div>
                        })}
                        {last_odds.and_then(|o| if football.il_odds.len() > 1 { Some(view! {
                            <div class="flex items-center gap-2">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_odds)}</span>
                                <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win}</span>
                                <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw}</span>
                                <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss}</span>
                            </div>
                        }) } else { None })}
                    </div>
                }.into_any()
            }}

            {if football.il_pred_over.is_empty() {
                ().into_any()
            } else {
                view! {
                    <div class="text-xs space-y-1 mb-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                        {init_pred.map(|c| view! {
                            <div class="flex items-center gap-2 flex-wrap">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_pred)}</span>
                                <span class="text-gray-600 dark:text-gray-300">
                                    {move || t!(i18n, football_s)} ": " {c.s}
                                    " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                                    " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                                    " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                                </span>
                            </div>
                        })}
                        {last_pred.and_then(|c| if football.il_pred_over.len() > 1 { Some(view! {
                            <div class="flex items-center gap-2 flex-wrap">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_pred)}</span>
                                <span class="text-gray-600 dark:text-gray-300">
                                    {move || t!(i18n, football_s)} ": " {c.s}
                                    " | " {move || t!(i18n, football_wdl)} ": " {c.wdl}
                                    " | " {move || t!(i18n, football_tg)} ": " {c.tg}
                                    " | " {move || t!(i18n, football_gd)} ": " {c.gd}
                                </span>
                            </div>
                        }) } else { None })}
                    </div>
                }.into_any()
            }}

            {match football_over {
                None => view! { <p class=ITALIC_XS>{move || t!(i18n, not_over)}</p> }.into_any(),
                Some(ov) => view! {
                    <div class="text-xs flex items-center gap-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                        <span class="text-gray-400">{move || t!(i18n, football_over)}</span>
                        <span class="font-semibold text-blue-700 dark:text-blue-300">
                            {move || t!(i18n, football_s)} ": " {ov.s}
                            " | " {move || t!(i18n, football_wdl)} ": " {ov.wdl}
                            " | " {move || t!(i18n, football_tg)} ": " {ov.tg}
                        </span>
                    </div>
                }.into_any(),
            }}

            <div class="flex items-center justify-between mt-3">
                <div class="flex flex-wrap gap-1">
                    {topics.iter().map(|t| view! {
                        <a href=format!("/footballs?filter=topic&fid={}", t.id) class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                    }).collect::<Vec<_>>()}
                </div>
                <span class="text-xs text-gray-400">{move || t!(i18n, football_hits)} {football.hits}</span>
            </div>
        </div>
    }
}
