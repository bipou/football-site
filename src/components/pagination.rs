use crate::models::PageInfo;
use leptos::prelude::*;

/// Cursor-based (actually page-number-based) pagination bar.
/// `base_url`: the URL prefix to append `?from=N` to, e.g. "/footballs"
#[component]
pub fn Pagination(page_info: PageInfo, base_url: String) -> impl IntoView {
    let pi = page_info.clone();
    let base = base_url.clone();

    if pi.total_pages <= 1 {
        return view! { <div/> }.into_any();
    }

    let prev_url = format!("{}?from={}", base, pi.current_page.saturating_sub(1).max(1));
    let next_url = format!("{}?from={}", base, pi.current_page + 1);

    view! {
        <nav class="flex items-center justify-between mt-8 px-4">
            <div class="text-sm text-gray-500 dark:text-gray-400">
                {format!("Page {} / {} — {} total", pi.current_page, pi.total_pages, pi.total_count)}
            </div>
            <div class="flex gap-2">
                {if pi.has_previous {
                    view! {
                        <a href=prev_url class="btn-secondary text-sm">
                            "← Previous"
                        </a>
                    }.into_any()
                } else {
                    view! {
                        <span class="btn bg-gray-50 dark:bg-gray-800 text-gray-300 dark:text-gray-600 cursor-not-allowed text-sm">
                            "← Previous"
                        </span>
                    }.into_any()
                }}
                {if pi.has_next {
                    view! {
                        <a href=next_url class="btn-secondary text-sm">
                            "Next →"
                        </a>
                    }.into_any()
                } else {
                    view! {
                        <span class="btn bg-gray-50 dark:bg-gray-800 text-gray-300 dark:text-gray-600 cursor-not-allowed text-sm">
                            "Next →"
                        </span>
                    }.into_any()
                }}
            </div>
        </nav>
    }.into_any()
}
