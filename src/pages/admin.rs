use crate::i18n::{t, use_i18n};
use crate::page_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::app::use_auth;
use crate::components::{Footer, Nav, Pagination};
use crate::models::FootballsResult;

use crate::utils::constant::{
    ALERT_ERROR, ALERT_SUCCESS, EMPTY, GRID_2, H1, HOVER_SHADOW, HOVER_UNDERLINE, MAIN,
    NO_UNDERLINE,
};

// ── Type aliases ─────────────────────────────────────────────────────────────

type Either3<A, B, C> = Either<A, Either<B, C>>;
type Either5<A, B, C, D, E> = Either<A, Either<B, Either<C, Either<D, E>>>>;

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
    use crate::utils::common::into_rid;
    let rid = into_rid(&football_id, "footballs");
    football_db::update_football_status(&rid, status)
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
        <main class=MAIN>
            {if auth.is_none() {
                Either::Left(view! {
                    <div class=EMPTY>
                        <p class="text-gray-500 mb-4">"Please sign in to access the admin area."</p>
                        <a href="/sign-in" class="btn-primary">"Sign In"</a>
                    </div>
                })
            } else {
                Either::Right(view! {
                    <h1 class=H1>
                        {move || t!(i18n, admin_dashboard)}
                    </h1>
                    <div class=GRID_2>
                        <a href="/admin/footballs" class=format!("card p-6 block {} {}", NO_UNDERLINE, HOVER_SHADOW)>
                            <h2 class="text-lg font-semibold text-blue-600 mb-2">"⚽ " {move || t!(i18n, admin_footballs)}</h2>
                            <p class="text-sm text-gray-500">"Manage football match status and visibility."</p>
                        </a>
                        <a href="/users" class=format!("card p-6 block {} {}", NO_UNDERLINE, HOVER_SHADOW)>
                            <h2 class="text-lg font-semibold text-blue-600 mb-2">"👥 Users"</h2>
                            <p class="text-sm text-gray-500">"View and manage registered users."</p>
                        </a>
                    </div>
                })
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
                Either::Left(view! {
                    <div class=EMPTY>
                        <a href="/sign-in" class="btn-primary">"Sign In Required"</a>
                    </div>
                })
            } else {
                Either::Right(view! {
                    <h1 class=H1>
                        {move || t!(i18n, admin_footballs)}
                    </h1>

                    {move || update_action.value().get().map(|r| match r {
                        Ok(()) => Either::Left(view! {
                            <p class=ALERT_SUCCESS>
                                "Status updated successfully."
                            </p>
                        }),
                        Err(e) => Either::Right(view! {
                            <p class=ALERT_ERROR>
                                {e.to_string()}
                            </p>
                        }),
                    })}

                    <Suspense fallback=move || view! { <p class="text-gray-400 text-center py-8">{move || t!(i18n, loading)}</p> }>
                        {move || data.get().map(|result| match result {
                            Err(e) => Either3::<_, _, ()>::Left(view! { <p class="text-red-500">{e.to_string()}</p> }),
                            Ok(d) => {
                                let pi = d.page_info.clone();
                                Either3::Right(Either::<_, ()>::Left(view! {
                                    <div class="space-y-3 mb-8">
                                        {d.items.into_iter().map(|f| {
                                            view! {
                                                <div class="card p-4 flex items-center gap-4 flex-wrap">
                                                    <div class="flex-1 min-w-0">
                                                        <a href=format!("/footballs/{}", crate::utils::common::record_key(&f.id))
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
                                                            let fid_val = f.id.to_string();
                                                            view! {
                                                                <ActionForm action=update_action>
                                                                    <input type="hidden" name="football_id" value=fid_val/>
                                                                    <input type="hidden" name="status" value=s.to_string()/>
                                                                    <button type="submit" class=format!("text-xs px-2 py-1 rounded transition-colors {cls}")>
                                                                        {move || match key {
                                                                            "status_publish" => Either5::Left(t!(i18n, status_publish)),
                                                                            "status_hot" => Either5::Right(Either::Left(t!(i18n, status_hot))),
                                                                            "status_picks" => Either5::Right(Either::Right(Either::Left(t!(i18n, status_picks)))),
                                                                            "status_hide" => Either5::Right(Either::Right(Either::Right(Either::Left(t!(i18n, status_hide))))),
                                                                            _ => Either5::Right(Either::Right(Either::Right(Either::Right("Both")))),
                                                                        }}
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
                                }))
                            }
                        })}
                    </Suspense>
                })
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
        <main class=MAIN>
            {if auth.is_none() {
                Either::Left(view! {
                    <div class=EMPTY>
                        <a href="/sign-in" class="btn-primary">"Sign In Required"</a>
                    </div>
                })
            } else {
                let detail_url = move || format!("/footballs/{}", id());
                Either::Right(view! {
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
                    <crate::pages::football::FootballDetailPage/>
                })
            }}
        </main>
    }
}
