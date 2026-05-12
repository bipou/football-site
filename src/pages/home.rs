use crate::i18n::t;
use crate::site_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use serde::{Deserialize, Serialize};

use crate::components::{FootballCard, Footer, Nav};
use crate::i18n::use_i18n;
use crate::models::Football;

use crate::utils::constant::{GRID_3, HOVER_UNDERLINE, TEXT_SUBTLE, WIDE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeData {
    pub jt: Vec<Football>,
    pub zt: Vec<Football>,
}

#[server]
pub async fn get_home_data() -> Result<HomeData, ServerFnError> {
    use crate::server::football_db;
    let jt = football_db::get_footballs_in_position("jt", 6)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let zt = football_db::get_footballs_in_position("zt", 6)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(HomeData { jt, zt })
}

#[component]
fn TodaySection(footballs: Vec<Football>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section class="mb-12">
            <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-200 border-b border-blue-200 dark:border-blue-800 pb-2 mb-4 flex items-center gap-2">
                <span class="text-blue-500">"⚽"</span>
                {move || t!(i18n, footballs_today)}
            </h2>
            {if footballs.is_empty() {
                Either::Left(view! {
                    <p class="text-gray-400 text-sm py-4 text-center">{move || t!(i18n, no_match)}</p>
                })
            } else {
                Either::Right(view! {
                    <div class={GRID_3}>
                        {footballs.into_iter().map(|f| view! {
                            <FootballCard football=f/>
                        }).collect::<Vec<_>>()}
                    </div>
                })
            }}
            <div class="mt-4 text-right">
                <a href="/footballs" class=format!("text-sm text-blue-500 {}", HOVER_UNDERLINE)>
                    {move || t!(i18n, more)}
                </a>
            </div>
        </section>
    }
}

#[component]
fn YesterdaySection(footballs: Vec<Football>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section>
            <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-200 border-b border-gray-200 dark:border-gray-700 pb-2 mb-4 flex items-center gap-2">
                <span class="text-gray-400">"📋"</span>
                {move || t!(i18n, footballs_yesterday)}
            </h2>
            {if footballs.is_empty() {
                Either::Left(view! {
                    <p class="text-gray-400 text-sm py-4 text-center">{move || t!(i18n, no_match)}</p>
                })
            } else {
                Either::Right(view! {
                    <div class={GRID_3}>
                        {footballs.into_iter().map(|f| view! {
                            <FootballCard football=f/>
                        }).collect::<Vec<_>>()}
                    </div>
                })
            }}
            <div class="mt-4 text-right">
                <a href="/footballs" class=format!("text-sm text-blue-500 {}", HOVER_UNDERLINE)>
                    {move || t!(i18n, more)}
                </a>
            </div>
        </section>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();
    let data = Resource::new_blocking(|| (), |_| get_home_data());

    view! {
        <Title text=move || site_title!(i18n)/>
        <Nav/>
        <main class={WIDE}>
            <div class="mb-10 text-center">
                <p class={format!("{} text-sm max-w-2xl mx-auto", TEXT_SUBTLE)}>
                    <a style="white-space: pre-line" href="https://github.com/bipou/football-site" target="_blank" rel="noopener noreferrer">
                        {move || t!(i18n, site_intro)}
                    </a>
                </p>
                <p class="text-xs text-red-400 dark:text-red-500 mt-3 max-w-2xl mx-auto">
                    {move || t!(i18n, site_warn)}
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex justify-center py-16">
                    <div class="text-gray-400 text-sm">{move || t!(i18n, loading)}</div>
                </div>
            }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either::Left(view! {
                        <p class="text-red-500 text-center py-8">{e.to_string()}</p>
                    }),
                    Ok(d) => Either::Right(view! {
                        <TodaySection footballs=d.jt/>
                        <YesterdaySection footballs=d.zt/>
                    }),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
