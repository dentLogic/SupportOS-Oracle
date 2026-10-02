//! The pages of the shell.
//!
//! Until a reference page's real screen is built in its own slice, it
//! renders the honest placeholder (docs/SPEC-AMENDMENTS.md, A23): the route
//! opens, states "Not built yet", and carries no fake controls or data.

use leptos::prelude::*;

/// The honest placeholder for a reference page whose slice has not landed.
#[component]
pub fn NotBuiltYet(#[prop(into)] title: String) -> impl IntoView {
    view! {
        <section class="page">
            <h1>{title}</h1>
            <p class="placeholder">"Not built yet"</p>
            <p class="placeholder-note">
                "This page is part of the reference product and arrives in a later slice."
            </p>
        </section>
    }
}

/// The fallback for any route outside the reference inventory.
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="page">
            <h1>"Page not found"</h1>
            <p class="placeholder-note">"The page you asked for does not exist."</p>
        </section>
    }
}
