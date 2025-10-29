use crate::auth::{AuthState, ProtectedRoute, load_auth_from_storage};
use crate::pages::{login::LoginPage, predictions_view::PredictionsView};
use leptos::prelude::*;
use leptos_router::{
    StaticSegment,
    components::{Redirect, Route, Router, Routes},
};

#[component]
pub fn App() -> impl IntoView {
    // Initialize auth state
    let auth_state = RwSignal::new(AuthState::Loading);
    provide_context(auth_state);

    // Check localStorage for existing auth on mount
    Effect::new(move |_| {
        if let Some(stored_auth) = load_auth_from_storage() {
            auth_state.set(AuthState::Authenticated {
                token: stored_auth.token,
                player_id: stored_auth.player_id,
                display_name: None, // Will be fetched by PredictionsView
            });
        } else {
            auth_state.set(AuthState::Unauthenticated);
        }
    });

    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=StaticSegment("/login") view=LoginPage/>
                <Route
                    path=StaticSegment("/app")
                    view=|| {
                        view! {
                            <ProtectedRoute>
                                <PredictionsView/>
                            </ProtectedRoute>
                        }
                    }
                />

                <Route
                    path=StaticSegment("/")
                    view=|| {
                        view! { <Redirect path="/app"/> }
                    }
                />

            </Routes>
        </Router>
    }
}
