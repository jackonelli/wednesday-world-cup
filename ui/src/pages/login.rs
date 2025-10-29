use crate::auth::{AuthState, save_auth_to_storage};
use crate::data::login as login_api;
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_sys::console;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth_state = expect_context::<RwSignal<AuthState>>();
    let display_name = expect_context::<RwSignal<Option<String>>>();
    let navigate = use_navigate();

    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_message = RwSignal::new(Option::<String>::None);
    let is_loading = RwSignal::new(false);

    // Redirect if already authenticated
    Effect::new(move |_| {
        if matches!(auth_state.get(), AuthState::Authenticated { .. }) {
            navigate("/app", Default::default());
        }
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let username_val = username.get();
        let password_val = password.get();

        is_loading.set(true);
        error_message.set(None);

        spawn_local(async move {
            console::log_1(&"Attempting login...".into());

            match login_api(&username_val, &password_val).await {
                Ok((token, player_id, user_display_name)) => {
                    console::log_1(&format!("Login successful: {}", user_display_name).into());

                    // Save to localStorage
                    save_auth_to_storage(&token, player_id);

                    // Update display name (separate from auth)
                    display_name.set(Some(user_display_name));

                    // Update auth state
                    auth_state.set(AuthState::Authenticated { token, player_id });

                    // Navigation will happen via Effect above
                }
                Err(e) => {
                    console::error_1(&format!("Login failed: {}", e).into());
                    error_message.set(Some(format!("Login failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-container">
            <h1>"Wednesday World Cup Login"</h1>
            <form on:submit=on_submit>
                <div>
                    <label>"Username:"</label>
                    <input
                        type="text"
                        prop:value=move || username.get()
                        on:input=move |ev| username.set(event_target_value(&ev))
                        prop:disabled=move || is_loading.get()
                        required
                    />
                </div>
                <div>
                    <label>"Password:"</label>
                    <input
                        type="password"
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                        prop:disabled=move || is_loading.get()
                        required
                    />
                </div>
                {move || error_message.get().map(|msg| view! {
                    <div class="error">{msg}</div>
                })}
                <button type="submit" prop:disabled=move || is_loading.get()>
                    {move || if is_loading.get() { "Logging in..." } else { "Login" }}
                </button>
            </form>
        </div>
    }
}
