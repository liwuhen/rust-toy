use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use axum::{async_trait, Json, Router, Server};
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use headers::Authorization;
use headers::authorization::Bearer;
use serde::{Deserialize, Serialize};

use jsonwebtoken as jwt;
use jsonwebtoken::Validation;

const SECRET: &[u8] = b"deadbeef";

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub user_id: usize,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: usize,
    name: String,
    exp: usize,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/todos", get(todos_handler)
            .post(create_todos_handler))
        .route("/login", post(login_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html("Hello world!")
}

async fn todos_handler() -> Json<Vec<Todo>> {
    Json(vec![
        Todo {
            id: 1,
            user_id: 1,
            title: "Todo 1".to_string(),
            completed: false,
        },
        Todo {
            id: 2,
            user_id: 2,
            title: "Todo 2".to_string(),
            completed: false,
        },
    ])
}

async fn create_todos_handler(claims: Claims, Json(_todo): Json<CreateTodo>) -> StatusCode {
    println!("{:?}", claims);
    StatusCode::CREATED
}

async fn login_handler(Json(login): Json<LoginRequest>) -> Json<LoginResponse> {
    let claims = Claims {
        id: 1,
        name: "QuakeWang".to_string(),
        exp: get_epoch() + 14 * 24 * 60 * 60,
    };
    let key = jwt::EncodingKey::from_secret(SECRET);
    let token = jwt::encode(&jwt::Header::default(), &claims, &key).unwrap();
    Json(LoginResponse { token })
}

#[async_trait]
impl<B> FromRequest<B> for Claims
    where B: Send,
{
    type Rejection = HttpError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| HttpError::Auth)?;
        let key = jwt::DecodingKey::from_secret(SECRET);
        let token = jwt::decode::<Claims>(bearer.token(), &key, &Validation::default())
            .map_err(|e| {
                println!("{:?}", e);
                HttpError::Auth
            })?;

        Ok(token.claims)
    }
}

#[derive(Debug)]
pub enum HttpError {
    Auth,
    Internal,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            HttpError::Auth => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            HttpError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        (code, msg).into_response()
    }
}

fn get_epoch() -> usize {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}