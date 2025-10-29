use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use web_sys::window;

const STORAGE_KEY: &str = "wwc_auth";

// Auth state enum
#[derive(Clone, Debug, PartialEq)]
pub enum AuthState {
    Loading,
    Unauthenticated,
    Authenticated {
        token: String,
        player_id: i32,
        display_name: Option<String>,
    },
}

// Struct for localStorage serialization
#[derive(Serialize, Deserialize)]
pub struct StoredAuth {
    pub token: String,
    pub player_id: i32,
}

// Load auth from localStorage
pub fn load_auth_from_storage() -> Option<StoredAuth> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    let json_str = storage.get_item(STORAGE_KEY).ok()??;
    serde_json::from_str(&json_str).ok()
}

// Save auth to localStorage
pub fn save_auth_to_storage(token: &str, player_id: i32) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
        let stored = StoredAuth {
            token: token.to_string(),
            player_id,
        };
        if let Ok(json_str) = serde_json::to_string(&stored) {
            let _ = storage.set_item(STORAGE_KEY, &json_str);
        }
    }
}

// Clear auth from localStorage
pub fn clear_auth_from_storage() {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.remove_item(STORAGE_KEY);
    }
}

// Protected route wrapper component
#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_state = expect_context::<RwSignal<AuthState>>();
    let navigate = use_navigate();

    // Redirect to login if unauthenticated
    Effect::new(move |_| {
        if matches!(auth_state.get(), AuthState::Unauthenticated) {
            navigate("/login", Default::default());
        }
    });

    move || match auth_state.get() {
        AuthState::Loading => view! { <div>"Loading..."</div> }.into_any(),
        AuthState::Unauthenticated => view! { <div></div> }.into_any(),
        AuthState::Authenticated { .. } => children().into_any(),
    }
}

// Logout action helper
pub fn logout(auth_state: RwSignal<AuthState>) {
    clear_auth_from_storage();
    auth_state.set(AuthState::Unauthenticated);
}
