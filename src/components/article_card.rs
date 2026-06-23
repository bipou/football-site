use crate::components::football_card::{CatBadge, status_badge, status_class};
use crate::i18n::{t, t_display, use_i18n};
use crate::models::Football;
use crate::shared::constant::{
    FLEX_BETWEEN, HOVER_SHADOW, NO_UNDERLINE, TEXT_SUBTLE, TEXT_XS_MUTED,
};
use crate::shared::fns::get_username_by_id;
use leptos::either::Either;
use leptos::prelude::*;

/// 用户 / AI 分析文章卡片
#[component]
pub fn ArticleCard(football: Football, on_click: Callback<String>) -> impl IntoView {
    let i18n = use_i18n();
    let title = football.title();
    let summary = football.summary.map(|s| {
        let has_cjk = s.chars().any(|c| c as u32 > 0x2000);
        let limit = if has_cjk { 45 } else { 100 };
        if s.chars().count() > limit {
            let mut t: String = s.chars().take(limit).collect();
            t.push_str("...");
            t
        } else {
            s
        }
    });
    let updated = football.updated_at;
    let ana_type = football.ana_type;
    let fid = crate::shared::common::record_key(&football.id).to_string();
    let href = ["/", &i18n.get_locale().to_string(), "/footballs/", &fid].join("");

    let status = football.status;
    let card_class = ["card", "p-4", HOVER_SHADOW, status_class(status), "min-w-0"].join(" ");
    let badge = status_badge(status);
    let badge_or_label = Memo::new(move |_| match (badge.is_empty(), ana_type) {
        (false, _) => badge.to_string(),
        (true, 1) => t_display!(i18n, pre_match_analysis).to_string(),
        (true, 2) => t_display!(i18n, post_match_review).to_string(),
        (true, _) => String::new(),
    });

    // 卡片点击回调：因 on:click 需闭包非 Callback，故每处内联

    // 类别
    let category_name = football.category_name;
    let cat_kid = category_name
        .as_ref()
        .map(|_| crate::shared::common::record_key(&football.category_id).to_string());
    let cat_name = Memo::new(move |_| {
        let loc = i18n.get_locale().to_string();
        category_name.as_ref().and_then(|c| c.get(&loc).cloned())
    });

    // 用户文章：获取作者 user_id
    let user_id = football.article_user_id.clone();

    let author_name = Resource::new(
        move || user_id.clone(),
        |uid| async move {
            match uid {
                Some(id) if !id.is_empty() => get_username_by_id(id).await.ok().flatten(),
                _ => None,
            }
        },
    );

    view! {
        <div class=card_class>
            <div class=[FLEX_BETWEEN, "mb-2"].join(" ")>
                <a
                                    class=["font-semibold text-gray-800 dark:text-gray-100 hover:underline hover:text-blue-600 text-lg leading-tight min-w-0 cursor-pointer p-0 text-left block", NO_UNDERLINE].join(" ")
                                    href=href.clone()
                                    on:click={
                                                                        let f = fid.clone();
                                                                        let cb = on_click.clone();
                                                                        move |ev| {
                                                                            ev.prevent_default();
                                                                            cb.run(f.clone())
                                                                        }
                                                                    }
                                >
                                    {title}
                                </a>
                <span class="text-sm ml-2 whitespace-nowrap">
                    {badge_or_label}
                </span>
            </div>

            <div class=["text-sm", TEXT_SUBTLE, "mb-3", "space-x-2"].join(" ")>
                <CatBadge name=cat_name kid=cat_kid/>
                <span class="text-blue-500">{updated}</span>
            </div>

            {match summary {
                Some(s) => Either::Left(view! {
                    <a class=["cursor-pointer block p-0 text-left w-full", NO_UNDERLINE].join(" ") href=href.clone() on:click={
                                                                let f = fid.clone();
                                                                let cb = on_click.clone();
                                            move |ev| {
                                                ev.prevent_default();
                                                cb.run(f.clone())
                                            }
                                        }>
                        <p class="text-sm text-gray-600 dark:text-gray-400 my-0">{s}</p>
                    </a>
                }),
                None => Either::Right(()),
            }}

            <div class=[FLEX_BETWEEN, "mt-3"].join(" ")>
                <span class=["text-sm", TEXT_XS_MUTED].join(" ")>
                    <Suspense fallback=|| "">
                                            {move || author_name.get().flatten().unwrap_or_default()}
                    </Suspense>
                </span>
                <span class="text-sm text-gray-400">
                    {move || t!(i18n, football_hits)}{football.hits}
                </span>
            </div>
        </div>
    }
}
