const BADGE_BLUE_NO_UL: &str = "badge-blue no-underline";
const ITALIC_XS_CLASS: &str = "text-xs text-gray-400 italic";
use crate::i18n::t;
use crate::i18n::use_i18n;
use crate::models::Football;
use leptos::prelude::*;

const BADGE_GRAY: &str = "badge-gray";
const BADGE_GREEN: &str = "badge-green";
const BADGE_RED: &str = "badge-red";
const ITALIC: &str = "italic";

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
    let f = football.clone();
    let cat_name = f
        .category
        .as_ref()
        .map(|c| c.name_en.clone())
        .unwrap_or_default();
    let card_class = format!(
        "card p-4 hover:shadow-md transition-shadow {}",
        status_class(f.status)
    );

    view! {
        <div class=card_class>
            // ── Header: teams ────────────────────────────────────────────────
            <div class="flex items-center justify-between mb-2">
                <a href=format!("/footballs/{}", f.id) class="font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 no-underline text-sm leading-tight">
                    {f.home_team.clone()} " vs " {f.away_team.clone()}
                </a>
                <span class="text-xs text-gray-400 ml-2 whitespace-nowrap">{status_badge(f.status)}</span>
            </div>

            // ── Meta: season, category, kick-off ────────────────────────────
            <div class="text-xs text-gray-500 dark:text-gray-400 mb-3 space-x-2">
                <span>{f.season.clone()}</span>
                {if !cat_name.is_empty() {
                    view! { <span class=BADGE_GRAY>{cat_name}</span> }.into_any()
                } else { view! { <span/> }.into_any() }}
                <span class="text-blue-500">{f.kick_off_at_mdhm8.clone()}</span>
            </div>

            // ── Odds ─────────────────────────────────────────────────────────
            {if f.il_odds.is_empty() {
                view! {
                    <p class=format!("text-xs text-gray-400 {} mb-2", ITALIC)>{move || t!(i18n, not_pred)}</p>
                }.into_any()
            } else {
                let init_odds = f.il_odds.first().cloned();
                let last_odds = f.il_odds.last().cloned();
                view! {
                    <div class="text-xs space-y-1 mb-2">
                        {init_odds.map(|o| view! {
                            <div class="flex items-center gap-2">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_odds)}</span>
                                <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win.clone()}</span>
                                <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw.clone()}</span>
                                <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss.clone()}</span>
                            </div>
                        })}
                        {last_odds.and_then(|o| {
                            if f.il_odds.len() > 1 { Some(view! {
                                <div class="flex items-center gap-2">
                                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_odds)}</span>
                                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {o.win.clone()}</span>
                                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {o.draw.clone()}</span>
                                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {o.loss.clone()}</span>
                                </div>
                            }) } else { None }
                        })}
                    </div>
                }.into_any()
            }}

            // ── Predictions ──────────────────────────────────────────────────
            {if f.il_pred_over.is_empty() {
                view! { <span/> }.into_any()
            } else {
                let init_pred = f.il_pred_over.first().cloned();
                let last_pred = f.il_pred_over.last().cloned();
                view! {
                    <div class="text-xs space-y-1 mb-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                        {init_pred.map(|c| view! {
                            <div class="flex items-center gap-2 flex-wrap">
                                <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_init_pred)}</span>
                                <span class="text-gray-600 dark:text-gray-300">
                                    {move || t!(i18n, football_s)} ": " {c.s.clone()}
                                    " | " {move || t!(i18n, football_wdl)} ": " {c.wdl.clone()}
                                    " | " {move || t!(i18n, football_tg)} ": " {c.tg.clone()}
                                    " | " {move || t!(i18n, football_gd)} ": " {c.gd.clone()}
                                </span>
                            </div>
                        })}
                        {last_pred.and_then(|c| {
                            if f.il_pred_over.len() > 1 { Some(view! {
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class="text-gray-400 w-20 shrink-0">{move || t!(i18n, football_last_pred)}</span>
                                    <span class="text-gray-600 dark:text-gray-300">
                                        {move || t!(i18n, football_s)} ": " {c.s.clone()}
                                        " | " {move || t!(i18n, football_wdl)} ": " {c.wdl.clone()}
                                        " | " {move || t!(i18n, football_tg)} ": " {c.tg.clone()}
                                        " | " {move || t!(i18n, football_gd)} ": " {c.gd.clone()}
                                    </span>
                                </div>
                            }) } else { None }
                        })}
                    </div>
                }.into_any()
            }}

            // ── Official result ───────────────────────────────────────────────
            {match f.football_over.clone() {
                None => view! {
                    <p class=ITALIC_XS_CLASS>{move || t!(i18n, not_over)}</p>
                }.into_any(),
                Some(ov) => view! {
                    <div class="text-xs flex items-center gap-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                        <span class="text-gray-400">{move || t!(i18n, football_over)}</span>
                        <span class="font-semibold text-blue-700 dark:text-blue-300">
                            {move || t!(i18n, football_s)} ": " {ov.s.clone()}
                            " | " {move || t!(i18n, football_wdl)} ": " {ov.wdl.clone()}
                            " | " {move || t!(i18n, football_tg)} ": " {ov.tg.clone()}
                        </span>
                    </div>
                }.into_any(),
            }}

            // ── Topics & hits ─────────────────────────────────────────────────
            <div class="flex items-center justify-between mt-3">
                <div class="flex flex-wrap gap-1">
                    {f.topics.iter().map(|t| view! {
                        <a href=format!("/footballs?filter=topic&fid={}", t.id)
                                                   class=BADGE_BLUE_NO_UL
                                                >{t.name.clone()}</a>
                    }).collect::<Vec<_>>()}
                </div>
                <span class="text-xs text-gray-400">{move || t!(i18n, football_hits)} {f.hits}</span>
            </div>
        </div>
    }
}
