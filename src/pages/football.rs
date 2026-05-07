const BADGE_BLUE_NO_UL: &str = "badge-blue no-underline";
use crate::i18n::t;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::i18n::use_i18n;
use crate::models::Football;

const BADGE_GRAY: &str = "badge-gray";
const ITALIC: &str = "italic";

#[server]
pub async fn get_football_and_increment(id: String) -> Result<Option<Football>, ServerFnError> {
    use crate::server::football_db;
    let _ = football_db::increment_hits(&id).await;
    football_db::get_football_by_id(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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

    view! {
        <Nav/>
        <main class="max-w-4xl mx-auto px-4 py-8">
            <Suspense fallback=move || view! { <div class="text-center py-16 text-gray-400">{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => view! { <p class="text-red-500 text-center py-8">{e.to_string()}</p> }.into_any(),
                    Ok(None) => view! {
                        <div class="text-center py-16">
                            <p class="text-gray-500">"Football match not found."</p>
                            <a href="/footballs" class="btn-primary mt-4 inline-block">"Back to list"</a>
                        </div>
                    }.into_any(),
                    Ok(Some(f)) => {
                        let title_text = format!("{} vs {} – BiPou", f.home_team, f.away_team);
                        let cat_name  = f.category.as_ref().map(|c| c.name_en.clone()).unwrap_or_default();
                        view! {
                            <Title text=title_text/>
                            // ── Match header ────────────────────────────────────
                            <div class="card p-6 mb-6">
                                <div class="flex items-start justify-between flex-wrap gap-4">
                                    <div>
                                        <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-1">
                                            {f.home_team.clone()} <span class="text-gray-400 mx-2">"vs"</span> {f.away_team.clone()}
                                        </h1>
                                        <div class="text-sm text-gray-500 space-x-3">
                                            <span>{move || t!(i18n, football_season)} " " {f.season.clone()}</span>
                                            {if !cat_name.is_empty() {
                                                view! { <span class=BADGE_GRAY>{cat_name}</span> }.into_any()
                                            } else { view! { <span/> }.into_any() }}
                                        </div>
                                    </div>
                                    <div class="text-right text-sm text-gray-500">
                                        <div>{move || t!(i18n, football_kick_off)}</div>
                                        <div class="font-semibold text-blue-600">{f.kick_off_at_mdhm8.clone()}</div>
                                        <div class="text-xs text-gray-400">"UTC: " {f.kick_off_at_mdhm.clone()}</div>
                                    </div>
                                </div>
                                <div class="mt-3 text-xs text-gray-400 flex gap-4 flex-wrap">
                                    <span>{move || t!(i18n, football_created)} ": " {f.created_at.clone()}</span>
                                    <span>{move || t!(i18n, football_updated)} ": " {f.updated_at.clone()}</span>
                                    <span>{move || t!(i18n, football_hits)} {f.hits}</span>
                                </div>
                            </div>

                            // ── Odds table ──────────────────────────────────────
                            <div class="card p-6 mb-6">
                                <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">"Odds"</h2>
                                {if f.il_odds.is_empty() {
                                    view! { <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_pred)}</p> }.into_any()
                                } else {
                                    let init = f.il_odds.first().cloned();
                                    let last = f.il_odds.last().cloned();
                                    view! {
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
                                                        <tr class="border-b border-gray-100 dark:border-gray-700">
                                                            <td class="px-4 py-2 text-gray-500">{move || t!(i18n, football_init_odds)}</td>
                                                            <td class="px-4 py-2 font-medium text-green-600">{o.win.clone()}</td>
                                                            <td class="px-4 py-2 font-medium text-gray-600">{o.draw.clone()}</td>
                                                            <td class="px-4 py-2 font-medium text-red-600">{o.loss.clone()}</td>
                                                            <td class="px-4 py-2 text-xs text-gray-400">{o.created_at.clone()}</td>
                                                        </tr>
                                                    })}
                                                    {last.and_then(|o| if f.il_odds.len() > 1 { Some(view! {
                                                        <tr class="border-b border-gray-100 dark:border-gray-700">
                                                            <td class="px-4 py-2 text-gray-500">{move || t!(i18n, football_last_odds)}</td>
                                                            <td class="px-4 py-2 font-medium text-green-600">{o.win.clone()}</td>
                                                            <td class="px-4 py-2 font-medium text-gray-600">{o.draw.clone()}</td>
                                                            <td class="px-4 py-2 font-medium text-red-600">{o.loss.clone()}</td>
                                                            <td class="px-4 py-2 text-xs text-gray-400">{o.created_at.clone()}</td>
                                                        </tr>
                                                    }) } else { None })}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }}
                            </div>

                            // ── Predictions table ──────────────────────────────
                            <div class="card p-6 mb-6">
                                <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">"Predictions"</h2>
                                {if f.il_pred_over.is_empty() {
                                    view! { <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_pred)}</p> }.into_any()
                                } else {
                                    let init = f.il_pred_over.first().cloned();
                                    let last = f.il_pred_over.last().cloned();
                                    view! {
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
                                                        <tr class="border-b border-gray-100 dark:border-gray-700">
                                                            <td class="px-4 py-2 text-gray-500">"Initial"</td>
                                                            <td class="px-4 py-2 font-medium">{c.s.clone()}</td>
                                                            <td class="px-4 py-2">{c.wdl.clone()}</td>
                                                            <td class="px-4 py-2">{c.tg.clone()}</td>
                                                            <td class="px-4 py-2">{c.gd.clone()}</td>
                                                            <td class="px-4 py-2 text-xs text-gray-400">{c.created_at.clone()}</td>
                                                        </tr>
                                                    })}
                                                    {last.and_then(|c| if f.il_pred_over.len() > 1 { Some(view! {
                                                        <tr class="border-b border-gray-100 dark:border-gray-700">
                                                            <td class="px-4 py-2 text-gray-500">"Latest"</td>
                                                            <td class="px-4 py-2 font-medium">{c.s.clone()}</td>
                                                            <td class="px-4 py-2">{c.wdl.clone()}</td>
                                                            <td class="px-4 py-2">{c.tg.clone()}</td>
                                                            <td class="px-4 py-2">{c.gd.clone()}</td>
                                                            <td class="px-4 py-2 text-xs text-gray-400">{c.created_at.clone()}</td>
                                                        </tr>
                                                    }) } else { None })}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }}
                            </div>

                            // ── Official result ──────────────────────────────────
                            <div class="card p-6 mb-6">
                                <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-4">
                                    {move || t!(i18n, football_over)}
                                </h2>
                                {match f.football_over.clone() {
                                    None => view! {
                                        <p class=format!("text-gray-400 text-sm {}", ITALIC)>{move || t!(i18n, not_over)}</p>
                                    }.into_any(),
                                    Some(ov) => view! {
                                        <div class="flex gap-6 flex-wrap text-sm">
                                            <div><span class="text-gray-500">{move || t!(i18n, football_s)} ": "</span>
                                                <span class="font-bold text-lg text-blue-700 dark:text-blue-300">{ov.s.clone()}</span></div>
                                            <div><span class="text-gray-500">{move || t!(i18n, football_wdl)} ": "</span>
                                                <span class="font-semibold">{ov.wdl.clone()}</span></div>
                                            <div><span class="text-gray-500">{move || t!(i18n, football_tg)} ": "</span>
                                                <span class="font-semibold">{ov.tg.clone()}</span></div>
                                        </div>
                                    }.into_any(),
                                }}
                            </div>

                            // ── Topics ────────────────────────────────────────────
                            {if !f.topics.is_empty() {
                                view! {
                                    <div class="card p-4 mb-6">
                                        <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, football_keys_tags)}</p>
                                        <div class="flex flex-wrap gap-2">
                                            {f.topics.iter().map(|t| view! {
                                                <a href=format!("/footballs?filter=topic&fid={}", t.id)
                                                   class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            } else { view! { <span/> }.into_any() }}

                            <p class="text-xs text-red-400 text-center mt-4">
                                {move || t!(i18n, site_warn)}
                            </p>
                        }.into_any()
                    }
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
