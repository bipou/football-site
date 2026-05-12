use crate::i18n::t;
use leptos::either::Either;
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

use crate::utils::constant::{BG_CARD, FLEX_BETWEEN, HOVER_NO_UNDERLINE, NO_UNDERLINE, TEXT_MUTED};

// ── Sub-components ────────────────────────────────────────────────────────

#[component]
fn Logo() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <span class="inline-flex items-center">
            <A href="/" attr:class=format!("font-bold text-blue-600 dark:text-blue-400 text-2xl site-title {} {}", NO_UNDERLINE, HOVER_NO_UNDERLINE)>
                {move || t!(i18n, site_name)}
            </A>
            <a href="/doc" class="inline-flex items-center justify-center text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 h-6 px-2 ml-2 no-underline" target="_blank" rel="noopener noreferrer">
                {move || t!(i18n, site_slogan)}
            </a>
        </span>
    }
}

#[component]
fn NavLinks() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <A href="/footballs" attr:class=format!("{} hover:text-blue-600 {}", TEXT_MUTED, NO_UNDERLINE)>
            {move || t!(i18n, nav_football)}
        </A>
        <A href="/users" attr:class=format!("{} hover:text-blue-600 {}", TEXT_MUTED, NO_UNDERLINE)>
            {move || t!(i18n, nav_user)}
        </A>
    }
}

#[component]
fn NavLeft() -> impl IntoView {
    view! {
        <Logo/>
        <div class="hidden sm:flex items-center gap-5 text-base">
            <NavLinks/>
        </div>
    }
}

#[component]
fn Random() -> impl IntoView {
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
            {move || t!(i18n, rand)}
        </a>
    }
}

#[component]
fn LangDropdown() -> impl IntoView {
    let i18n = use_i18n();
    let (open, set_open) = signal(false);
    view! {
        <div class="relative inline-block">
            <button
                class={format!("px-2 py-1 text-sm border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200 {} hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors", BG_CARD)}
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
            >
                "🌐 "
                <span class="hidden sm:inline">{move || t!(i18n, lang_current)}</span>
                <span class="hidden sm:inline ml-1 opacity-50">"▾"</span>
            </button>
            <div
                class=move || format!("{} border border-gray-200 dark:border-gray-700 rounded shadow-md py-1 {} absolute top-full left-1/2 -translate-x-1/2 mt-1 whitespace-nowrap z-50", BG_CARD,
                    if open.get() { "" } else { "hidden" })
            >
                <button
                    on:click=move |_| { i18n.set_locale(Locale::zh); set_open.set(false); }
                    class=move || format!("block w-full text-left px-3 py-1.5 text-sm border-0 cursor-pointer {} {}",
                        if i18n.get_locale() == Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700" },
                        NO_UNDERLINE)
                >
                    {move || t!(i18n, lang_zh)}
                </button>
                <button
                    on:click=move |_| { i18n.set_locale(Locale::en); set_open.set(false); }
                    class=move || format!("block w-full text-left px-3 py-1.5 text-sm border-0 cursor-pointer {} {}",
                        if i18n.get_locale() != Locale::zh { "bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-semibold" }
                        else { "bg-transparent text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-