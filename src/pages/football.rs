use crate::i18n::{Locale, t, use_i18n};
use crate::site_title;
use crate::utils::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY, CARD_SECTION, EMPTY, FLEX_WRAP_GAP, ITALIC, MAIN, SECTION_H2,
    TEXT_XS_MUTED,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::models::Football;

#[server]
pub async fn get_football_and_increment(id: String) -> Result<Option<Football>, ServerFnError> {
    use crate::server::football_db;
    let _ = football_db::increment_hits(&id).await;
    football_db::get_football_by_id(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
fn MatchHeader(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let title_text = format!("{} vs {} – {}", f.home_team, f.away_team, site_title!(i18n));
    let cat = Memo::new(move |_| {
        f.category
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
    view! {
        <Title text=title_text/>
        <div class=CARD_SECTION>
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                        {f.home_team} <span class="text-gray-400 mx-2">"vs"</span> {f.away_team}
                    </h1>
                    <div class="text-sm text-gray-500 space-x-3">
                        <span>{move || t!(i18n, football_season)} " " {f.season}</span>
                        {move || if !cat.get().is_empty() {
                            Either::Left(view! { <span class=BADGE_GRAY>{cat.get()}</span> })
                        } else {
                            Either::Right(())
                        }}
                    </div>
                </div>
                <div class="text-right text-sm text-gray-500">
                    <div>{move || t!(i18n, football_kick_off)}</div>
                    <div class="font-semibold text-blue-600">{f.kick_off_at_mdhm8}</div>
                    <div class=TEXT_XS_MUTED>"UTC: " {f.kick_off_at_mdhm}</div>
                </div>
            </div>
            <div class=format!("mt-3 {} flex gap-4 flex-wrap", TEXT_XS_MUTED)>
                <span>{move || t!(i18n, football_created)} ": " {f.created_at}</span>
                <span>{move || t!(i18n, football_updated)} ": " {f.updated_at}</span>
                <span>{move || t!(i18n, football_hits)} {f.hits}</span>
            </div>
        </div>
    }
}

#[component]
fn OddsTable(odds: Vec<crate::models::FootballLine>) -> impl IntoView {
    let i18n = use_i18n();
    if odds.is_empty() {
        return Either::Left(view! {
            <div class=CARD_SECTION>
                <p class=format!("text-gray-400 text-sm {}", ITALIC)>
                    {move || t!(i18n, not_calc)}
                </p>
            </div>
        });
    }
    let init = odds.first().cloned();
    let last = odds.last().cloned();
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>"Odds"</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500 dark:text-gray-400">
                        <tr>
                            <th class="px-4 py-2">"Kind"</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_win)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_draw)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_loss)}</th>
                            <th class="px-4 py-2">"Time"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {init.map(|o| view! {
                            <tr class="table-row">
                                <td class="px-4 py-2 text-gray-500">{move || t!(i18n, football_init_odds)}</td>
                                <td class="px-4 py-2 font-medium text-green-600">{o.win}</td>
                                <td class="px-4 py-2 font-medium text-gray-600">{o.draw}</td>
                                <td class="px-4 py-2 font-medium text-red-600">{o.loss}</td>
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{o.created_at}</td>
                            </tr>
                        })}
                        {last.and_then(|o| if odds.len() > 1 { Some(view! {
                            <tr class="table-row">
                                <td class="px-4 py-2 text-gray-500">{move || t!(i18n, football_last_odds)}</td>
                                <td class="px-4 py-2 font-medium text-green-600">{o.win}</td>
                                <td class="px-4 py-2 font-medium text-gray-600">{o.draw}</td>
                                <td class="px-4 py-2 font-medium text-red-600">{o.loss}</td>
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{o.created_at}</td>
                            </tr>
                        }) } else { None })}
                    </tbody>
                </table>
            </div>
        </div>
    })
}

#[component]
fn CalcsTable(calcs: Vec<crate::models::FootballOver>) -> impl IntoView {
    let i18n = use_i18n();
    if calcs.is_empty() {
        return Either::Left(view! {
            <div class=CARD_SECTION>
                <p class=format!("text-gray-400 text-sm {}", ITALIC)>
                    {move || t!(i18n, not_calc)}
                </p>
            </div>
        });
    }
    let init = calcs.first().cloned();
    let last = calcs.last().cloned();
    Either::Right(view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, calc)}</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500">
                        <tr>
                            <th class="px-4 py-2">"Kind"</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_s)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_wdl)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_tg)}</th>
                            <th class="px-4 py-2">{move || t!(i18n, football_gd)}</th>
                            <th class="px-4 py-2">"Time"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {init.map(|c| view! {
                            <tr class="table-row">
                                <td class="px-4 py-2 text-gray-500">"Initial"</td>
                                <td class="px-4 py-2 font-medium">{c.s}</td>
                                <td class="px-4 py-2">{c.wdl}</td>
                                <td class="px-4 py-2">{c.tg}</td>
                                <td class="px-4 py-2">{c.gd}</td>
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{c.created_at}</td>
                            </tr>
                        })}
                        {last.and_then(|c| if calcs.len() > 1 { Some(view! {
                            <tr class="table-row">
                                <td class="px-4 py-2 text-gray-500">"Latest"</td>
                                <td class="px-4 py-2 font-medium">{c.s}</td>
                                <td class="px-4 py-2">{c.wdl}</td>
                                <td class="px-4 py-2">{c.tg}</td>
                                <td class="px-4 py-2">{c.gd}</td>
                                <td class=format!("px-4 py-2 {}", TEXT_XS_MUTED)>{c.created_at}</td>
                            </tr>
                        }) } else { None })}
                    </tbody>
                </table>
            </div>
        </div>
    })
}

#[component]
fn OverDetail(football_over: Option<crate::models::FootballOver>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class=CARD_SECTION>
            <h2 class=SECTION_H2>{move || t!(i18n, football_over)}</h2>
            {match football_over {
                None => Either::Left(view! {
                    <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_full)}</p>
                }),
                Some(ov) => Either::Right(view! {
                    <div class="flex gap-6 flex-wrap text-sm">
                        <div><span class="text-gray-500">{move || t!(i18n, football_s)} ": "</span>
                            <span class="font-bold text-lg text-blue-700 dark:text-blue-300">{ov.s}</span></div>
                        <div><span class="text-gray-500">{move || t!(i18n, football_wdl)} ": "</span>
                            <span class="font-semibold">{ov.wdl}</span></div>
                        <div><span class="text-gray-500">{move || t!(i18n, football_tg)} ": "</span>
                            <span class="font-semibold">{ov.tg}</span></div>
                    </div>
                }),
            }}
        </div>
    }
}

#[component]
fn DetailTopicsSection(topics: Vec<crate::models::Topic>) -> impl IntoView {
    let i18n = use_i18n();
    if topics.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-4 mb-6">
                <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, football_keys_tags)}</p>
                <div class=FLEX_WRAP_GAP>
                    {topics.iter().map(|t| view! {
                        <a href=format!("/footballs?filter=topic&fid={}", t.id) class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        })
    }
}

#[component]
fn FootballDetail(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let header_f = f.clone();
    let odds = f.il_odds.clone();
    let calcs = f.il_calc_over.clone();
    let topics = f.topics;
    let football_over = f.football_over;
    view! {
        <MatchHeader f=header_f/>
        <OddsTable odds=odds/>
        <CalcsTable calcs=calcs/>
        <OverDetail football_over=football_over/>
        <DetailTopicsSection topics=topics/>
        <p class="text-xs text-red-400 text-center mt-4">{move || t!(i18n, site_warn)}</p>
    }
}

// ── Three-way view type alias ───────────────────────────────────────────
type DetailResult<A, B, C> = Either<A, Either<B, C>>;

#[component]
pub fn FootballDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let data = Resource::new_blocking(
        move || id(),
        |id| async move { get_football_and_increment(id).await },
    );

    view! {
        <Nav/>
        <main class=MAIN>
            <Suspense fallback=move || view! {
                <div class=format!("{} text-gray-400", EMPTY)>
                    {move || t!(i18n, loading)}
                </div>
            }>
                {move || data.get().map(|result| match result {
                    Err(e) => DetailResult::Left(view! {
                        <p class="text-red-500 text-center py-8">{e.to_string()}</p>
                    }),
                    Ok(None) => DetailResult::Right(Either::Left(view! {
                        <div class=EMPTY>
                            <p class="text-gray-500">"Football match not found."</p>
                            <a href="/footballs" class="btn-primary mt-4 inline-block">"Back to list"</a>
                        </div>
                    })),
                    Ok(Some(f)) => DetailResult::Right(Either::Right(view! { <FootballDetail f=f/> })),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
