use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn App() -> impl IntoView {
    let (version, set_version) = signal(String::new());

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
        <main class="container">
            <h1>"SupportOS Oracle"</h1>

            <p>"A local-first support desk companion. Linux only. Early development."</p>

            <p>
                "App version: " {move || version.get()}
            </p>
        </main>
    }
}
