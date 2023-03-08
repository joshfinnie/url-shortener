use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
//use base62;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};

#[derive(FromRow, Deserialize, Serialize)]
struct Url {
    id: i64,
    url: String,
    visit: Option<i32>,
}

#[derive(Deserialize, Serialize)]
struct NewUrl {
    url: String,
}

#[derive(Deserialize, Serialize)]
struct ResponseBody<T> {
    data: T,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to load environment variables from .env file");

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await
        .expect("Unable to connect to Postgres");

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/:url_id", get(get_url))
        .route("/api/url", post(new_url))
        .layer(Extension(pool));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn new_url(
    Extension(pool): Extension<PgPool>,
    Json(req): Json<ResponseBody<NewUrl>>,
) -> Result<(StatusCode, Json<ResponseBody<String>>), AppError> {
    let query = "INSERT INTO url (url) values ($1) RETURNING *";
    let url = sqlx::query_as::<_, Url>(&query)
        .bind(req.data.url)
        .fetch_one(&pool)
        .await;
    match url {
        Ok(url) => Ok((
            StatusCode::CREATED,
            Json(ResponseBody {
                data: url.id.to_string(),
            }),
        )),
        Err(_) => Err(AppError::CouldNotCreate),
    }
}

async fn get_url(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ResponseBody<String>>), AppError> {
    let query = "SELECT * FROM url WHERE id=$1";
    let url = sqlx::query_as::<_, Url>(&query)
        .bind(id.parse::<i64>().unwrap())
        .fetch_one(&pool)
        .await;
    match url {
        Ok(url) => Ok((StatusCode::CREATED, Json(ResponseBody { data: url.url }))),
        Err(err) => {
            dbg!(err);
            Err(AppError::CouldNotFetch)
        }
    }
}

#[allow(dead_code)]
enum AppError {
    CouldNotCreate,
    CouldNotFetch,
    TeaPot, // TeaPot is just fun...
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::CouldNotCreate => (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't save URL"),
            AppError::CouldNotFetch => (StatusCode::NOT_FOUND, "Couldn't fetch URL."),
            AppError::TeaPot => (StatusCode::IM_A_TEAPOT, "I'm a teapot."),
        };
        let body = Json(json!({
            "error": message,
        }));
        (status, body).into_response()
    }
}
