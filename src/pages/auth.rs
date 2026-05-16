use crate::i18n::{t, t_string, use_i18n};
use crate::page_title;
use leptos::either::Either;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{Footer, Nav};
use crate::utils::constant::{GRID_2, H1, HOVER_UNDERLINE, TEXT_SUBTLE};

// ── Type alias ───────────────────────────────────────────────────────────
type Either5<A, B, C, D, E> = Either<A, Either<B, Either<C, Either<D, E>>>>;
type Either3<A, B, C> = Either<A, Either<B, C>>;

// ── Topic input component ────────────────────────────────────────────────

#[component]
fn TopicInput() -> impl IntoView {
    let (topics, set_topics) = signal(Vec::<String>::new());
    let (input, set_input) = signal(String::new());
    let input_ref = NodeRef::<Input>::new();

    let add = move |name: &str| {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return;
        }
        set_topics.update(|v| {
            if !v.contains(&name) {
                v.push(name);
            }
        });
        set_input.set(String::new());
    };

    let remove = move |i: usize| {
        set_topics.update(|v| {
            v.remove(i);
        });
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
        "Enter" | "," | " " => {
            ev.prevent_default();
            add(&input.get());
        }
        "Backspace" => {
            if input.get().is_empty() {
                set_topics.update(|v| {
                    v.pop();
                });
            }
        }
        _ => {}
    };

    let csv = move || topics.get().join(",");

    view! {
        <label class="form-label">{move || t!(use_i18n(), register_topics)}</label>
        <div class="form-input flex flex-wrap items-center gap-1 cursor-text"
             on:click=move |_| {
                 if let Some(el) = input_ref.get() {
                     let _ = el.focus();
                 }
             }>
            {move || topics.get().iter().enumerate().map(|(i, t)| {
                let t = t.clone();
                view! {
                    <span class="badge-blue inline-flex items-center gap-1 text-xs">
                        {t}
                        <button type="button"
                            class="ml-0.5 text-blue-500 hover:text-red-500 font-bold leading-none cursor-pointer border-0 bg-transparent p-0 text-base"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                remove(i);
                            }>
                            "×"
                        </button>
                    </span>
                }
            }).collect::<Vec<_>>()}
            <input
                type="text"
                node_ref=input_ref
                class="border-0 outline-none flex-1 min-w-24 bg-transparent text-sm"
                placeholder="..."
                on:keydown=on_keydown
                on:input=move |ev| set_input.set(event_target_value(&ev))
                prop:value=input
            />
        </div>
        <input type="hidden" name="topics" prop:value=csv/>
    }
}

// ── Sign In server function ───────────────────────────────────────────────────

#[server]
pub async fn sign_in(
    signature: String,
    password: String,
    captcha_token: String,
    captcha_answer: String,
) -> Result<(), ServerFnError> {
    use crate::server::{auth as auth_mod, captcha, user_db};
    use axum::http::{HeaderValue, header};

    if captcha::verify_token(&captcha_token, &captcha_answer).is_none() {
        return Err(ServerFnError::new("captcha_invalid"));
    }

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
    confirm_password: String,
    introduction: String,
    topics: String,
    lang: String,
    captcha_token: String,
    captcha_answer: String,
) -> Result<(), ServerFnError> {
    use crate::server::{captcha, email as email_mod, user_db};
    use crate::utils::common::{into_rid, record_key};

    if captcha::verify_token(&captcha_token, &captcha_answer).is_none() {
        return Err(ServerFnError::new("captcha_invalid"));
    }

    if password != confirm_password {
        return Err(ServerFnError::new("register_password_mismatch"));
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_upper || !has_lower || !has_digit {
        return Err(ServerFnError::new("register_password_weak"));
    }

    let data = user_db::RegisterData {
        username,
        email,
        password,
        introduction,
        topics,
    };

    let (user_id, username) = user_db::register_user(data)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_rid = into_rid(&user_id, "users");
    if let Ok(Some((email_addr, _))) = user_db::get_user_email_username(&user_rid).await {
        let kid = record_key(&user_id);
        let _ = email_mod::send_activation_email(&lang, &username, &kid, &email_addr).await;
    }

    Ok(())
}

// ── Activate / Resend server functions ────────────────────────────────────────

#[server]
pub async fn activate_user(user_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::server::user_db;
    use crate::utils::common::into_rid;
    let rid = into_rid(&user_id, "users");
    user_db::activate_user(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn resend_activation(user_id: String, lang: String) -> Result<(), ServerFnError> {
    use crate::server::{email as email_mod, user_db};
    use crate::utils::common::into_rid;
    let rid = into_rid(&user_id, "users");
    let (email, username) = user_db::get_user_email_username(&rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("User not found".to_string()))?;
    email_mod::send_activation_email(&lang, &username, &user_id, &email)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Captcha server function ────────────────────────────────────────────────────

#[server]
pub async fn get_captcha() -> Result<(String, String, u8), ServerFnError> {
    use crate::server::captcha;
    let c = captcha::generate_captcha();
    Ok((c.svg, c.token, c.answer))
}

// ── Captcha gate（页面加载即现，按钮灰色，答对恢复）─────────────────────────

fn check_answer(input: &str, answer: u8) -> bool {
    input.parse::<u8>().ok() == Some(answer)
}

#[component]
fn CaptchaGate(
    children: Children,
    button_label: Signal<String>,
    pending_label: Signal<String>,
    action: ServerAction<SignIn>,
) -> impl IntoView {
    let i18n = use_i18n();
    let (captcha_ok, set_captcha_ok) = signal(false);
    let (status_msg, set_status_msg) = signal(String::new());
    let answer_ref = NodeRef::<Input>::new();

    let captcha_res = Resource::new(|| (), |_| async move { get_captcha().await.ok() });

    let svg = move || {
        captcha_res
            .get()
            .flatten()
            .map(|(s, _, _)| s)
            .unwrap_or_default()
    };
    let token = move || {
        captcha_res
            .get()
            .flatten()
            .map(|(_, t, _)| t)
            .unwrap_or_default()
    };
    let answer = move || captcha_res.get().flatten().map(|(_, _, a)| a).unwrap_or(0);

    let on_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        let ans = answer();
        if val.is_empty() {
            set_status_msg.set(String::new());
            set_captcha_ok.set(false);
        } else if check_answer(&val, ans) {
            set_status_msg.set("✓".into());
            set_captcha_ok.set(true);
        } else {
            set_status_msg.set("✗".into());
            set_captcha_ok.set(false);
        }
    };

    view! {
        <ActionForm action=action>
            {children()}

            <div class="space-y-3 border-t pt-4 mt-4">
                <label class="form-label">{move || t!(i18n, captcha_label)}</label>
                <div class="flex items-center gap-2">
                    <div class="rounded overflow-hidden cursor-pointer shrink-0" style="width:160px;height:40px;border:1px solid #d1d5db"
                         inner_html=svg on:click=move |_| captcha_res.refetch() />
                    <input type="text" name="captcha_answer" required node_ref=answer_ref
                           placeholder="?" class="form-input w-16 text-center text-xl" on:input=on_input />
                    <button type="button"
                            class="text-blue-500 hover:text-blue-700 text-lg font-bold shrink-0 leading-none"
                            title="换一个"
                            on:click=move |_| { captcha_res.refetch(); set_captcha_ok.set(false); set_status_msg.set(String::new()); }>
                        "↻"
                    </button>
                    <span class=move || if captcha_ok.get() { "text-green-500 font-bold text-sm" }
                                         else if status_msg.get().is_empty() { "text-gray-300 text-sm" }
                                         else { "text-red-400 text-sm" }>
                        {move || status_msg.get()}
                    </span>
                </div>
                <input type="hidden" name="captcha_token" value=token />
            </div>

            <button type="submit"
                disabled=move || !captcha_ok.get() || action.pending().get()
                class=move || if captcha_ok.get() {
                    "btn-primary w-full justify-center mt-4".to_string()
                } else {
                    "w-full justify-center bg-gray-300 text-gray-500 rounded-lg py-2 px-4 cursor-not-allowed mt-4".to_string()
                }
            >
                {move || if action.pending().get() { pending_label.get() } else { button_label.get() }}
            </button>

            // Error
            {move || action.value().get().and_then(|r| r.err()).map(|e| {
                let raw = e.to_string();
                if raw.contains("captcha_invalid") {
                    Either5::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, captcha_invalid)}</p> })
                } else if raw.contains("sign_in_incorrect") || raw.contains("register_exist") {
                    Either5::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_incorrect)}</p> }))
                } else if raw.contains("sign_in_not_activation") || raw.contains("register_password_mismatch") {
                    Either5::Right(Either::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_not_activation)}</p> })))
                } else if raw.contains("sign_in_banned") || raw.contains("register_password_weak") {
                    Either5::Right(Either::Right(Either::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_banned)}</p> }))))
                } else {
                    Either5::Right(Either::Right(Either::Right(Either::Right(view! { <p class="text-red-500 text-sm text-center">{raw}</p> }))))
                }
            })}
        </ActionForm>
    }
}

#[component]
fn CaptchaGateRegister(children: Children, action: ServerAction<Register>) -> impl IntoView {
    let i18n = use_i18n();
    let (captcha_ok, set_captcha_ok) = signal(false);
    let (status_msg, set_status_msg) = signal(String::new());
    let answer_ref = NodeRef::<Input>::new();
    let btn = Signal::derive(move || t_string!(i18n, register).to_string());

    let captcha_res = Resource::new(|| (), |_| async move { get_captcha().await.ok() });

    let svg = move || {
        captcha_res
            .get()
            .flatten()
            .map(|(s, _, _)| s)
            .unwrap_or_default()
    };
    let token = move || {
        captcha_res
            .get()
            .flatten()
            .map(|(_, t, _)| t)
            .unwrap_or_default()
    };
    let answer = move || captcha_res.get().flatten().map(|(_, _, a)| a).unwrap_or(0);

    let on_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        let ans = answer();
        if val.is_empty() {
            set_status_msg.set(String::new());
            set_captcha_ok.set(false);
        } else if check_answer(&val, ans) {
            set_status_msg.set("✓".into());
            set_captcha_ok.set(true);
        } else {
            set_status_msg.set("✗".into());
            set_captcha_ok.set(false);
        }
    };

    view! {
        <ActionForm action=action>
            {children()}

            <div class="space-y-3 border-t pt-4 mt-4">
                <label class="form-label">{move || t!(i18n, captcha_label)}</label>
                <div class="flex items-center gap-2">
                    <div class="rounded overflow-hidden cursor-pointer shrink-0" style="width:160px;height:40px;border:1px solid #d1d5db"
                         inner_html=svg on:click=move |_| captcha_res.refetch() />
                    <input type="text" name="captcha_answer" required node_ref=answer_ref
                           placeholder="?" class="form-input w-16 text-center text-xl" on:input=on_input />
                    <button type="button"
                            class="text-blue-500 hover:text-blue-700 text-lg font-bold shrink-0 leading-none"
                            title="换一个"
                            on:click=move |_| { captcha_res.refetch(); set_captcha_ok.set(false); set_status_msg.set(String::new()); }>
                        "↻"
                    </button>
                    <span class=move || if captcha_ok.get() { "text-green-500 font-bold text-sm" }
                                         else if status_msg.get().is_empty() { "text-gray-300 text-sm" }
                                         else { "text-red-400 text-sm" }>
                        {move || status_msg.get()}
                    </span>
                </div>
                <input type="hidden" name="captcha_token" value=token />
            </div>

            <button type="submit"
                disabled=move || !captcha_ok.get() || action.pending().get()
                class=move || if captcha_ok.get() {
                    "btn-primary w-full justify-center mt-4".to_string()
                } else {
                    "w-full justify-center bg-gray-300 text-gray-500 rounded-lg py-2 px-4 cursor-not-allowed mt-4".to_string()
                }
            >
                {move || if action.pending().get() { btn.get() } else { btn.get() }}
            </button>

            // Error
            {move || action.value().get().and_then(|r| r.err()).map(|e| {
                let raw = e.to_string();
                if raw.contains("captcha_invalid") {
                    Either5::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, captcha_invalid)}</p> })
                } else if raw.contains("sign_in_incorrect") || raw.contains("register_exist") {
                    Either5::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_incorrect)}</p> }))
                } else if raw.contains("sign_in_not_activation") || raw.contains("register_password_mismatch") {
                    Either5::Right(Either::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_not_activation)}</p> })))
                } else if raw.contains("sign_in_banned") || raw.contains("register_password_weak") {
                    Either5::Right(Either::Right(Either::Right(Either::Left(view! { <p class="text-red-500 text-sm text-center">{move || t!(i18n, sign_in_banned)}</p> }))))
                } else {
                    Either5::Right(Either::Right(Either::Right(Either::Right(view! { <p class="text-red-500 text-sm text-center">{raw}</p> }))))
                }
            })}
        </ActionForm>
    }
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

    let btn = Signal::derive(move || t_string!(i18n, sign_in).to_string());
    let pending = Signal::derive(move || "Signing in...".to_string());

    view! {
        <Title text=move || page_title!(i18n, user_sign_in)/>
        <Nav/>
        <main class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="card p-8 w-full max-w-sm">
                <h1 class=format!("{} text-center", H1)>
                    {move || t!(i18n, user_sign_in)}
                </h1>

                <div class="space-y-4">
                    <CaptchaGate button_label=btn pending_label=pending action=action>
                        <div>
                            <label class="form-label">{move || t!(i18n, sign_in_account)}</label>
                            <input type="text" name="signature" required
                                   class="form-input " autocomplete="username"/>
                        </div>
                        <div>
                            <label class="form-label">{move || t!(i18n, sign_in_password)}</label>
                            <input type="password" name="password" required
                                   class="form-input " autocomplete="current-password"/>
                        </div>
                    </CaptchaGate>
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
                    <CaptchaGateRegister action=action>
                        <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                        <div class=GRID_2>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_username)} " *"</label>
                                <input type="text" name="username" required
                                       class="form-input " pattern="[a-z0-9_-]+" autocomplete="username"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_email)} " *"</label>
                                <input type="email" name="email" required
                                       class="form-input " autocomplete="email"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_password)} " *"</label>
                                <input type="password" name="password" required
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                            <div>
                                <label class="form-label">{move || t!(i18n, register_confirm_password)} " *"</label>
                                <input type="password" name="confirm_password" required
                                       class="form-input " autocomplete="new-password"/>
                            </div>
                        </div>
                        <div class="space-y-4 mt-4">
                        <div>
                            <TopicInput/>
                        </div>
                        <div>
                            <label class="form-label">
                                {move || t!(i18n, register_intro)}
                                <span class="text-xs text-gray-400 ml-1">
                                    {move || if i18n.get_locale().to_string() == "zh" { "（支持markdown语法）" } else { "(supports Markdown syntax)" }}
                                </span>
                            </label>
                            <textarea name="introduction" rows="4" class="form-input">
                                "## About Me \n我关注足球数据与计算。"
                            </textarea>
                        </div>
                        </div>
                    </CaptchaGateRegister>

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
            <div class="card p-8 text-center max-w-md">
                <Suspense fallback=move || view! { <p class="text-gray-400">{move || t!(i18n, loading)}</p> }>
                    {move || activate_res.get().map(|result| match result {
                        Err(e) => Either3::Left(view! {
                            <p class="text-red-500">{e.to_string()}</p>
                        }),
                        Ok(Some(username)) => Either3::Right(Either::Left(view! {
                            <h1 class="text-xl font-bold text-green-600 mb-4">
                                {move || t!(i18n, user_activated)}
                            </h1>
                            <p class="text-gray-600 mb-4">{username}</p>
                            <a href="/sign-in" class="btn-primary">"Sign In →"</a>
                        })),
                        Ok(None) => Either3::Right(Either::Right(view! {
                            <p class="text-red-500 mb-4">{move || t!(i18n, user_activate_problem)}</p>
                            <ActionForm action=resend_action>
                                <input type="hidden" name="user_id" value=user_id/>
                                <input type="hidden" name="lang" value=move || i18n.get_locale().to_string()/>
                                <button type="submit" class="btn-primary">
                                    {move || t!(i18n, resend_activation)}
                                </button>
                            </ActionForm>
                        })),
                    })}
                </Suspense>
            </div>
        </main>
        <Footer/>
    }
}
