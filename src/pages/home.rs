use crate::i18n::t;
use crate::site_title;
use leptos::prelude::*;
use leptos_meta::Title;
use serde::{Deserialize, Serialize};

use crate::components::{FootballCard, Footer, Nav};
use crate::i18n::use_i18n;
use crate::models::Football;

use crate::utils::constant::HOVER_UNDERLINE;

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
                view! {
                    <p class="text-gray-400 text-sm py-4 text-center">"No matches today"</p>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {footballs.into_iter().map(|f| view! {
                            <FootballCard football=f/>
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
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
                view! {
                    <p class="text-gray-400 text-sm py-4 text-center">"No matches to verify"</p>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {footballs.into_iter().map(|f| view! {
                            <FootballCard football=f/>
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
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
        <main class="max-w-6xl mx-auto px-4 py-8">
            <div class="mb-10 text-center">
                <p class="text-gray-500 dark:text-gray-400 text-sm max-w-2xl mx-auto">
                    {move || t!(i18n, site_intro)}
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
                    Err(e) => view! { <p class="text-red-500 text-center py-8">{e.to_string()}</p> }.into_any(),
                    Ok(d) => view! {
                        <TodaySection footballs=d.jt/>
                        <YesterdaySection footballs=d.zt/>
                    }.into_any(),
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
