use crate::i18n::{t, use_i18n};
use crate::page_title;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::utils::constant::{GRID_2, H1, HOVER_UNDERLINE, TEXT_SUBTLE};

// ── Type alias ───────────────────────────────────────────────────────────
type Either4<A, B, C, D> = Either<A, Either<B, Either<C, D>>>;
type Either3<A, B, C> = Either<A, Either<B, C>>;

// ── Sign In server function ───────────────────────────────────────────────────

#[server]
pub async fn sign_in(signature: String, password: String) -> Result<(), ServerFnError> {
    use crate::server::{auth as auth_mod, user_db};
    use axum::http::{HeaderValue, header};

    let auth_user = user_db::sign_in(&signature, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let cookie = auth_mod::make_set_cookie("fs_token", &auth_user.token, 30 * 24 * 3600);
    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.insert_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    leptos_axum::redirect("/footballs");
    Ok(())
}

// ── Sign Out server function ──────────────────────────────────────────────────

#[server]
pub async fn sign_out() -> Result<(), ServerFnError> {
    use crate::server::auth as auth_mod;
    use axum::http::{HeaderValue, header};

    let resp = expect_context::<leptos_axum::ResponseOptions>();
    resp.insert_header(
        header::SET_COOKIE,
        HeaderValue::from_str(&auth_mod::make_clear_cookie("fs_token"))
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

// ── Register server function ──────────────────────────────────────────────────

#[server]
pub async fn register(
    username: String,
    email: String,
    password: String,
    nickname: String,
    phone_number: String,
    phone_public: bool,
    im_account: String,
    im_public: bool,
    website: String,
    introduction: String,
    topics: String,
    lang: String,
) -> Result<(), ServerFnError> {
    use crate::server::{email as email_mod, user_db};

    let data = user_db::RegisterData {
        username,
        email,
        password,
        nickname: nickname.clone(),
        phone_number,
        phone_public,
        im_account,
        im_public,
        website,
        introduction,
        topics,
    };

    let (user_id, _, username) = user_db::register_user(data)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Send activation email (non-fatal)
    if let Ok(Some((email_addr, _))) = user_db::get_user_email_username(&user_id).await {
        let _ =
            email_mod::send_activation_email(&lang, &username, &user_id, &email_addr)
                .await;
    }

    Ok(())
}

// ── Activate / Resend server functions ────────────────────────────────────────

#[server]
pub async fn activate_user(user_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::user_db;
    user_db::activate_user(&user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn resend_activation(user_id: String, lang: String) -> Result<(), ServerFnError> {
    use crate::server::{email as email_mod, user_db};
    let (email, username) = user_db::get_user_email_username(&user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("User not found".to_string()))?;
    email_mod::send_activation_email(&lang, &username, &user_id, &email)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Sign In page component ────────────────────────────────────────────────────

#[component]
pub fn SignInPage() -> impl IntoView {
    let i18n = use_i18n();
    let action = ServerAction::<SignIn>::new();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = action.value().get() {
            navigate("/footballs", Default::default());
        }
    });

    view! {
        <Title text=move || page_title!(i18n, user_sign_in)/>
        <Nav/>
        <main class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="card p-8 w-full max-w-sm">
                <h1 class=format!("{} text-center", H1)>
                    {move || t!(i18n, user_sign_in)}
                </h1>

                <div class="space-y-4">
                    <ActionForm action=action>
                    <div>
                        <label class="form-label">{move || t!(i18n, sign_in_account)}</label>
                        <input type="text" name="signature" required
                               placeholder=move || String::from("") /* was: t!(i18n, sign_in_account_tip) */
                               class="form-input " autocomplete="username"/>
                    </div>
                    <div>
                        <label class="form-label">{move || t!(i18n, sign_in_password)}</label>
                        <input type="password" name="password" required
                               placeholder=move || String::from("") /* was: t!(i18n, sign_in_password_tip) */
                               class="form-input " autocomplete="current-password"/>
                    </div>

                    // Error
                    {move || action.value().get().and_then(|r| r.err()).map(|e| {
                        let msg = move || if e.to_string().contains("sign_in_incorrect") {
                            Either4::Left(t!(i18n, sign_in_incorrect))
                        } else if e.to_string().contains("sign_in_not_activation") {
                            Either4::Right(Either::Left(t!(i18n, sign_in_not_activation)))
                        } else if e.to_string().contains("sign_in_banned") {
                            Either4::Right(Either::Right(Either::Left(t!(i18n, sign_in_banned))))
                        } else {
                            Either4::Right(Either::Right(Either::Right(t!(i18n, sign_in_security_problem))))
                        };
                        view! { <p class="text-red-500 text-sm text-center">{msg}</p> }
                    })}

                    <button type="submit"
                            disabled=move || action.pending().get()
                            class="btn-primary w-full justify-center">
                        {move || if action.pending().get() { "Signing in..." } else { "" }}
                        {move || t!(i18n, sign_in)}
                    </button>
                    </ActionForm>
                </div>

                <p class="mt-4 text-sm text-center text-gray-500">
                    {move || t!(i18n, sign_in_new_user)} " "
                    <a href="/register" class=format!("text-blue-500 {}", HOVER_UNDERLINE)>{move || t!(i18n, sign_in_create_account)}</a>
                </p>
            </div>
        </main>
        <Footer/>
    }
}

// ── Sign Out page component ───────────────────────────────────────────────────

#[component]
pub fn SignOutPage() -> impl IntoView {
    let action = ServerAction::<SignOut>::new();
    let navigate = leptos_router::hooks::use_navigate();

    // Auto-trigger sign-out on mount
    Effect::new(move |_| {
        action.dispatch(SignOut {});
    });
    Effect::new(move |_| {
        if action.value().get().is_some() {
            navigate("/", Default::default());
        }
    });

    view! {
        <div class="min-h-screen flex items-center justify-center">
            <p class=format!("{} text-lg", TEXT_SUBTLE)>"Signing out..."</p>
        </div>
    }
}

// ── Register page component ───────────────────────────────────────────────────

#[component]
pub fn RegisterPage() -> impl IntoView {
    let i18n = use_i18n();
    let action = ServerAction::<Register>::new();
    let (success, set_success) = signal(false);

    Effect::new(move |_| {
        if let Some(Ok(())) = action.value().get() {
            set_success.set(true);
        }
    });

    view! {
        <Title text=move || page_title!(i18n, user_register)/>
        <Nav/>
        <main class="max-w-2xl mx-auto px-4 py-8">
            <div class="card p-8">
                <h1 class=H1>
                    {move || t!(i18n, user_register)}
                </h1>

                <Show when=move || success.get() fallback=|| ()>
                    <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4 mb-6 text-center">
                        <p class="text-green-700 dark:text-green-300 text-sm font-medium">
                            {move || t!(i18n, register_success)}
                        </p>
                        <a href="/sign-in" class="btn-primary mt-4 inline-block">
                            {move || t!(i18n, sign_in)}
                        </a>
                    </div>
                </Show>

                <Show when=move || !success.get() fallback=|| ()>
                    <div class="space-y-4">
                    <ActionForm action=action>
                        <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                        <input type="hidden" name="phone_public" value="false"/>
                        <input type="hidden" name="im_public" value="false"/>

                        <div class=GRID_2>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_username)} " *"</label>
                                <input type="text" name="username" required
                                       placeholder=move || String::from("") /* was: t!(i18n, register_username_tip) */
                                       class="form-input " pattern="[a-z0-9_-]+" autocomplete="username"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_email)} " *"</label>
                                <input type="email" name="email" required
                                       placeholder=move || String::from("") /* was: t!(i18n, register_email_tip) */
                                       class="form-input " autocomplete="email"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_password)} " *"</label>
                                <input type="password" name="password" required
                                       placeholder=move || String::from("") /* was: t!(i18n, register_password_tip) */
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_password2)} " *"</label>
                                <input type="password" name="confirm_password" required
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                        </div>
                        <div>
                            <label class="form-label">{move || t!(i18n, register_topics)}</label>
                            <input type="text" name="topics"
                                   placeholder=move || String::from("") /* was: t!(i18n, register_topics_tip) */
                                   class="form-input "/>
                        </div>
                        <div>
                            <label class="form-label">{move || t!(i18n, register_intro)}</label>
                            <textarea name="introduction" rows="4"
                                      placeholder=move || String::from("") /* was: t!(i18n, register_intro_tip) */
                                      class="form-input "/>
                        </div>

                        {move || action.value().get().and_then(|r| r.err()).map(|e| {
                            let msg = if e.to_string().contains("register_exist") {
                                String::from("") /* was: t!(i18n, register_exist) */
                            } else { e.to_string() };
                            view! { <p class="text-red-500 text-sm text-center">{msg}</p> }
                        })}

                        <button type="submit"
                                disabled=move || action.pending().get()
                                class="btn-primary w-full justify-center">
                            {move || t!(i18n, register)}
                        </button>
                    </ActionForm>
                    </div>

                    <p class="mt-4 text-sm text-center text-gray-500">
                        {move || t!(i18n, register_have_account)} " "
                        <a href="/sign-in" class=format!("text-blue-500 {}", HOVER_UNDERLINE)>{move || t!(i18n, register_go_sign_in)}</a>
                    </p>
                </Show>
            </div>
        </main>
        <Footer/>
    }
}

// ── Activation page component ─────────────────────────────────────────────────

#[component]
pub fn UserActivatePage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_params_map();
    let user_id = move || params.read().get("id").unwrap_or_default();

    let activate_res = Resource::new_blocking(
        move || user_id(),
        |id| async move { activate_user(id).await },
    );

    let resend_action = ServerAction::<ResendActivation>::new();

    view! {
        <Title text=move || page_title!(i18n, user_activate)/>
        <Nav/>
        <main class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="card p-8 w-full max-w-md text-center">
                <h1 class=H1>
                    {move || t!(i18n, user_activate)}
                </h1>
                <Suspense fallback=|| view! { <p class="text-gray-400">"Activating..."</p> }>
                    {move || activate_res.get().map(|result| match result {
                        Err(e) => Either3::Left(view! { <p class="text-red-500">{e.to_string()}</p> }),
                        Ok(None) => Either3::Right(Either::Left(view! { <p class="text-gray-500">"User not found."</p> })),
                        Ok(Some(username)) => Either3::Right(Either::Right(view! {
                            <div class="space-y-4">
                                <div class="text-5xl">"✅"</div>
                                <p class="text-green-600 dark:text-green-400 font-semibold text-lg">
                                    {username} " — " {move || t!(i18n, user_activated)}
                                </p>
                                <a href="/sign-in" class="btn-primary inline-block">
                                    {move || t!(i18n, sign_in)}
                                </a>
                            </div>
                        })),
                    })}
                </Suspense>

                <div class="mt-8 border-t border-gray-100 dark:border-gray-700 pt-6">
                    <p class="text-sm text-gray-500 mb-3">{move || t!(i18n, resend_activation)}</p>
                    <div class="space-y-3">
                    <ActionForm action=resend_action>
                        <input type="hidden" name="user_id" value=move || user_id()/>
                        <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                        <button type="submit"
                                disabled=move || resend_action.pending().get()
                                class="btn-secondary w-full justify-center text-sm">
                            {move || t!(i18n, resend_activation)}
                        </button>
                    </ActionForm>
                    </div>
                    {move || resend_action.value().get().map(|r| match r {
                        Ok(()) => Either::Left(view! {
                            <p class="text-green-500 text-sm mt-2">{move || t!(i18n, user_re_activate)}</p>
                        }),
                        Err(e) => Either::Right(view! {
                            <p class="text-red-500 text-sm mt-2">{e.to_string()}</p>
                        }),
                    })}
                </div>

                <p class="text-xs text-gray-400 mt-6">
                    {move || t!(i18n, user_activate_problem)}
                </p>
            </div>
        </main>
        <Footer/>
    }
}
