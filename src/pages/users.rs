use crate::i18n::t;
use crate::page_title;
use crate::site_title;
use crate::utils::constant::{
    BADGE_BLUE_NO_UL, BADGE_GRAY_NO_UL, EMPTY, FLEX_WRAP_GAP, GRID_3, H1, MAIN, WIDE,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::{Footer, Nav, Pagination};
use crate::i18n::use_i18n;
use crate::models::{User, UsersResult};

const CARD_BLOCK_NO_UL: &str = "card p-4 block no-underline hover:shadow-md transition-shadow";
const PROSE_CLASS: &str = "prose prose-sm dark:prose-invert max-w-none";
const RISK_CLASS: &str = "text-xs text-gray-400 text-center mt-6";

// ── Type alias ───────────────────────────────────────────────────────────

type Either3<A, B, C> = Either<A, Either<B, C>>;

#[server]
pub async fn get_users_page(from: i64) -> Result<UsersResult, ServerFnError> {
    use crate::server::user_db;
    user_db::get_users(from)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_user_profile(username: String) -> Result<Option<User>, ServerFnError> {
    use crate::server::user_db;
    user_db::get_user_by_username(&username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let i18n = use_i18n();
    let query = use_query_map();
    let from = move || {
        query
            .read()
            .get("from")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1i64)
    };

    let data = Resource::new(move || from(), |f| async move { get_users_page(f).await });

    view! {
        <Title text=move || page_title!(i18n, users_list)/>
        <Nav/>
        <main class={WIDE}>
            <h1 class={H1}>
                {move || t!(i18n, users_list)}
            </h1>
            <Suspense fallback=move || view! { <div class={format!("{} text-gray-400", EMPTY)}>{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either::Left(view! {
                        <p class="text-red-500 text-center">{e.to_string()}</p>
                    }),
                    Ok(d) => {
                        let pi = d.page_info.clone();
                        Either::Right(view! {
                            <div class={format!("{} mb-8", GRID_3)}>
                                {d.items.into_iter().map(|u| {
                                    let url = format!("/users/{}", u.username);
                                    let uname = u.username.to_string();
                                    let initial = u.username.chars().next().unwrap_or('?');
                                    let updated = u.updated_at.clone();
                                    view! {
                                        <a href=url class=CARD_BLOCK_NO_UL>
                                            <div class="flex items-start gap-3">
                                                <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-2xl shrink-0 mt-1">
                                                    {initial.to_string()}
                                                </div>
                                                <div class="min-w-0 flex-1">
                                                    <p class="text-2xl font-bold text-gray-800 dark:text-gray-100 truncate">{uname}</p>
                                                    <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)} {updated}</p>
                                                    {if !u.keywords.is_empty() {
                                                        Either::Left(view! {
                                                            <div class="flex flex-wrap gap-1 mt-1">
                                                                {u.keywords.iter().take(8).map(|t| {
                                                                    let kid = crate::utils::common::record_key(&t.id).to_string();
                                                                    let url = format!("/footballs?topic={}", kid);
                                                                    view! {
                                                                        <a href=url class=format!("text-sm {}", BADGE_BLUE_NO_UL)>{t.name.clone()}</a>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>
                                                        })
                                                    } else {
                                                        Either::Right(())
                                                    }}
                                                </div>
                                            </div>
                                        </a>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <Pagination page_info=pi base_url="/users".to_string()/>
                        })
                    }
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}

// ── User profile sub-components ──────────────────────────────────────────

#[component]
fn IntroSection(intro_html: String) -> impl IntoView {
    let i18n = use_i18n();
    if intro_html.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-6 mb-6">
                <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-3">
                    {move || t!(i18n, user_intro)}
                </h2>
                <article class=PROSE_CLASS inner_html=intro_html/>
            </div>
        })
    }
}

#[component]
fn KeywordsTopics(
    keywords: Vec<crate::models::Topic>,
    topics: Vec<crate::models::Topic>,
) -> impl IntoView {
    let i18n = use_i18n();
    if keywords.is_empty() && topics.is_empty() {
        Either::Left(())
    } else {
        Either::Right(view! {
            <div class="card p-6">
                {if !keywords.is_empty() {
                    Either::Left(view! {
                        <div class="mb-4">
                            <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, features_topics)}</p>
                            <div class={FLEX_WRAP_GAP}>
                                {keywords.iter().map(|t| {
                                    let kid = crate::utils::common::record_key(&t.id).to_string();
                                    let url = format!("/footballs?topic={}", kid);
                                    view! {
                                        <a href=url class=BADGE_BLUE_NO_UL>{t.name.clone()}</a>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                } else {
                    Either::Right(())
                }}
                {if !topics.is_empty() {
                    Either::Left(view! {
                        <div>
                            <p class="text-xs text-gray-500 mb-2">{move || t!(i18n, related_topics)}</p>
                            <div class={FLEX_WRAP_GAP}>
                                {topics.iter().map(|t| {
                                    let kid = crate::utils::common::record_key(&t.id).to_string();
                                    let url = format!("/footballs?topic={}", kid);
                                    view! {
                                        <a href=url class=BADGE_GRAY_NO_UL>
                                            {t.name.clone()}
                                        </a>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                } else {
                    Either::Right(())
                }}
            </div>
        })
    }
}

#[component]
pub fn UserProfilePage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let username = move || params.read().get("username").unwrap_or_default();

    let data = Resource::new_blocking(
        move || username(),
        |u| async move { get_user_profile(u).await },
    );

    view! {
        <Nav/>
        <main class={MAIN}>
            <Suspense fallback=move || view! { <div class={format!("{} text-gray-400", EMPTY)}>{move || t!(i18n, loading)}</div> }>
                {move || data.get().map(|result| match result {
                    Err(e) => Either3::Left(view! {
                        <p class="text-red-500 text-center">{e.to_string()}</p>
                    }),
                    Ok(None) => Either3::Right(Either::Left(view! {
                        <div class={EMPTY}>
                            <h1 class="text-xl font-bold text-gray-800 dark:text-gray-100 mb-4">User not found</h1>
                            <a href="/users" class="btn-primary">Back to users</a>
                        </div>
                    })),
                    Ok(Some(user)) => {
                        let created_at = user.created_at.clone();
                        let updated_at = user.updated_at.clone();
                        let uname = user.username.to_string();
                        let title = format!("{} – {}", uname, site_title!(i18n));
                        let initial = user.username.chars().next().unwrap_or('?');
                        let intro_html = user.introduction_html.clone();
                        let keywords = user.keywords.clone();
                        let topics = user.topics.clone();
                        Either3::Right(Either::Right(view! {
                            <Title text=title/>

                            <div class="card p-6 mb-6">
                                <div class="flex items-start gap-3">
                                    <div class="w-14 h-14 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 font-bold text-2xl shrink-0 mt-1">
                                        {initial.to_string()}
                                    </div>
                                    <div class="min-w-0 flex-1">
                                        <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">{uname}</h1>
                                        <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, registration_time)} {created_at}</p>
                                        <p class="text-xs text-gray-400 mt-1">{move || t!(i18n, profile_updated)} {updated_at}</p>
                                    </div>
                                </div>
                            </div>

                            <IntroSection intro_html=intro_html/>
                            <KeywordsTopics keywords=keywords topics=topics/>

                            <p class=RISK_CLASS>
                                {move || t!(i18n, user_warn)}
                            </p>
                        }))
                    }
                })}
            </Suspense>
        </main>
        <Footer/>
    }
}
