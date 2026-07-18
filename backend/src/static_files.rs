use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir, File};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = ASSETS.get_file(path) {
        return file_response(file);
    }

    match ASSETS.get_file("index.html") {
        Some(file) => file_response(file),
        None => (
            StatusCode::NOT_FOUND,
            "frontend not built: run `npm run build` in frontend/ then rebuild the backend",
        )
            .into_response(),
    }
}

fn file_response(file: &'static File<'static>) -> Response {
    let mime = mime_guess::from_path(file.path()).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(file.contents()))
        .expect("valid static response")
}
