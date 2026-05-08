use crate::i18n::{t, t_string, use_i18n};
use crate::site_title;
use crate::utils::constant::{BADGE_BLUE_NO_UL, BADGE_GRAY, ITALIC};
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
    let cat = f
        .category
        .as_ref()
        .map(|c| c.name_en.clone())
        .unwrap_or_default();
    view! {
        <Title text=title_text/>
        <div class="card p-6 mb-6">
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                        {f.home_team} <span class="text-gray-400 mx-2">"vs"</span> {f.away_team}
                    </h1>
                    <div class="text-sm text-gray-500 space-x-3">
                        <span>{move || t!(i18n, football_season)} " " {f.season}</span>
                        {if !cat.is_empty() {
                            view! { <span class=BADGE_GRAY>{cat}</span> }.into_any()
                        } else { ().into_any() }}
                    </div>
                </div>
                <div class="text-right text-sm text-gray-500">
                    <div>{move || t!(i18n, football_kick_off)}</div>
                    <div class="font-semibold text-blue-600">{f.kick_off_at_mdhm8}</div>
                    <div class="text-xs text-gray-400">"UTC: " {f.kick_off_at_mdhm}</div>
                </div>
            </div>
            <div class="mt-3 text-xs text-gray-400 flex gap-4 flex-wrap">
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
        let msg = t_string!(i18n, not_pred);
        return view! { <div class="card p-6 mb-6"><p class=format!("text-gray-400 text-sm {}", ITALIC)>{msg}</p></div> }.into_any();
    }
    let init = odds.first().cloned();
    let last = odds.last().cloned();
    let init_label = t_string!(i18n, football_init_odds);
    let last_label = t_string!(i18n, football_last_odds);
    let win_label = t_string!(i18n, football_win);
    let draw_label = t_string!(i18n, football_draw);
    let loss_label = t_string!(i18n, football_loss);
    view! {
        <div class="card p-6 mb-6">
            <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">"Odds"</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500 dark:text-gray-400">
                        <tr>
                            <th class="px-4 py-2">"Kind"</th>
                            <th class="px-4 py-2">{win_label}</th>
                            <th class="px-4 py-2">{draw_label}</th>
                            <th class="px-4 py-2">{loss_label}</th>
                            <th class="px-4 py-2">"Time"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {init.map(|o| view! {
                            <tr class="border-b border-gray-100 dark:border-gray-700">
                                <td class="px-4 py-2 text-gray-500">{&*init_label}</td>
                                <td class="px-4 py-2 font-medium text-green-600">{o.win}</td>
                                <td class="px-4 py-2 font-medium text-gray-600">{o.draw}</td>
                                <td class="px-4 py-2 font-medium text-red-600">{o.loss}</td>
                                <td class="px-4 py-2 text-xs text-gray-400">{o.created_at}</td>
                            </tr>
                        })}
                        {last.and_then(|o| if odds.len() > 1 { Some(view! {
                            <tr class="border-b border-gray-100 dark:border-gray-700">
                                <td class="px-4 py-2 text-gray-500">{&*last_label}</td>
                                <td class="px-4 py-2 font-medium text-green-600">{o.win}</td>
                                <td class="px-4 py-2 font-medium text-gray-600">{o.draw}</td>
                                <td class="px-4 py-2 font-medium text-red-600">{o.loss}</td>
                                <td class="px-4 py-2 text-xs text-gray-400">{o.created_at}</td>
                            </tr>
                        }) } else { None })}
                    </tbody>
                </table>
            </div>
        </div>
    }.into_any()
}

#[component]
fn PredictionsTable(calcs: Vec<crate::models::FootballOver>) -> impl IntoView {
    let i18n = use_i18n();
    if calcs.is_empty() {
        let msg = t_string!(i18n, not_pred);
        return view! { <div class="card p-6 mb-6"><p class=format!("text-gray-400 text-sm {}", ITALIC)>{msg}</p></div> }.into_any();
    }
    let init = calcs.first().cloned();
    let last = calcs.last().cloned();
    let s_label = t_string!(i18n, football_s);
    let wdl_label = t_string!(i18n, football_wdl);
    let tg_label = t_string!(i18n, football_tg);
    let gd_label = t_string!(i18n, football_gd);
    view! {
        <div class="card p-6 mb-6">
            <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">"Predictions"</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead class="bg-gray-50 dark:bg-gray-700 text-xs text-gray-500">
                        <tr>
                            <th class="px-4 py-2">"Kind"</th>
                            <th class="px-4 py-2">{s_label}</th>
                            <th class="px-4 py-2">{wdl_label}</th>
                            <th class="px-4 py-2">{tg_label}</th>
                            <th class="px-4 py-2">{gd_label}</th>
                            <th class="px-4 py-2">"Time"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {init.map(|c| view! {
                            <tr class="border-b border-gray-100 dark:border-gray-700">
                                <td class="px-4 py-2 text-gray-500">"Initial"</td>
                                <td class="px-4 py-2 font-medium">{c.s}</td>
                                <td class="px-4 py-2">{c.wdl}</td>
                                <td class="px-4 py-2">{c.tg}</td>
                                <td class="px-4 py-2">{c.gd}</td>
                                <td class="px-4 py-2 text-xs text-gray-400">{c.created_at}</td>
                            </tr>
                        })}
                        {last.and_then(|c| if calcs.len() > 1 { Some(view! {
                            <tr class="border-b border-gray-100 dark:border-gray-700">
                                <td class="px-4 py-2 text-gray-500">"Latest"</td>
                                <td class="px-4 py-2 font-medium">{c.s}</td>
                                <td class="px-4 py-2">{c.wdl}</td>
                                <td class="px-4 py-2">{c.tg}</td>
                                <td class="px-4 py-2">{c.gd}</td>
                                <td class="px-4 py-2 text-xs text-gray-400">{c.created_at}</td>
                            </tr>
                        }) } else { None })}
                    </tbody>
                </table>
            </div>
        </div>
    }.into_any()
}

#[component]
fn FootballDetail(f: Football) -> impl IntoView {
    let i18n = use_i18n();
    let header_f = f.clone();
    let odds = f.il_odds.clone();
    let calcs = f.il_pred_over.clone();
    let over_label = t_string!(i18n, football_over);
    let s_label = t_string!(i18n, football_s);
    let wdl_label = t_string!(i18n, football_wdl);
    let tg_label = t_string!(i18n, football_tg);
    let tags_label = t_string!(i18n, football_keys_tags);
    let warn = t_string!(i18n, site_warn);
    let topics = f.topics;
    let football_over = f.football_over;
    view! {
        <MatchHeader f=header_f/>
        <OddsTable odds=odds/>
        <PredictionsTable calcs=calcs/>
        <div class="card p-6 mb-6">
            <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">{over_label}</h2>
            {match football_over {
                None => view! {
                    <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_over)}</p>
                }.into_any(),
                Some(ov) => view! {
                    <div class="flex gap-6 flex-wrap text-sm">
                        <div><span class="text-gray-500">{s_label} ": "</span>
                            <span class="font-bold text-lg text-blue-700 dark:text-blue-300">{ov.s}</span></div>
                        <div><span class="text-gray-500">{wdl_label} ": "</span>
                            <span class="font-semibold">{ov.wdl}</span></div>
                        <div><span class="text-gray-500">{tg_label} ": "</span>
                            <span class="font-semibold">{ov.tg}</span></div>
                    </div>
                }.into_any(),
            }}
        </div>
        {if !topics.is_empty() {
            view! {
                <div class="card p-4 mb-6">
                    <p class="text-xs text-gray-500 mb-2">{tags_label}</p>
                    <div class="flex flex-wrap gap-2">
                        {topics.iter().map(|t| view! {
                            <a href=format!("/footballs?filter=topic&fid={}", t.id) class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            }.into_any()
        } else { ().into_any() }}
        <p class="text-xs text-red-400 text-center mt-4">{warn}</p>
    }
}

#[component]
pub fn FootballDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let data = Resource::new_blocking(
        move || id(),
        |id| async move { get_football_and_increment(id).await },
    );
    let loading = t_string!(i18n, loading);

    view! {
        <Nav/>
        <main class="max-w-4xl mx-auto px-4 py-8">
            <Suspense fallback=move || view! { <div class="text-center py-16 text-gray-400">{loading}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => view! { <p class="text-red-500 text-center py-8">{e.to_string()}</p> }.into_any(),
                    Ok(None) => view! {
                        <div class="text-center py-16">
                            <p class="text-gray-500">"Football match not found."</p>
                            <a href="/footballs" class="btn-primary mt-4 inline-block">"Back to list"</a>
                        </div>
                    }.into_any(),
                    Ok(Some(f)) => view! { <FootballDetail f=f/> }.into_any(),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
