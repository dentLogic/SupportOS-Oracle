//! The persistent UI shell: sidebar navigation, routed content, version.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;
use wasm_bindgen::prelude::*;

use crate::nav::NAV_ITEMS;
use crate::pages::{NotBuiltYet, NotFound};

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
