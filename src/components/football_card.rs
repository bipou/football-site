use crate::i18n::{t, use_i18n};
use crate::models::Football;
use crate::shared::common::Either3;
use crate::shared::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, BADGE_GRAY_NO_UL, FLEX_BETWEEN, HOVER_SHADOW, NO_UNDERLINE,
    TEXT_SUBTLE,
};
#[cfg(feature = "oth")]
use crate::shared::constant::{BADGE_GREEN, BADGE_RED, TEXT_MUTED};
use crate::shared::locale::use_locale;
use leptos::either::Either;
use leptos::prelude::*;

pub(crate) fn status_class(status: i8) -> &'static str {
    match status {
        4 => "fc-status-4",
        3 => "fc-status-3",
        2 => "fc-status-2",
        _ => "",
    }
}

pub(crate) fn status_badge(status: i8) -> &'static str {
    match status {
        4 => "⭐🔥",
        3 => "⭐",
        2 => "🔥",
        _ => "",
    }
}

#[component]
pub(crate) fn CatBadge(
    #[prop(into)] name: Signal<Option<String>>,
    #[prop(into)] kid: Option<String>,
) -> impl IntoView {
    let loc_str = use_locale();
    move || {
        let n = name.get();
        if n.is_none() {
            Either3::Left(())
        } else if let Some(kid) = &kid {
            let href = ["/", &loc_str.get(), "/footballs/category/", kid].join("");
            Either3::Right(Either::Left(view! {
                <a href=href class=BADGE_GRAY_NO_UL>{n.unwrap_or_default()}</a>
            }))
        } else {
            Either3::Right(Either::Right(
                view! { <span class=BADGE_GRAY>{n.unwrap_or_default()}</span> },
            ))
        }
    }
}

#[component]
fn ResultSection(
    #[prop(into)] s: Option<String>,
    #[prop(into)] wdl: Option<u8>,
    #[prop(into)] tg: Option<u8>,
) -> impl IntoView {
    let i18n = use_i18n();
    match (s, wdl, tg) {
        (Some(s), Some(wdl), Some(tg)) => Either::Right(view! {
            <div class="text-xs flex items-center gap-2 border-t border-gray-100 dark:border-gray-700 pt-2">
                <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_result)}</span>
                <span class="font-semibold text-blue-700 dark:text-blue-300">
                    <span class="mr-4">{move || t!(i18n, football_s)} ": " {s}</span>
                    <span class="mr-4">{move || t!(i18n, football_wdl)} ": " {wdl}</span>
                    <span>{move || t!(i18n, football_tg)} ": " {tg}</span>
                </span>
            </div>
        }),
        _ => Either::Left(()),
    }
}

#[allow(unused_variables)]
fn render_card_extra(football: &Football) -> impl IntoView + use<> {
    #[cfg(feature = "oth")]
    {
        let odds = football.il_odds.clone();
        let calcs = football.il_calcs.clone();
        view! {
            <OddsSection odds=odds/>
            <CalcsSection calcs=calcs/>
        }
        .into_view()
    }
    #[cfg(not(feature = "oth"))]
    {
        ().into_view()
    }
}

// ── oth-only components ─────────────────────────────────────────────────────

#[cfg(feature = "oth")]
#[component]
fn OddsSection(odds: Vec<crate::models::Line>) -> impl IntoView {
    let i18n = use_i18n();
    if odds.is_empty() {
        return Either::Left(());
    }
    let init = odds.first().cloned();
    let last = odds.last().cloned();
    Either::Right(view! {
        <div class="text-xs space-y-1 mb-2">
            {init.map(|o| view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_init_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {format!("{:.2}", o.win)}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {format!("{:.2}", o.draw)}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {format!("{:.2}", o.loss)}</span>
                </div>
            })}
            {last.and_then(|o| if odds.len() > 1 { Some(view! {
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_last_odds)}</span>
                    <span class=BADGE_GREEN>{move || t!(i18n, football_win)} " " {format!("{:.2}", o.win)}</span>
                    <span class=BADGE_GRAY>{move || t!(i18n, football_draw)} " " {format!("{:.2}", o.draw)}</span>
                    <span class=BADGE_RED>{move || t!(i18n, football_loss)} " " {format!("{:.2}", o.loss)}</span>
                </div>
            }) } else { None })}
        </div>
    })
}

#[cfg(feature = "oth")]
#[component]
fn CalcsSection(calcs: Vec<crate::models::Calc>) -> impl IntoView {
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
                            <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_init_calc)}</span>
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
                            <span class="text-gray-400 w-16 shrink-0">{move || t!(i18n, football_last_calc)}</span>
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
pub fn FootballCard(football: Football, on_click: Callback<String>) -> impl IntoView {
    let i18n = use_i18n();
    let loc_str = use_locale();
    let extra = render_card_extra(&football);
    let card_class = [
        "card",
        "p-4",
        HOVER_SHADOW,
        status_class(football.status),
        "min-w-0",
    ]
    .join(" ");
    let title = football.title();
    let season = football.season;
    let article_title = football.article_title;
    let kick_off = football.kick_off_at_mdhm8;
    let status = football.status;
    let hits = football.hits;
    let topics = football.topics;
    let summary = football.summary.map(|s| {
        let has_cjk = s.chars().any(|c| c as u32 > 0x2000);
        let limit = if has_cjk { 45 } else { 100 };
        if s.chars().count() > limit {
            let mut t: String = s.chars().take(limit).collect();
            t.push_str("...");
            t
        } else {
            s
        }
    });
    let fid = crate::shared::common::record_key(&football.id).to_string();
    let href = ["/", &i18n.get_locale().to_string(), "/footballs/", &fid].join("");
    // 卡片点击回调：因 on:click 需闭包非 Callback，故每处内联
    let summary_view = summary.map(|s| {
        view! {
            <a class=["cursor-pointer block p-0 text-left w-full", NO_UNDERLINE].join(" ") href=href.clone() on:click={
                                        let f = fid.clone();
                                        let cb = on_click.clone();
                            move |ev| {
                                ev.prevent_default();
                                cb.run(f.clone())
                            }
                        }>
                <p class="text-sm text-gray-600 dark:text-gray-400 my-0">{s}</p>
            </a>
        }
    });
    let category_name = football.category_name;
    let cat_kid = category_name
        .as_ref()
        .map(|_| crate::shared::common::record_key(&football.category_id).to_string());
    let cat_name = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category_name.as_ref().and_then(|c| c.get(&loc).cloned())
    });

    view! {
        <div class=card_class>
            <div class=[FLEX_BETWEEN, "mb-2"].join(" ")>
                <a
                                    class=["font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 text-lg leading-tight min-w-0 cursor-pointer p-0 text-left block", NO_UNDERLINE].join(" ")
                                    href=href.clone()
                                    on:click={
                                                                        let f = fid.clone();
                                                                        let cb = on_click.clone();
                                                                        move |ev| {
                                                                            ev.prevent_default();
                                                                            cb.run(f.clone())
                                                                        }
                                                                    }
                                >
                                    {title}
                                </a>
                {let badge = status_badge(status); match badge.is_empty() {
                    false => Either::Left(view! {
                        <span class="text-sm ml-2 whitespace-nowrap">{badge}</span>
                    }),
                    true => Either::Right(()),
                }}
            </div>

            <div class=["text-sm", TEXT_SUBTLE, "mb-3", "space-x-2"].join(" ")>
                {match season {
                    Some(season) => Either::Left(view! { <span>{season}</span> }),
                    None => Either::Right(()),
                }}
                <CatBadge name=cat_name kid=cat_kid/>
                {match article_title {
                    Some(at) => Either::Left(view! { <span class=BADGE_GRAY>{at}</span> }),
                    None => Either::Right(()),
                }}
                {match kick_off {
                    Some(ko) => Either::Left(view! { <span class="text-blue-500">{ko}</span> }),
                    None => Either::Right(()),
                }}
            </div>

            {summary_view}
            <div class="mt-2">
            {extra}
            <ResultSection s=football.result_s wdl=football.result_wdl tg=football.result_tg/>
            </div>

            <div class="flex items-start justify-between mt-3">
                <div class="flex flex-wrap gap-1 min-w-0" style="max-width:calc(100% - 5rem)">
                    {topics.into_iter().map(|topic| {
                        let kid = crate::shared::common::record_key(&topic.id).to_string();
                        let name = topic.name;
                        let href = ["/", &loc_str.get(), "/footballs/topic/", &kid].join("");
                        view! {
                            <a href=href class=["text-sm", BADGE_BLUE_NO_UL].join(" ")>{name}</a>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <span class="text-sm text-gray-400">{move || t!(i18n, football_hits)}{hits}</span>
            </div>
        </div>
    }
}
