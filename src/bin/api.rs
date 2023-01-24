use axum::extract::Query;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum::{extract::Path, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;

static FILE_DIR: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/web/dist");

#[tokio::main]
async fn main() {
    let db = Arc::new(
        SqlitePool::connect("sqlite://fcc.db")
            .await
            .expect("Error connecting to database"),
    );

    let last_update = artemis::meta::get_last_update(&db, artemis::meta::UpdateType::Any)
        .await
        .unwrap();
    println!("last update: {:?}", last_update);

    let app = Router::new()
        .route("/api/v1/call/:call_sign", get(get_by_call_sign))
        .route("/api/v1/search", get(search))
        .fallback(static_path)
        .layer(Extension(db));

    println!("binding to 0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn static_path(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match FILE_DIR.get_file(path) {
        None => {
            // Try to serve index.html if the file doesn't exist
            match FILE_DIR.get_file("index.html") {
                Some(file) => {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(
                            axum::http::header::CONTENT_TYPE,
                            axum::http::HeaderValue::from_str(mime::TEXT_HTML_UTF_8.essence_str())
                                .unwrap(),
                        )
                        .body(axum::body::boxed(axum::body::Full::from(file.contents())))
                        .unwrap();
                }
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(axum::body::boxed(axum::body::Empty::new()))
                    .unwrap(),
            }
        }
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(axum::body::boxed(axum::body::Full::from(file.contents())))
            .unwrap(),
    }
}

async fn get_by_call_sign(
    Extension(db): Extension<Arc<SqlitePool>>,
    Path(call_sign): Path<String>,
) -> impl IntoResponse {
    let call_sign = call_sign.to_uppercase();
    Json(serde_json::json!(query_call_sign(&db, call_sign)
        .await
        .unwrap()))
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    call_sign: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    limit: Option<u32>,
}
async fn search(
    Extension(db): Extension<Arc<SqlitePool>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let res = query_search(&db, params).await.unwrap();
    Json(serde_json::json!(res))
}

#[derive(Debug, FromRow, Serialize)]
struct CallSign {
    call_sign: String,
    operator_class: String,
    frn: String,
    first_name: String,
    mi: String,
    last_name: String,
    city: String,
    state: String,
    license_status: String,
    grant_date: String,
    expired_date: String,
    cancellation_date: String,
    call_count: i64,
    call_history: String,
}
async fn query_call_sign(db: &SqlitePool, call_sign: String) -> Result<Vec<CallSign>, sqlx::Error> {
    let query_str = "SELECT amateurs.call_sign, amateurs.operator_class, entities.first_name, entities.mi, entities.last_name, entities.city, entities.state, headers.license_status, headers.grant_date, headers.expired_date, headers.cancellation_date FROM amateurs JOIN entities ON amateurs.unique_system_identifier = entities.unique_system_identifier JOIN headers ON amateurs.unique_system_identifier = headers.unique_system_identifier WHERE amateurs.call_sign = ? ORDER BY headers.grant_date DESC";
    let result = sqlx::query_as::<_, CallSign>(query_str)
        .bind(call_sign)
        .fetch_all(db)
        .await?;
    Ok(result)
}
async fn query_search(
    db: &SqlitePool,
    search_params: SearchParams,
) -> Result<Vec<CallSign>, sqlx::Error> {
    let query_str = r#"SELECT
            amateurs.call_sign,
            amateurs.operator_class,
            entities.frn,
            entities.first_name,
            entities.mi,
            entities.last_name,
            entities.city,
            entities.state,
            headers.license_status,
            MAX(headers.grant_date) AS grant_date,
            headers.expired_date,
            headers.cancellation_date,
            count(amateurs.call_sign) AS call_count,
            group_concat(amateurs.call_sign, ',') AS call_history
        FROM entities
        JOIN amateurs
            ON amateurs.unique_system_identifier = entities.unique_system_identifier
        JOIN headers
            ON amateurs.unique_system_identifier = headers.unique_system_identifier
        WHERE
            entities.unique_system_identifier IN (
                SELECT unique_system_identifier
                FROM entities
                WHERE
                    (?1 IS NULL OR call_sign LIKE ?1)
                    AND (?2 IS NULL OR first_name LIKE ?2)
                    AND (?3 IS NULL OR last_name LIKE ?3)
            )
            AND frn != ''
        GROUP BY entities.frn
        ORDER BY headers.grant_date DESC
        LIMIT ?4"#;

    let result = sqlx::query_as::<_, CallSign>(query_str)
        .bind(search_params.call_sign)
        .bind(search_params.first_name)
        .bind(search_params.last_name)
        .bind(search_params.limit.unwrap_or(20).min(100))
        .fetch_all(db)
        .await?;
    Ok(result)
}
