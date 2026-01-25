use axum::Json;
use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sanitize_filename::sanitize;
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub(crate) async fn get_all_uploads() -> Result<Json<Vec<String>>, StatusCode> {
    let mut res = vec![];

    match tokio::fs::read_dir(dotenvy::var("UPLOADS_PATH").unwrap()).await {
        Ok(mut dir) => {
            while let Some(entry) = dir.next_entry().await.unwrap() {
                let file_name = entry.file_name();
                let metadata = entry.metadata().await.unwrap();

                if metadata.is_file() {
                    res.push(file_name.into_string().unwrap());
                }
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(res))
}

#[derive(Deserialize)]
pub(crate) struct DownloadParams {
    file_name: String,
}

pub(crate) async fn download_file(Query(params): Query<DownloadParams>) -> impl IntoResponse {
    let file_path = format!(
        "{}/{}",
        dotenvy::var("UPLOADS_PATH").unwrap(),
        sanitize(params.file_name)
    );

    let file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((StatusCode::OK, body))
}
