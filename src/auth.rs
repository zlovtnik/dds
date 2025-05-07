use async_graphql::{Context, Error, ErrorExtensions, Result};
use async_trait::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::graphql::GraphQLContext;
use crate::models::etl::{DateTimeScalar, UuidScalar};
use crate::models::user::User;

/// Auth provider trait for different authentication backends
#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn login(&self, email: String, password: String) -> Result<AuthResponse>;
    async fn validate_token(&self, token: &str) -> Result<TokenClaims>;
}

/// Auth0/Okta implementation of the auth provider
pub struct Auth0Okta {
    client: Client,
    domain: String,
    client_id: String,
    client_secret: String,
    audience: String,
}

impl Auth0Okta {
    pub fn new() -> Self {
        let domain = env::var("AUTH0_DOMAIN").expect("AUTH0_DOMAIN must be set");
        let client_id = env::var("AUTH0_CLIENT_ID").expect("AUTH0_CLIENT_ID must be set");
        let client_secret =
            env::var("AUTH0_CLIENT_SECRET").expect("AUTH0_CLIENT_SECRET must be set");
        let audience =
            env::var("AUTH0_AUDIENCE").unwrap_or_else(|_| format!("https://{}/api/v2/", domain));

        Self {
            client: Client::new(),
            domain,
            client_id,
            client_secret,
            audience,
        }
    }
}

#[async_trait]
impl AuthProvider for Auth0Okta {
    async fn login(&self, email: String, password: String) -> Result<AuthResponse> {
        tracing::debug!("Attempting login for user: {}", email);

        // First, check if we have all required env variables
        if self.domain.is_empty() || self.client_id.is_empty() || self.client_secret.is_empty() {
            tracing::error!(
                "Auth0/Okta configuration missing: domain={}, client_id={}, audience={}",
                self.domain.is_empty(),
                self.client_id.is_empty(),
                self.audience.is_empty()
            );
            return Err(Error::new("Auth0/Okta configuration is incomplete"));
        }

        // Add more helpful logging
        tracing::debug!("Using Auth0 domain: {}", self.domain);
        tracing::debug!("Using Auth0 audience: {}", self.audience);

        let token_url = format!("https://{}/oauth/token", self.domain);
        tracing::debug!("Requesting token from: {}", token_url);

        let params = [
            ("grant_type", "password"),
            ("username", &email),
            ("password", &password),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("audience", &self.audience),
            ("scope", "openid profile email"),
        ];

        // For development/testing only, create a mock response
        // IMPORTANT: Remove this in production
        if std::env::var("AUTH_MOCK").unwrap_or_default() == "true" {
            tracing::info!("Using mock Auth0 response for development");
            return Ok(AuthResponse {
                token: "mock_jwt_token".to_string(),
                refresh_token: "mock_refresh_token".to_string(),
                user: User {
                    id: UuidScalar(uuid::Uuid::new_v4()),
                    username: "mock_user".to_string(),
                    email: email.clone(),
                    created_at: DateTimeScalar(chrono::Utc::now()),
                    updated_at: DateTimeScalar(chrono::Utc::now()),
                },
            });
        }

        // Send the actual request to Auth0/Okta
        let response = match self.client.post(&token_url).form(&params).send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to send Auth0 request: {}", e);
                return Err(Error::new(format!("Failed to send request: {}", e)));
            }
        };

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Auth0 authentication failed: {}", error_text);
            return Err(Error::new("Authentication failed")
                .extend_with(|_, e| e.set("details", error_text)));
        }

        // Parse the token response
        let token_response: TokenResponse = match response.json().await {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failed to parse Auth0 response: {}", e);
                return Err(Error::new(format!("Failed to parse response: {}", e)));
            }
        };

        tracing::debug!("Successfully obtained token");

        // Get user info
        let user_info_url = format!("https://{}/userinfo", self.domain);
        tracing::debug!("Requesting user info from: {}", user_info_url);

        let user_info_response = match self
            .client
            .get(&user_info_url)
            .bearer_auth(&token_response.access_token)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to get user info: {}", e);
                return Err(Error::new(format!("Failed to get user info: {}", e)));
            }
        };

        if !user_info_response.status().is_success() {
            tracing::error!("Failed to get user info: {}", user_info_response.status());
            return Err(Error::new("Failed to get user info"));
        }

        let user_info: UserInfo = match user_info_response.json().await {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failed to parse user info: {}", e);
                return Err(Error::new(format!("Failed to parse user info: {}", e)));
            }
        };

        tracing::info!("Login successful for user: {}", email);

        Ok(AuthResponse {
            token: token_response.access_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            user: User {
                id: UuidScalar(
                    uuid::Uuid::parse_str(&user_info.sub).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                ),
                username: user_info
                    .nickname
                    .unwrap_or_else(|| user_info.email.clone()),
                email: user_info.email.clone(),
                created_at: DateTimeScalar(chrono::Utc::now()),
                updated_at: DateTimeScalar(chrono::Utc::now()),
            },
        })
    }

    async fn validate_token(&self, token: &str) -> Result<TokenClaims> {
        // Auth0/Okta token validation logic
        // This is a simplified implementation, in a production environment, you would:
        // 1. Fetch the JWKS from Auth0/Okta
        // 2. Find the correct key using the kid in the token header
        // 3. Verify the token signature using that key
        // 4. Validate the token claims (expiration, issuer, audience)

        // For simplicity, we're using a shared secret here
        let secret = env::var("AUTH0_CLIENT_SECRET").expect("AUTH0_CLIENT_SECRET must be set");

        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| Error::new(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub nickname: Option<String>,
    pub email: String,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}

// Helper function to get user id from context
pub fn get_current_user_id(ctx: &Context<'_>) -> Result<Option<UuidScalar>> {
    if let Ok(ctx_data) = ctx.data::<GraphQLContext>() {
        if let Some(user_id) = &ctx_data.current_user_id {
            return Ok(Some(*user_id));
        }
    }
    Ok(None)
}
