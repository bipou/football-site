use crate::i18n::{t, t_string};
use crate::page_title;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::app::use_auth;
use crate::components::{Footer, Nav, Pagination};
use crate::i18n::use_i18n;
use crate::models::FootballsResult;

use crate::utils::constant::{HOVER_UNDERLINE, NO_UNDERLINE};

// ── Server functions ──────────────────────────────────────────────────────────

#[server]
pub async fn get_admin_footballs(from: i64) -> Result<FootballsResult, ServerFnError> {
    use crate::server::football_db;
    football_db::get_footballs_admin(from)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn admin_update_status(football_id: String, status: i8) -> Result<(), ServerFnError> {
    use crate::server::football_db;
    football_db::update_football_status(&football_id, status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Admin dashboard ───────────────────────────────────────────────────────────

#[component]
pub fn AdminPage() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();

    view! {
        <Title text=move || page_title!(i18n, admin_dashboard)/>
        <Nav/>
        <main class="max-w-4xl mx-auto px-4 py-8">
            {if auth.is_none() {
                view! {
                    <div class="text-center py-16">
                        <p class="text-gray-500 mb-4">"Please sign in to access the admin area."</p>
                        <a href="/sign-in" class="btn-primary">"Sign In"</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <h1 class="text-xl font-bold text-gray-800 dark:text-gray-100 mb-6">
                        {move || t!(i18n, admin_dashboard)}
                    </h1>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <a href="/admin/footballs" class=format!("card p-6 block {} hover:shadow-md transition-shadow", NO_UNDERLINE)>
                            <h2 class="text-lg font-semibold text-blue-600 mb-2">"⚽ " {move || t!(i18n, admin_footballs)}</h2>
                            <p class="text-sm text-gray-500">"Manage football match status and visibility."</p>
                        </a>
                        <a href="/users" class=format!("card p-6 block {} hover:shadow-md transition-shadow", NO_UNDERLINE)>
                            <h2 class="text-lg font-semibold text-blue-600 mb-2">"👥 Users"</h2>
                            <p class="text-sm text-gray-500">"View and manage registered users."</p>
                        </a>
                    </div>
                }.into_any()
            }}
        </main>
        <Footer/>
    }
}

// ── Admin football list ───────────────────────────────────────────────────────

#[component]
pub fn AdminFootballsPage() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();
    let query = use_query_map();
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };

    let data = Resource::new(
        move || from(),
        |f| async move { get_admin_footballs(f).await },
    );
    let update_action = ServerAction::<AdminUpdateStatus>::new();

    view! {
        <Title text=move || page_title!(i18n, admin_footballs)/>
        <Nav/>
        <main class="max-w-5xl mx-auto px-4 py-8">
            {if auth.is_none() {
                view! {
                    <div class="text-center py-16">
                        <a href="/sign-in" class="btn-primary">"Sign In Required"</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <h1 class="text-xl font-bold text-gray-800 dark:text-gray-100 mb-6">
                        {move || t!(i18n, admin_footballs)}
                    </h1>

                    {move || update_action.value().get().map(|r| match r {
                        Ok(()) => view! {
                            <p class="text-green-500 text-sm mb-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded p-2">
                                "Status updated successfully."
                            </p>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-red-500 text-sm mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-2">
                                {e.to_string()}
                            </p>
                        }.into_any(),
                    })}

                    <Suspense fallback=move || view! { <p class="text-gray-400 text-center py-8">{move || t!(i18n, loading)}</p> }>
                        {move || data.get().map(|result| match result {
                            Err(e) => view! { <p class="text-red-500">{e.to_string()}</p> }.into_any(),
                            Ok(d) => {
                                let pi = d.page_info.clone();
                                view! {
                                    <div class="space-y-3 mb-8">
                                        {d.items.into_iter().map(|f| {
                                            let fid = f.id.clone();
                                            let fid2 = f.id.clone();
                                            view! {
                                                <div class="card p-4 flex items-center gap-4 flex-wrap">
                                                    <div class="flex-1 min-w-0">
                                                        <a href=format!("/footballs/{}", fid)
                                                           class=format!("font-semibold text-gray-800 dark:text-gray-100 hover:text-blue-600 {} text-sm", NO_UNDERLINE)>
                                                                                                                       {f.home_team.clone()} " vs " {f.away_team.clone()}
                                                        </a>
                                                        <p class="text-xs text-gray-400 mt-1">
                                                            {f.season.clone()} " · " {f.kick_off_at_mdhm8.clone()}
                                                            " · Status: "
                                                            <span class="font-medium text-gray-600">{f.status}</span>
                                                        </p>
                                                    </div>
                                                    // Status action buttons
                                                    <div class="flex gap-1 flex-wrap">
                                                        {[
                                                            (1i8, "status_publish", "bg-blue-100 hover:bg-blue-200 text-blue-700 dark:bg-blue-900/30 dark:hover:bg-blue-900/50 dark:text-blue-300"),
                                                            (2, "status_hot", "bg-indigo-100 hover:bg-indigo-200 text-indigo-700 dark:bg-indigo-900/30 dark:hover:bg-indigo-900/50 dark:text-indigo-300"),
                                                            (3, "status_picks", "bg-orange-100 hover:bg-orange-200 text-orange-700 dark:bg-orange-900/30 dark:hover:bg-orange-900/50 dark:text-orange-300"),
                                                            (4, "status_both", "bg-red-100 hover:bg-red-200 text-red-700 dark:bg-red-900/30 dark:hover:bg-red-900/50 dark:text-red-300"),
                                                            (0, "status_hide", "bg-gray-100 hover:bg-gray-200 text-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-300"),
                                                        ].into_iter().map(|(s, key, cls)| {
                                                            let fid3 = fid2.clone();
                                                            let label: String = match key {
                                                                "status_publish" => t_string!(i18n, status_publish).to_owned(),
                                                                "status_hot" => t_string!(i18n, status_hot).to_owned(),
                                                                "status_picks" => t_string!(i18n, status_picks).to_owned(),
                                                                "status_hide" => t_string!(i18n, status_hide).to_owned(),
                                                                _ => "Both".to_string(),
                                                            };
                                                            view! {
                                                                <ActionForm action=update_action>
                                                                    <input type="hidden" name="football_id" value=fid3.clone()/>
                                                                    <input type="hidden" name="status" value=s.to_string()/>
                                                                    <button type="submit" class=format!("text-xs px-2 py-1 rounded transition-colors {cls}")>
                                                                        {label}
                                                                    </button>
                                                                </ActionForm>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                    <Pagination page_info=pi base_url="/admin/footballs".to_string()/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                }.into_any()
            }}
        </main>
        <Footer/>
    }
}

// ── Admin football detail ─────────────────────────────────────────────────────

#[component]
pub fn AdminFootballDetailPage() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    view! {
        <Title text="BiPou"/>
        <Nav/>
        <main class="max-w-4xl mx-auto px-4 py-8">
            {if auth.is_none() {
                view! {
                    <div class="text-center py-16">
                        <a href="/sign-in" class="btn-primary">"Sign In Required"</a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let detail_url = move || format!("/footballs/{}", id());
                view! {
                    <div class="flex items-center gap-4 mb-6">
                        <a href="/admin/footballs" class="text-sm text-gray-500 hover:text-blue-600">
                            "← Back to admin list"
                        </a>
                        <a href=detail_url class=format!("text-sm text-blue-500 {}", HOVER_UNDERLINE)>
                            "Public view →"
                        </a>
                        <h1 class="text-xl font-bold text-gray-800 dark:text-gray-100 ml-2">
                            {move || t!(i18n, admin_football_detail)}
                        </h1>
                    </div>
                    // Reuse the public detail page component
                    <crate::pages::football::FootballDetailPage/>
                }.into_any()
            }}
        </main>
    }
}
