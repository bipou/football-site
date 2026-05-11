use crate::i18n::t;
use crate::i18n::use_i18n;
use crate::utils::constant::{BG_CARD, TEXT_SUBTLE, WIDE};
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <footer class={format!("mt-16 border-t border-gray-200 dark:border-gray-700 {}", BG_CARD)}>
            <ins
                class="adsbygoogle"
                style="display: block"
                data-ad-client="ca-pub-2498669832870483"
                data-ad-slot="3837498575"
                data-ad-format="auto"
                data-full-width-responsive="true"
            ></ins>
            <script>
                (adsbygoogle = window.adsbygoogle || []).push({});
            </script>
            <div class={format!("{} text-center text-sm {} space-y-2", WIDE, TEXT_SUBTLE)}>
                <p class="text-xs text-red-500">
                    {move || t!(i18n, site_warn)}
                </p>
                <p>
                    <a href="https://irust.net" target="_blank">{move || t!(i18n, based_on)}</a>
                    " - "
                    {move || t!(i18n, site_name)}
                    " ©2024-2026 "
                    {move || t!(i18n, copyright)}
                </p>
                <small class="text-xs text-gray-500">
                    琼ICP备2024032236号-13
                    " · "
                    琼ICP备2024032236号-13
                </small>
            </div>
        </footer>
    }
    .into_any()
}
