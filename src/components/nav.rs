use crate::i18n::t;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::use_auth;
use crate::i18n::{use_i18n, Locale};

const NO_UNDERLINE: &str = "no-underline";
const HOVER_NO_UNDERLINE: &str = "hover:no-underline";

#[component]
pub fn Nav() -> impl IntoView {
    let i18n = use_i18n();
    let auth  = use_auth();

    let is_zh     = move || i18n.get_locale() == Locale::zh;
    let toggle_lang = move |_| {
        i18n.set_locale(if is_zh() { Locale::en } else { Locale::zh });
    };

    // Random football action
    let random_action = Action::new(|_: &()| {
        async move { crate::pages::footballs::get_random_id().await }
    });
    let navigate = leptos_router::hooks::use_navigate();
    Effect::new(move |_| {
        if let Some(Ok(Some(id))) = random_action.value().get() {
            navigate(&format!("/footballs/{}", id), Default::default());
        }
    });

    view! {
        <nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 shadow-sm">
            <div class="max-w-6xl mx-auto px-4">
                <div class="flex items-center justify-between h-12">

                    // ── Logo ─────────────────────────────────────────────────
                    <A href="/" attr:class=format!("font-bold text-blue-600 dark:text-blue-400 text-xl site-title {} {}", NO_UNDERLINE, HOVER_NO_UNDERLINE)>
                        {move || t!(i18n, site_name)}
                    </A>

                    // ── Desktop nav links ─────────────────────────────────────
                    <div class="hidden sm:flex items-center gap-5 text-sm">
                        <A href="/footballs" attr:class=format!("text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                            {move || t!(i18n, nav_footballs)}
                        </A>
                        <A href="/users" attr:class=format!("text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                            {move || t!(i18n, nav_users)}
                        </A>
                    </div>

                    // ── Right actions ─────────────────────────────────────────
                    <div class="flex items-center gap-2 text-sm">
                        // Random football button
                        <button
                            on:click=move |_| { random_action.dispatch(()); }
                            title=move || String::from("") /* was: t!(i18n, random_football) */
                            class="w-8 h-8 flex items-center justify-center rounded-full bg-blue-50 dark:bg-blue-900/30 text-blue-600 hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors "
                        >
                            "🎲"
                        </button>

                        // Language switcher
                        <button
                            on:click=toggle_lang
                            class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors "
                        >
                            {move || if is_zh() { "EN" } else { "中" }}
                        </button>

                        // Auth state
                        {move || {
                            let a = auth.clone();
                            if let Some(user) = a {
                                view! {
                                    <span class="text-gray-700 dark:text-gray-200 font-medium hidden sm:inline">
                                        {user.username.clone()}
                                    </span>
                                    <A href="/sign-out" attr:class=format!("text-xs text-gray-500 hover:text-red-500 {}", NO_UNDERLINE)>
                                        {move || t!(i18n, sign_out)}
                                    </A>
                                }.into_any()
                            } else {
                                view! {
                                    <A href="/sign-in " attr:class=format!("text-sm text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                                        {move || t!(i18n, sign_in)}
                                    </A>
                                    <A href="/register" attr:class="btn-primary text-xs">
                                        {move || t!(i18n, register)}
                                    </A>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </nav>
    }
}
