use leptos::prelude::*;

/// Google AdSense display unit.
///
/// `slot`   — data-ad-slot (from AdSense dashboard)
/// `format` — "auto" (default), "rectangle", "horizontal", "vertical"
/// `class`  — extra CSS classes
#[component]
pub fn AdUnit(
    #[prop(into)] slot: String,
    #[prop(default = "auto".into())] format: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let layout_key = match format.as_str() {
        "auto" => Some("-fb+5w+4e-db+86"),
        _ => None,
    };
    let extra_class = class.unwrap_or_default();

    view! {
        <ins
            class=format!("adsbygoogle {extra_class}")
            style="display:block"
            data-ad-format=format.clone()
            data-ad-layout-key=layout_key
            data-ad-client="ca-pub-2498669832870483"
            data-ad-slot=slot
        />
    }
}
