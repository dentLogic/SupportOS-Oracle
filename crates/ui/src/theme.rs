//! The persisted theme preference: cycle order, command arguments, and
//! applying the choice to the document root.
//!
//! The names match the settings store's boundary exactly (core validates
//! both directions). `system` clears the document attribute so the CSS
//! media query rules; `light` and `dark` pin the palette tokens regardless
//! of the system preference.

use leptos::prelude::document;
use wasm_bindgen::prelude::JsValue;

/// The theme names the settings store accepts, in cycle order.
pub const NAMES: &[&str] = &["system", "light", "dark"];

/// The document-root attribute the CSS token overrides key off.
const THEME_ATTRIBUTE: &str = "data-theme";

/// Returns the theme that follows `current` in the cycle.
///
/// An unknown name falls back to `system`, so a value the store would
/// reject cannot loop forever: one click lands on a valid name.
pub fn next(current: &str) -> &'static str {
    let found = NAMES.iter().position(|name| *name == current);
    let following = found.map_or(0, |position| (position + 1) % NAMES.len());
    NAMES[following]
}

/// Builds the invoke arguments for the `settings_set_theme` command.
pub fn command_args(theme: &str) -> Result<JsValue, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        args.as_ref(),
        &JsValue::from_str("theme"),
        &JsValue::from_str(theme),
    )
    .map_err(|error| format!("could not build the command arguments: {error:?}"))?;
    Ok(args.into())
}

/// Applies `theme` to the document root so the token overrides engage.
pub fn apply(theme: &str) -> Result<(), String> {
    let root: web_sys::Element = document()
        .document_element()
        .ok_or_else(|| "the document has no root element".to_string())?;
    if theme == "system" {
        root.remove_attribute(THEME_ATTRIBUTE)
            .map_err(|error| format!("could not clear the theme: {error:?}"))
    } else {
        root.set_attribute(THEME_ATTRIBUTE, theme)
            .map_err(|error| format!("could not apply the theme {theme}: {error:?}"))
    }
}

/// Renders a rejected command as a readable message.
pub fn error_message(context: &str, error: &JsValue) -> String {
    let reason = error.as_string().unwrap_or_else(|| format!("{error:?}"));
    format!("{context}: {reason}")
}
