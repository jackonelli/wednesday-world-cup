use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub player_id: i32,
    pub bot_name: Option<String>,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthUser {
    pub player_id: i32,
    pub bot_name: Option<String>,
}

/// Admin middleware - checks for ADMIN_SECRET in Authorization header
pub async fn admin_auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let admin_secret = env::var("ADMIN_SECRET").unwrap_or_else(|_| {
        eprintln!("Warning: ADMIN_SECRET not set");
        String::new()
    });

    if admin_secret.is_empty() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match auth_header {
        Some(header) if header == format!("Bearer {}", admin_secret) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// User middleware - validates JWT token and extracts claims
pub async fn user_auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Decode and validate JWT
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        eprintln!("JWT validation error: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Insert the auth user into request extensions
    let auth_user = AuthUser {
        player_id: token_data.claims.player_id,
        bot_name: token_data.claims.bot_name,
    };

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}
