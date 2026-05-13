use crate::i18n::t;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::use_auth;
use crate::i18n::{Locale, td_string, use_i18n};

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

use crate::utils::constant::{BG_CARD, FLEX_BETWEEN, HOVER_NO_UNDERLINE, NO_UNDERLINE};
use leptos_i18n::Locale as LocaleTrait;

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
        <A href="/footballs">
            {move || t!(i18n, nav_football)}
        </A>
        <A href="/users">
            {move || t!(i18n, nav_user)}
        </A>
    }
}

#[component]
fn NavLeft() -> impl IntoView {
    view! {
        <Logo/>
        <div class="nav-links hidden sm:flex items-center gap-5 text-base">
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
                <span class="hidden sm:inline">{move || t!(i18n, lang)}</span>
                <span class="hidden sm:inline ml-1 opacity-50">"▾"</span>
            </button>
            <div
                class=move || format!("lang-list {} border border-gray-200 dark:border-gray-700 rounded shadow-md py-1 {} absolute top-full left-1/2 -translate-x-1/2 mt-1 whitespace-nowrap z-50", BG_CARD,
                    if open.get() { "" } else { "hidden" })
            >
                {Locale::get_all().iter().map(|&locale| {
                    
                    view! {
                        <button
                            on:click=move |_| { i18n.set_locale(locale); set_open.set(false); }
                            class=move || if i18n.get_locale() == locale { "lang-active" } else { "" }
                        >
                            {td_string!(locale, lang)}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
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
}

#[component]
fn AuthSection() -> impl IntoView {
    let i18n = use_i18n();
    let auth = use_auth();
    move || {
        if let Some(user) = auth.clone() {
            Either::Left(view! {
                <span class="text-gray-700 dark:text-gray-200 font-medium hidden sm:inline text-base">
                    {user.username.clone()}
                </span>
                <A href="/sign-out" attr:class=format!("text-sm text-gray-500 hover:text-red-500 {}", NO_UNDERLINE)>
                    {move || t!(i18n, sign_out)}
                </A>
            })
        } else {
            Either::Right(view! {
                <A href="/sign-in" attr:class="nav-links text-sm">
                    {move || t!(i18n, sign_in)}
                </A>
                <A href="/register" attr:class="nav-links text-sm">
                    {move || t!(i18n, register)}
                </A>
            })
        }
    }
}

// ── Hamburger menu (small screens) ────────────────────────────────────────

#[component]
fn HamburgerMenu() -> impl IntoView {
    let (open, set_open) = signal(false);
    let i18n = use_i18n();
    let auth = use_auth();
    let close = move |_| set_open.set(false);

    view! {
        <div class="sm:hidden">
            <button
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
                class="w-8 h-8 flex items-center justify-center border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200 bg-transparent cursor-pointer text-lg shrink-0"
            >
                {move || if open.get() { "✕" } else { "☰" }}
            </button>

            {move || if open.get() {
                Some(view! { <div class="fixed inset-0 z-40" on:click=move |_| set_open.set(false)></div> })
            } else {
                None
            }}

            <div
                class=move || format!(
                    "hm-menu {} border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg absolute top-full mt-2 z-50 whitespace-nowrap {}",
                    BG_CARD,
                    if open.get() { "" } else { "hidden" }
                )
                style="right:1rem"
            >
                <div class="px-4 py-3 flex flex-col gap-2">
                    <div class="nav-links flex flex-col gap-2"><A href="/footballs" on:click=close>
                        {move || t!(i18n, nav_football)}
                    </A>
                    <A href="/users" on:click=close>
                        {move || t!(i18n, nav_user)}
                    </A>
                    </div>
                    <Random/>
                </div>
                <hr/>
                <div class="px-4 py-3 flex flex-col gap-2">
                    {move || {
                        if auth.is_some() {
                            Either::Left(view! {
                                <A href="/sign-out" on:click=close attr:class="hm-signout text-sm hover:text-red-500">
                                    {move || t!(i18n, sign_out)}
                                </A>
                            })
                        } else {
                            Either::Right(view! { <div class="nav-links text-sm flex flex-col gap-2">
                                <A href="/sign-in" on:click=close>
                                    {move || t!(i18n, sign_in)}
                                </A>
                                <A href="/register" on:click=close>
                                    {move || t!(i18n, register)}
                                </A>
                            </div> })
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ── Right side of the nav bar ─────────────────────────────────────────────

#[component]
fn NavRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 text-sm">
            <LangDropdown/>
            <div class="hidden sm:flex items-center gap-3">
                <Random/>
                <AuthSection/>
            </div>
            <ThemeToggle/>
            <HamburgerMenu/>
        </div>
    }
}

// ── Main Nav ──────────────────────────────────────────────────────────────

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class={format!("{} border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 shadow-sm", BG_CARD)}>
            <div class="max-w-6xl mx-auto px-4">
                <div class={format!("{} h-12", FLEX_BETWEEN)}>
                    <NavLeft/>
                    <NavRight/>
                </div>
            </div>
        </nav>
    }
}
