use crate::i18n::t;
use crate::i18n::use_i18n;
use crate::shared::constant::{BG_CARD, TEXT_SUBTLE, TEXT_WARN, WIDE};
use leptos::prelude::*;
#[cfg(all(feature = "oth", feature = "hydrate"))]
use leptos_router::hooks::use_location;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

// ── AdSense push（仅 hydrate 端） ───────────────────────────────────────────────

#[cfg(feature = "hydrate")]
#[wasm_bindgen(inline_js = r#"
    export function push_ad_unit() {
        setTimeout(function() {
            try {
                (window.adsbygoogle = window.adsbygoogle || []).push({});
            } catch(e) {}
        }, 0);
    }
"#)]
extern "C" {
    fn push_ad_unit();
}

// ── AdBanner: 随路由切换销毁旧 <ins>、创建新 <ins> 并触发 AdSense 填充 ─────

#[cfg(feature = "oth")]
#[component]
pub fn AdBanner(
    /// 外部触发信号（如面板打开），变化时重建广告位
    #[prop(into, optional)]
    trigger: Option<Signal<u32>>,
) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    let location = use_location();
    let counter = RwSignal::new(0u32);
    #[cfg(feature = "hydrate")]
    let is_first = RwSignal::new(true);

    // 路由变化时递增 counter → inner_html 强制重建 <ins> DOM
    // 跳过首次挂载（SSR 已由 AdSense 库自动填充）
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let _ = location.pathname.get();
        if is_first.get() {
            is_first.set(false);
        } else {
            counter.update(|n| *n += 1);
            push_ad_unit();
        }
    });

    // 外部 trigger 触发重建（如详情面板打开）
    #[cfg(feature = "hydrate")]
    if let Some(t) = trigger {
        let is_first_t = RwSignal::new(true);
        Effect::new(move |_| {
            let _ = t.get();
            if is_first_t.get() {
                is_first_t.set(false);
            } else {
                counter.update(|n| *n += 1);
                push_ad_unit();
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = &trigger;

    // counter 嵌入 data-refresh 使每次 inner_html 字符串不同，
    // Leptos 检测到差异后替换 innerHTML → 销毁旧 <ins>、创建新 <ins>
    let ad_html = move || {
        let c = counter.get();
        format!(
            r#"<ins class="adsbygoogle" style="display:block" data-ad-client="ca-pub-2498669832870483" data-ad-slot="3837498575" data-ad-format="auto" data-full-width-responsive="true" data-refresh="{}"></ins>"#,
            c
        )
    };

    view! {
        <div class="ad-container overflow-x-hidden max-w-full" inner_html=ad_html></div>
    }
}

// ── Footer ─────────────────────────────────────────────────────────────────────

#[component]
pub fn Footer() -> impl IntoView {
    let i18n = use_i18n();

    #[cfg(feature = "oth")]
    let google_ads = view! { <AdBanner/> };
    #[cfg(not(feature = "oth"))]
    let google_ads = ();

    #[cfg(feature = "oth")]
    let beian = ();
    #[cfg(not(feature = "oth"))]
    let beian = view! {
        <small class="text-xs text-gray-500">
            "琼ICP备2024032236号-14"
            " · "
            "琼公网安备46020002000431号"
        </small>
    };

    view! {
        <footer class={["border-t border-gray-200 dark:border-gray-700", BG_CARD].join(" ")}>
            <div class={[WIDE, "text-center text-sm", TEXT_SUBTLE, "space-y-2"].join(" ")}>
                <p class=TEXT_WARN>
                    {move || t!(i18n, site_warn)}
                </p>
                <a href="https://irust.net" target="_blank">
                    {move || t!(i18n, based_on)}
                </a>
                {google_ads}
                <p>
                    {move || t!(i18n, site_name)}
                    " ©2024-2026 "
                    {move || t!(i18n, copyright)}
                </p>
                {beian}
            </div>
        </footer>
    }
}
