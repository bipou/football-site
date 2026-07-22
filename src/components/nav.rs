use crate::app::{AuthMode, AuthPanelSignal};
use crate::i18n::t;
use crate::i18n::{Locale, td_string, use_i18n};
use crate::pages::auth::SignOut;
use crate::shared::auth::use_auth;
use crate::shared::locale::{LocaleA, use_locale};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server::ServerAction;

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

use crate::shared::constant::{BG_CARD, FLEX_BETWEEN, NO_UNDERLINE};
use leptos_i18n::Locale as LocaleTrait;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
fn Logo() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <span class="inline-flex items-center">
            <LocaleA href="/" class=["font-bold text-blue-600 dark:text-blue-400 text-2xl site-title", NO_UNDERLINE].join(" ")>
                {move || t!(i18n, site_name)}
            </LocaleA>
            <a href="/doc" class="inline-flex items-center justify-center text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 h-6 px-2 ml-2 no-underline" target="_blank" rel="noopener noreferrer">
                {move || t!(i18n, site_slogan)}
            </a>
        </span>
    }
}

#[component]
fn NavLinks() -> impl IntoView {
    let i18n = use_i18n();
    let location = use_location();
    let is_active = move |seg: &str| {
        let path = location.pathname.get();
        path.split('/').any(|s| s == seg)
    };
    view! {
        <span class=move || if is_active("footballs") { "nav-active" } else { "" }>
            <LocaleA href="/footballs">
                {move || t!(i18n, nav_football)}
            </LocaleA>
        </span>
        <span class=move || if is_active("users") { "nav-active" } else { "" }>
            <LocaleA href="/users">
                {move || t!(i18n, nav_user)}
            </LocaleA>
        </span>
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
fn LangDropdown() -> impl IntoView {
    let i18n = use_i18n();
    let (open, set_open) = signal(false);
    let nav = use_navigate();
    let loc_str = use_locale();
    let location = use_location();
    view! {
        <div class="relative inline-block">
            <button
                class={["px-2 py-1 text-sm border border-gray-300 dark:border-gray-500 rounded text-gray-700 dark:text-gray-200", BG_CARD, "hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"].join(" ")}
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
                class=move || ["lang-list border border-gray-200 dark:border-gray-700 rounded shadow-md py-1", BG_CARD, if open.get() { "" } else { "hidden" }, "absolute top-full left-1/2 -translate-x-1/2 mt-1 whitespace-nowrap z-50"].join(" ")
            >
                {Locale::get_all().iter().map(|&locale| {
                    let new_loc = locale.to_string();
                    view! {
                        <button
                            on:click={
                                let navigate = nav.clone();
                                move |_| {
                                    let old = loc_str.get_untracked();
                                    let path = location.pathname.get_untracked();
                                    let rest = path.strip_prefix(&["/", &old].join("")).unwrap_or(&path).to_string();
                                    i18n.set_locale(locale);
                                    navigate(&["/", &new_loc, &rest].join(""), Default::default());
                                    set_open.set(false);
                                }
                            }
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
        <button on:click=on_click
            class="border-0 bg-transparent cursor-pointer">
            "☀️"
        </button>
    }
}

#[component]
fn AuthSection() -> impl IntoView {
    let i18n = use_i18n();
    let auth_panel = use_context::<AuthPanelSignal>();
    let sign_out_action = ServerAction::<SignOut>::new();
    let auth_res = use_context::<crate::shared::auth::AuthResource>();
    let navigate = leptos_router::hooks::use_navigate();

    // 签出成功后刷新 auth 并跳首页
    Effect::new(move |_| {
        if sign_out_action.value().get().is_some() {
            if let Some(ref res) = auth_res {
                res.refetch();
            }
            let loc = i18n.get_locale().to_string();
            navigate(&["/", &loc, "/"].join(""), Default::default());
        }
    });

    move || {
        let auth = use_auth();
        match auth {
            Some(user) => Either::Left(view! {
                <span class="text-gray-700 dark:text-gray-200 font-medium hidden sm:inline text-base">
                    {user.username}
                </span>
                <button class="text-sm text-gray-500 hover:text-red-500 border-0 bg-transparent cursor-pointer" on:click=move |_| {
                    sign_out_action.dispatch(SignOut {});
                }>
                    {move || match sign_out_action.pending().get() {
                        true => Either::Left(t!(i18n, signing_out)),
                        false => Either::Right(t!(i18n, sign_out)),
                    }}
                </button>
            }),
            None => Either::Right(view! {
                <button class="nav-links text-sm border-0 bg-transparent cursor-pointer" on:click={
                    let ap = auth_panel.clone();
                    move |_| {
                        if let Some(ref ap) = ap {
                            ap.set(Some(AuthMode::SignIn));
                        }
                    }
                }>
                    {move || t!(i18n, sign_in)}
                </button>
                <button class="nav-links text-sm border-0 bg-transparent cursor-pointer" on:click={
                    let ap = auth_panel.clone();
                    move |_| {
                        if let Some(ref ap) = ap {
                            ap.set(Some(AuthMode::Register));
                        }
                    }
                }>
                    {move || t!(i18n, register)}
                </button>
            }),
        }
    }
}

#[component]
fn HamburgerMenu() -> impl IntoView {
    let (open, set_open) = signal(false);
    let i18n = use_i18n();
    let location = use_location();
    let is_active = move |seg: &str| {
        let path = location.pathname.get();
        path.split('/').any(|s| s == seg)
    };
    let close = Callback::new(move |_: leptos::ev::MouseEvent| set_open.set(false));

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

            {move || open.get().then(|| view! { <div class="fixed inset-0 z-40" on:click=move |_| set_open.set(false)></div> })}

            <div
                class=move || ["hm-menu border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg absolute top-full mt-2 z-50 whitespace-nowrap", BG_CARD, if open.get() { "" } else { "hidden" }].join(" ")
                style="right:1rem"
            >
                <div class="nav-links px-4 py-3 flex flex-col gap-2 items-center">
                    <span class=move || if is_active("footballs") { "nav-active" } else { "" }>
                        <LocaleA href="/footballs" on_click=close>
                            {move || t!(i18n, nav_football)}
                        </LocaleA>
                    </span>
                    <span class=move || if is_active("users") { "nav-active" } else { "" }>
                        <LocaleA href="/users" on_click=close>
                            {move || t!(i18n, nav_user)}
                        </LocaleA>
                    </span>
                </div>
                <hr/>
                <div class="nav-links px-4 py-3 flex flex-col gap-2 items-center">
                    {move || {
                        let auth = use_auth();
                        let auth_panel = use_context::<AuthPanelSignal>();
                        let sign_out_action = ServerAction::<SignOut>::new();
                        match auth {
                            Some(user) => Either::Left(view! {
                                <span class="text-gray-700 dark:text-gray-200 font-medium text-base">
                                    {user.username}
                                </span>
                                <button class="hm-signout text-sm border-0 bg-transparent cursor-pointer" on:click=move |_| {
                                    sign_out_action.dispatch(SignOut {});
                                    set_open.set(false);
                                }>
                                    {move || match sign_out_action.pending().get() {
                                        true => Either::Left(t!(i18n, signing_out)),
                                        false => Either::Right(t!(i18n, sign_out)),
                                    }}
                                </button>
                            }),
                            None => Either::Right(view! {
                                <button class="nav-links text-sm border-0 bg-transparent cursor-pointer" on:click={
                                    let ap = auth_panel.clone();
                                    move |_| {
                                        if let Some(ref ap) = ap {
                                            ap.set(Some(AuthMode::SignIn));
                                        }
                                        set_open.set(false);
                                    }
                                }>
                                    {move || t!(i18n, sign_in)}
                                </button>
                                <button class="nav-links text-sm border-0 bg-transparent cursor-pointer" on:click={
                                    let ap = auth_panel.clone();
                                    move |_| {
                                        if let Some(ref ap) = ap {
                                            ap.set(Some(AuthMode::Register));
                                        }
                                        set_open.set(false);
                                    }
                                }>
                                    {move || t!(i18n, register)}
                                </button>
                            }),
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn NavRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 text-sm">
            <ThemeToggle/>
            <LangDropdown/>
            <div class="hidden sm:flex items-center gap-3">
                <AuthSection/>
            </div>
            <HamburgerMenu/>
        </div>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class={[BG_CARD, "border-b border-gray-200 dark:border-gray-700 sticky top-0 shadow-sm"].join(" ")} style="z-index:52">
            <div class="max-w-6xl mx-auto px-4">
                <div class={[FLEX_BETWEEN, "h-12"].join(" ")}>
                    <NavLeft/>
                    <NavRight/>
                </div>
            </div>
        </nav>
    }
}
