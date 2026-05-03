use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::i18n::I18nContextProvider;
use crate::models::AuthUser;
use crate::pages::{
    admin::{AdminFootballDetailPage, AdminFootballsPage, AdminPage},
    auth::{RegisterPage, SignInPage, SignOutPage, UserActivatePage},
    football::FootballDetailPage,
    footballs::FootballsPage,
    home::HomePage,
    users::{UserProfilePage, UsersPage},
};

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
                <link rel="icon" href="/favicon.ico"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="min-h-screen bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-100">
                <App/>
            </body>
        </html>
    }
}

/// JWT cookie auth check — runs server-side.
#[server]
pub async fn get_auth_user() -> Result<Option<AuthUser>, ServerFnError> {
    use crate::server::auth::{decode_jwt, get_cookie_value};
    use axum::http::HeaderMap;

    let headers: HeaderMap = leptos_axum::extract().await?;
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let token = match get_cookie_value(cookie, "fs_token") {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(None),
    };

    match decode_jwt(&token) {
        Ok(claims) => Ok(Some(AuthUser {
            username: claims.username,
            token,
        })),
        Err(_) => Ok(None),
    }
}

pub type AuthResource = Resource<Result<Option<AuthUser>, ServerFnError>>;

/// Call inside any reactive scope to get the current user (if signed in).
pub fn use_auth() -> Option<AuthUser> {
    use_context::<AuthResource>()
        .and_then(|r| r.get())
        .and_then(|r| r.ok())
        .flatten()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Blocking: SSR waits until auth resolved before rendering any route.
    let auth_res: AuthResource = Resource::new_blocking(|| (), |_| get_auth_user());
    provide_context(auth_res);

    view! {
        <Stylesheet id="leptos" href="/pkg/football_site.css"/>
        <Title text="BiPou – Football Calculation"/>

        <I18nContextProvider>
            <Suspense fallback=|| view! { <p class="p-4 text-center">"Loading..."</p> }>
                <Router>
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=path!("/")                    view=HomePage/>
                        <Route path=path!("/register")            view=RegisterPage/>
                        <Route path=path!("/sign-in")             view=SignInPage/>
                        <Route path=path!("/sign-out")            view=SignOutPage/>
                        <Route path=path!("/footballs")           view=FootballsPage/>
                        <Route path=path!("/footballs/:id")       view=FootballDetailPage/>
                        <Route path=path!("/users")               view=UsersPage/>
                        <Route path=path!("/users/:username")     view=UserProfilePage/>
                        <Route path=path!("/users/:id/activate")  view=UserActivatePage/>
                        <Route path=path!("/admin")               view=AdminPage/>
                        <Route path=path!("/admin/footballs")     view=AdminFootballsPage/>
                        <Route path=path!("/admin/football/:id")  view=AdminFootballDetailPage/>
                    </Routes>
                </Router>
            </Suspense>
        </I18nContextProvider>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center space-y-4 p-8">
                <h1 class="text-7xl font-bold text-blue-600">"404"</h1>
                <p class="text-xl text-gray-500 dark:text-gray-400">"Page not found"</p>
                <a href="/" class="btn-primary inline-block mt-4">"Go Home"</a>
            </div>
        </div>
    }
}
