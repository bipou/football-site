use crate::i18n::{t, use_i18n};
use crate::models::PageInfo;
use crate::utils::constant::{FLEX_BETWEEN, TEXT_SUBTLE};
use leptos::either::Either;
use leptos::prelude::*;

/// Cursor-based (actually page-number-based) pagination bar.
/// `base_url`: the URL prefix to append `?from=N` to, e.g. "/footballs"
#[component]
pub fn Pagination(page_info: PageInfo, base_url: String) -> impl IntoView {
    let i18n = use_i18n();
    let pi = page_info.clone();
    let base = base_url.clone();

    if pi.total_pages <= 1 {
        return Either::Left(view! { <div/> });
    }

    let prev_url = format!("{}?from={}", base, pi.current_page.saturating_sub(1).max(1));
    let next_url = format!("{}?from={}", base, pi.current_page + 1);

    Either::Right(view! {
        <nav class={format!("{} mt-8 px-4", FLEX_BETWEEN)}>
            <div class={format!("text-sm {}", TEXT_SUBTLE)}>
                <span>"Page " {pi.current_page} " / " {pi.total_pages} " — " {pi.total_count} " "</span>
                {move || t!(i18n, pagination_aggregate)}
            </div>
            <div class="flex gap-2">
                {if pi.has_previous {
                    Either::Left(view! {
                        <a href=prev_url class="btn-secondary text-sm">
                            {move || t!(i18n, pagination_previous)}
                        </a>
                    })
                } else {
                    Either::Right(view! {
                        <span class="btn bg-gray-50 dark:bg-gray-800 text-gray-300 dark:text-gray-600 cursor-not-allowed text-sm">
                            {move || t!(i18n, pagination_previous)}
                        </span>
                    })
                }}
                {if pi.has_next {
                    Either::Left(view! {
                        <a href=next_url class="btn-secondary text-sm">
                            {move || t!(i18n, pagination_next)}
                        </a>
                    })
                } else {
                    Either::Right(view! {
                        <span class="btn bg-gray-50 dark:bg-gray-800 text-gray-300 dark:text-gray-600 cursor-not-allowed text-sm">
                            {move || t!(i18n, pagination_next)}
                        </span>
                    })
                }}
            </div>
        </nav>
    })
}
