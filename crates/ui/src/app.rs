//! The persistent UI shell: sidebar navigation, routed content, version.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;
use wasm_bindgen::prelude::*;

use crate::nav::NAV_ITEMS;
use crate::pages::{NotBuiltYet, NotFound};
use crate::theme;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn App() -> impl IntoView {
    let (version, set_version) = signal(String::new());

    // The command-and-event pattern: the UI invokes the shell's one-line
    // command, which wraps a tested core function. A failed or empty answer
    // renders as "unknown" rather than a guess.
    Effect::new(move |_| {
        spawn_local(async move {
            let reported = match invoke("app_version", JsValue::UNDEFINED).await {
                Ok(value) => value.as_string().filter(|reported| !reported.is_empty()),
                Err(_) => None,
            };
            set_version.set(reported.unwrap_or_else(|| "unknown".into()));
        });
    });

    // The persisted theme arrives through the settings command; the toggle
    // shows a value only once the store holds it, and a rejected save leaves
    // both the label and the page on the previous theme (no fake success).
    let (current_theme, set_current_theme) = signal("system".to_string());
    let (theme_save_in_flight, set_theme_save_in_flight) = signal(false);
    let (theme_error, set_theme_error) = signal(None::<String>);

    Effect::new(move |_| {
        spawn_local(async move {
            match invoke("settings_theme", JsValue::UNDEFINED).await {
                Ok(value) => match value.as_string() {
                    Some(stored) => {
                        set_current_theme.set(stored.clone());
                        if let Err(message) = theme::apply(&stored) {
                            set_theme_error.set(Some(message));
                        }
                    }
                    None => set_theme_error.set(Some(
                        "the theme command answered with a non-string value".to_string(),
                    )),
                },
                Err(error) => set_theme_error.set(Some(theme::error_message(
                    "could not read the theme",
                    &error,
                ))),
            }
        });
    });

    let toggle_theme = move |_| {
        if theme_save_in_flight.get_untracked() {
            return;
        }
        let target = theme::next(&current_theme.get_untracked()).to_string();
        set_theme_save_in_flight.set(true);
        spawn_local(async move {
            let args = match theme::command_args(&target) {
                Ok(args) => args,
                Err(message) => {
                    set_theme_error.set(Some(message));
                    set_theme_save_in_flight.set(false);
                    return;
                }
            };
            match invoke("settings_set_theme", args).await {
                Ok(_) => {
                    set_current_theme.set(target.clone());
                    set_theme_error.set(None);
                    if let Err(message) = theme::apply(&target) {
                        set_theme_error.set(Some(message));
                    }
                }
                Err(error) => set_theme_error.set(Some(theme::error_message(
                    "could not save the theme",
                    &error,
                ))),
            }
            set_theme_save_in_flight.set(false);
        });
    };

    view! {
        <Router>
            <div class="shell">
                <nav class="sidebar" aria-label="Main navigation">
                    <header class="brand">
                        <h1>"SupportOS Oracle"</h1>
                        <p class="tagline">"A local-first support desk companion"</p>
                    </header>
                    <ul class="nav-list">
                        {NAV_ITEMS
                            .iter()
                            .map(|item| {
                                view! {
                                    <li>
                                        <A href={item.path} attr:class="nav-link">
                                            {item.label}
                                        </A>
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </ul>
                    <div class="theme-controls">
                        <button
                            class="theme-toggle"
                            title="Cycle the theme: system, light, dark"
                            disabled=move || theme_save_in_flight.get()
                            on:click=toggle_theme
                        >
                            "Theme: " {move || current_theme.get()}
                        </button>
                        {move || {
                            theme_error.get().map(|message| {
                                view! { <p class="theme-error">{message}</p> }
                            })
                        }}
                    </div>
                    <footer class="version">
                        "App version: " {move || version.get()}
                    </footer>
                </nav>
                <main class="content">
                    <Routes fallback=NotFound>
                        <Route path=path!("") view=|| view! { <NotBuiltYet title="Dashboard"/> } />
                        <Route path=path!("inbox") view=|| view! { <NotBuiltYet title="Inbox"/> } />
                        <Route path=path!("notifications") view=|| view! { <NotBuiltYet title="Notifications"/> } />
                        <Route path=path!("search") view=|| view! { <NotBuiltYet title="Search"/> } />
                        <Route path=path!("customers") view=|| view! { <NotBuiltYet title="Customers"/> } />
                        <Route path=path!("organizations") view=|| view! { <NotBuiltYet title="Organizations"/> } />
                        <Route path=path!("ai") view=|| view! { <NotBuiltYet title="AI Center"/> } />
                        <Route path=path!("issues") view=|| view! { <NotBuiltYet title="Issues"/> } />
                        <Route path=path!("incidents") view=|| view! { <NotBuiltYet title="Incidents"/> } />
                        <Route path=path!("knowledge") view=|| view! { <NotBuiltYet title="Knowledge"/> } />
                        <Route path=path!("docs") view=|| view! { <NotBuiltYet title="Docs"/> } />
                        <Route path=path!("custom-objects") view=|| view! { <NotBuiltYet title="Objects"/> } />
                        <Route path=path!("connectors") view=|| view! { <NotBuiltYet title="Connectors"/> } />
                        <Route path=path!("graph") view=|| view! { <NotBuiltYet title="Graph"/> } />
                        <Route path=path!("operations") view=|| view! { <NotBuiltYet title="Operations"/> } />
                        <Route path=path!("reports") view=|| view! { <NotBuiltYet title="Reports"/> } />
                        <Route path=path!("outreach") view=|| view! { <NotBuiltYet title="Outreach"/> } />
                        <Route path=path!("automation") view=|| view! { <NotBuiltYet title="Automation"/> } />
                        <Route path=path!("sync-health") view=|| view! { <NotBuiltYet title="Sync Health"/> } />
                        <Route path=path!("settings") view=|| view! { <NotBuiltYet title="Settings"/> } />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
