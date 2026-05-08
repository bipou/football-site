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

use crate::utils::constant::{HOVER_NO_UNDERLINE, NO_UNDERLINE};

// ── Sub-components ────────────────────────────────────────────────────────

#[component]
fn Logo() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <span class="inline-flex items-center">
            <A href="/" attr:class=format!("font-bold text-blue-600 dark:text-blue-400 text-2xl site-title {} {}", NO_UNDERLINE, HOVER_NO_UNDERLINE)>
                {move || t!(i18n, site_name)}
            </A>
            <a href="/doc" class="hidden sm:inline-flex items-center justify-center text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 h-6 px-2 ml-2 no-underline" target="_blank" rel="noopener noreferrer">
                {move || t!(i18n, site_slogan)}
            </a>
        </span>
    }
    .into_any()
}

#[component]
fn NavLinks() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="hidden sm:flex items-center gap-5 text-base">
            <A href="/footballs" attr:class=format!("text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                {move || t!(i18n, nav_football)}
            </A>
            <A href="/users" attr:class=format!("text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                {move || t!(i18n, nav_user)}
            </A>
        </div>
    }
    .into_any()
}

#[component]
fn NavLeft() -> impl IntoView {
    view! {
        <Logo/>
        <NavLinks/>
    }
    .into_any()
}

#[component]
fn LuckySlip() -> impl IntoView {
    let i18n = use_i18n();
    let random_action =
        Action::new(|_: &()| async move { crate::pages::footballs::get_random_id().await });
    let navigate = leptos_router::hooks::use_navigate();
    Effect::new(move |_| {
        if let Some(Ok(Some(id))) = random_action.value().get() {
            navigate(&format!("/footballs/{}", id), Default::default());
        }
    });
    view! {
        <a
            href="javascript:void(0)"
            on:click=move |ev| {
                ev.prevent_default();
                random_action.dispatch(());
            }
            class=format!("text-red-500 dark:text-red-400 hover:text-red-600 dark:hover:text-red-300 transition-colors {}", NO_UNDERLINE)
        >
            {move || t!(i18n, slip)}
        </a>
    }
    .into_any()
}

#[component]
fn LangDropdown() -> impl IntoView {
    let i18n = use_i18n();
    let (open, set_open) = signal(false);
    view! {
        <div class="relative inline-block">
            <button
                class="px-2 py-1 text-sm border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
            >
                "🌐 "
                {move || t!(i18n, lang_current)}
                <span class="ml-1 opacity-50">"▾"</span>
            </button>
            <div
                class=move || format!("bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-md py-1 {} absolute top-full left-1/2 -translate-x-1/2 mt-1 whitespace-nowrap z-50",
                    if open.get() { "" } else { "hidden" })
            >
                <button
                    on:click=move |_| { i18n.set_locale(Locale::zh); set_open.set(false); }
                    class=move || format!("block w-full text-left px-3 py-1.5 text-xs border-0 cursor-pointer {} {}",
                        if i18n.get_locale() == Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700" },
                        NO_UNDERLINE)
                >
                    {move || t!(i18n, lang_zh)}
                </button>
                <button
                    on:click=move |_| { i18n.set_locale(Locale::en); set_open.set(false); }
                    class=move || format!("block w-full text-left px-3 py-1.5 text-xs border-0 cursor-pointer {} {}",
                        if i18n.get_locale() != Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700" },
                        NO_UNDERLINE)
                >
                    {move || t!(i18n, lang_en)}
                </button>
            </div>
        </div>
    }
    .into_any()
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let on_click = move |_| {
        #[cfg(feature = "hydrate")]
        {
            toggle_theme();
        }
    };
    view! {
        <button
            title="Toggle theme"
            on:click=on_click
            class="w-7 h-7 flex items-center justify-center rounded-full border-0 bg-transparent cursor-pointer text-base leading-1"
        >
            "🌓"
        </button>
    }
    .into_any()
}

#[component]
fn AuthSection() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();
    move || {
        if let Some(user) = auth.clone() {
            view! {
                <span class="text-gray-700 dark:text-gray-200 font-medium hidden sm:inline text-base">
                    {user.username.clone()}
                </span>
                <A href="/sign-out" attr:class=format!("text-sm text-gray-500 hover:text-red-500 {}", NO_UNDERLINE)>
                    {move || t!(i18n, sign_out)}
                </A>
            }.into_any()
        } else {
            view! {
                <A href="/sign-in" attr:class=format!("text-sm text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                    {move || t!(i18n, sign_in)}
                </A>
                <A href="/register" attr:class=format!("text-sm text-gray-600 dark:text-gray-300 hover:text-blue-600 {}", NO_UNDERLINE)>
                    {move || t!(i18n, register)}
                </A>
            }.into_any()
        }
    }
    .into_any()
}

#[component]
fn NavTools() -> impl IntoView {
    view! {
        <LuckySlip/>
        <LangDropdown/>
    }
    .into_any()
}

#[component]
fn NavActions() -> impl IntoView {
    view! {
        <ThemeToggle/>
        <AuthSection/>
    }
    .into_any()
}

#[component]
fn NavRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 text-sm">
            <NavTools/>
            <NavActions/>
        </div>
    }
    .into_any()
}

// ── Main Nav ──────────────────────────────────────────────────────────────

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 shadow-sm">
            <div class="max-w-6xl mx-auto px-4">
                <div class="flex items-center justify-between h-12">
                    <NavLeft/>
                    <NavRight/>
                </div>
            </div>
        </nav>
    }
    .into_any()
}
