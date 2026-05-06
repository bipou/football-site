use crate::i18n::t;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::use_auth;
use crate::i18n::{Locale, use_i18n};

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen(inline_js = r#"
    export function toggle_theme() {
        console.log("toggle_theme called");
        const h = document.documentElement;
        if (h.classList.contains("dark")) {
            h.classList.remove("dark");
            h.classList.add("light");
            localStorage.setItem("theme", "light");
        } else {
            h.classList.remove("light");
            h.classList.add("dark");
            localStorage.setItem("theme", "dark");
        }
    }
"#)]
extern "C" {
    fn toggle_theme();
}

const NO_UNDERLINE: &str = "no-underline";
const HOVER_NO_UNDERLINE: &str = "hover:no-underline";

#[component]
pub fn Nav() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();

    let (lang_open, set_lang_open) = signal(false);

    // Theme toggle
    let on_theme_click = move |_| {
        #[cfg(feature = "hydrate")]
        {
            toggle_theme();
        }
    };

    // Random football → "Lucky Winning Slip" link
    let random_action =
        Action::new(|_: &()| async move { crate::pages::footballs::get_random_id().await });
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
                    <span>
                        <A href="/" attr:class=format!("font-bold text-blue-600 dark:text-blue-400 text-xl site-title {} {}", NO_UNDERLINE, HOVER_NO_UNDERLINE)>
                            {move || t!(i18n, site_name)}
                        </A>
                        <span class="hidden sm:inline-flex items-center justify-center text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300 align-text-top h-5 px-2 ml-3">
                            {move || t!(i18n, site_slogan)}
                        </span>
                    </span>

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
                    <div class="flex items-center gap-3 text-sm">
                        // Lucky Winning Slip link
                        <a
                            href="javascript:void(0)"
                            on:click=move |ev| {
                                ev.prevent_default();
                                random_action.dispatch(());
                            }
                            class=format!("text-red-500 dark:text-red-400 font-medium hover:text-red-600 dark:hover:text-red-300 transition-colors {}", NO_UNDERLINE)
                        >
                            {move || t!(i18n, lucky_slip)}
                        </a>

                        // Language dropdown
                        <div style="position:relative; display:inline-block">
                            <button
                                class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    set_lang_open.update(|v| *v = !*v);
                                }
                            >
                                "🌐 "
                                {move || t!(i18n, lang_current)}
                                <span class="ml-1 opacity-50">"▾"</span>
                            </button>
                            <div
                                class=move || format!("bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-md py-1 {}",
                                    if lang_open.get() { "" } else { "hidden" })
                                style="position:absolute; top:100%; left:50%; transform:translateX(-50%); margin-top:0.25rem; white-space:nowrap; z-index:50"
                            >
                                <button
                                    on:click=move |_| { i18n.set_locale(Locale::zh); set_lang_open.set(false); }
                                    class=move || format!("block w-full text-left px-3 py-1.5 text-xs border-0 cursor-pointer {} {}",
                                        if i18n.get_locale() == Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700" },
                                        NO_UNDERLINE)
                                >
                                    {move || t!(i18n, lang_zh)}
                                </button>
                                <button
                                    on:click=move |_| { i18n.set_locale(Locale::en); set_lang_open.set(false); }
                                    class=move || format!("block w-full text-left px-3 py-1.5 text-xs border-0 cursor-pointer {} {}",
                                        if i18n.get_locale() != Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700" },
                                        NO_UNDERLINE)
                                >
                                    {move || t!(i18n, lang_en)}
                                </button>
                            </div>
                        </div>

                        // Theme toggle
                        <button
                            title="Toggle theme"
                            on:click=on_theme_click
                            style="width:1.75rem;height:1.75rem;display:flex;align-items:center;justify-content:center;border-radius:50%;border:0;background:transparent;cursor:pointer;font-size:1rem;line-height:1"
                        >
                            "🌓"
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
                                    <A href="/register" attr:class=format!("text-xs text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
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
