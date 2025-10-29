use crate::AppError;
use axum::{
    Json,
    extract::Request,
    extract::State,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::env;
use tracing::info;

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

/// Login request
#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

/// Login response
#[derive(Serialize)]
pub(crate) struct LoginResponse {
    token: String,
    player_id: i32,
}

/// Login endpoint
pub(crate) async fn login(
    State(pool): State<SqlitePool>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Get user by username
    let user = wwc_db::get_user_by_username(&pool, &login_req.username)
        .await?
        .ok_or_else(|| AppError::Generic("Invalid username or password".to_string()))?;

    // Verify password
    let password_valid = bcrypt::verify(&login_req.password, &user.password_hash)
        .map_err(|_| AppError::Generic("Invalid username or password".to_string()))?;

    if !password_valid {
        return Err(AppError::Generic(
            "Invalid username or password".to_string(),
        ));
    }

    // Generate JWT token (for human user, not a bot)
    let token = generate_jwt_token(user.id, None);

    info!("User '{}' logged in", user.username);

    Ok(Json(LoginResponse {
        token,
        player_id: user.id,
    }))
}

/// Generate a JWT token
fn generate_jwt_token(player_id: i32, bot_name: Option<String>) -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        player_id: i32,
        bot_name: Option<String>,
        exp: usize,
    }

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        player_id,
        bot_name,
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate JWT token")
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
