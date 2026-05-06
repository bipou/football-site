use crate::i18n::t;
use crate::i18n::use_i18n;
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <footer class="mt-16 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
            <div class="max-w-6xl mx-auto px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400 space-y-2">
                <p class="text-xs text-red-500">
                    {move || t!(i18n, site_warn)}
                </p>
                <p>
                    {move || t!(i18n, based_on)}
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
}
