use crate::i18n::{Locale, t, t_string};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

use crate::components::{FootballCard, Footer, Nav, Pagination};
use crate::i18n::use_i18n;
use crate::models::{Category, FootballsResult};

use crate::page_title;
use crate::utils::constant::CAT_ITEM;

// ── Server functions ──────────────────────────────────────────────────────────

/// Returns a random published football ID for the "random" nav button.
#[server]
pub async fn get_random_id() -> Result<Option<String>, ServerFnError> {
    use crate::server::football_db;
    football_db::get_random_football_id()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_sidebar_categories() -> Result<Vec<Category>, ServerFnError> {
    use crate::server::category_db;
    category_db::get_categories_by_levels(&[1, 2])
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FootballsFilter {
    All,
    Picks,
    Hot,
    ByCategory(String),
    ByTopic(String),
}

#[server]
pub async fn get_footballs_page(
    from: i64,
    filter: String,
    filter_id: String,
) -> Result<FootballsResult, ServerFnError> {
    use crate::server::football_db;
    let res = match filter.as_str() {
        "picks" => football_db::get_footballs(from, 3, 4).await,
        "hot" => football_db::get_footballs(from, 2, 4).await,
        "category" => football_db::get_footballs_by_category(&filter_id, from).await,
        "topic" => football_db::get_footballs_by_topic(&filter_id, from).await,
        _ => football_db::get_footballs(from, 1, 4).await,
    };
    res.map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Page component ────────────────────────────────────────────────────────────

#[component]
pub fn FootballsPage() -> impl IntoView {
    let i18n = use_i18n();
    let query = use_query_map();

    // Reactive query params
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };
    let filter = move || query.read().get("filter").unwrap_or_default();
    let filter_id = move || query.read().get("fid").unwrap_or_default();

    let cats_res = Resource::new(|| (), |_| get_sidebar_categories());

    let footballs_res = Resource::new(
        move || (from(), filter(), filter_id()),
        |(f, fi, fid)| async move { get_footballs_page(f, fi, fid).await },
    );

    let filter_label = move || match filter().as_str() {
        "picks" => t_string!(i18n, status_picks),
        "hot" => t_string!(i18n, status_hot),
        _ => t_string!(i18n, footballs_list),
    };

    view! {
        <Title text=move || page_title!(i18n, footballs_list)/>
        <Nav/>
        <main class="max-w-6xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-4">
                {filter_label}
            </h1>

            // ── Horizontal category filter bar ───────────────────────────
            <div class="mb-6">
                <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                    <span class="text-sm text-gray-400 dark:text-gray-500 shrink-0 mr-1">
                        {move || t!(i18n, footballs_filter_category)}
                    </span>
                    <a href="/footballs"
                       class=format!("{} text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700", CAT_ITEM)>
                        {move || t!(i18n, all)}
                    </a>
                    <a href="/footballs?filter=picks"
                        class=format!("{} text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50", CAT_ITEM)>
                        {move || t!(i18n, status_picks)}
                    </a>
                    <a href="/footballs?filter=hot"
                        class=format!("{} text-red-500 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50", CAT_ITEM)>
                        {move || t!(i18n, status_hot)}
                    </a>
                    <Suspense fallback=|| ()>
                        {move || cats_res.get().map(|r| r.ok()).flatten().map(|cats| {
                            view! {
                                {cats.into_iter().map(|cat| {
                                    let url = format!("/footballs?filter=category&fid={}", cat.id);
                                    let cat_name = if i18n.get_locale() == Locale::zh { cat.name_zh.clone() } else { cat.name_en.clone() };
                                    view! {
                                        <a href=url class=format!("{} text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700", CAT_ITEM)>
                                            {cat_name}
                                        </a>
                                    }
                                }).collect::<Vec<_>>()}
                            }
                        })}
                    </Suspense>
                </div>
            </div>

            // ── Main content ─────────────────────────────────────────────
            <div>
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center py-16">
                            <div class="text-gray-400">{move || t!(i18n, loading)}</div>
                        </div>
                    }>
                        {move || footballs_res.get().map(|result| match result {
                            Err(e) => view! {
                                <p class="text-red-500 py-8 text-center">{e.to_string()}</p>
                            }.into_any(),
                            Ok(data) => {
                                let pi = data.page_info.clone();
                                let base = format!("/footballs?filter={}&fid={}", filter(), filter_id());
                                view! {
                                    {if data.items.is_empty() {
                                        view! {
                                            <p class="text-center text-gray-400 py-16">
                                                {move || t!(i18n, no_matches)}
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                {data.items.into_iter().map(|f| view! {
                                                    <FootballCard football=f/>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }}
                                    <Pagination page_info=pi base_url=base/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
        </main>
        <Footer/>
    }
}
