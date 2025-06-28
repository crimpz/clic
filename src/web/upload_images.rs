use crate::AppState;
use crate::Ctx;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::Multipart;
use axum_extra::typed_header::TypedHeader;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn upload_image(
    ctx: Ctx,
    State(state): State<AppState>,
    TypedHeader(_cookies): TypedHeader<headers::Cookie>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = ctx.user_id();
    let mut message_id: Option<i64> = None;
    let mut original_file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut image_bytes: Option<bytes::Bytes> = None;

    tracing::debug!("UPLOAD IMAGE: Received request to save image");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("message_id") => {
                let val = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                message_id = val.parse::<i64>().ok();
                tracing::debug!("UPLOAD IMAGE: Message Id {:?}", &message_id);
            }
            Some("file") => {
                original_file_name = field.file_name().map(String::from);
                tracing::debug!("UPLOAD IMAGE: Original file name {:?}", &original_file_name);
                content_type = field
                    .content_type()
                    .map(|s| s.to_string())
                    .or(Some("application/octet-stream".to_string()));
                image_bytes = Some(field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    tracing::debug!("UPLOAD IMAGE: Multipart and Message Id parsed");

    let (Some(mid), Some(bytes), Some(file_name)) = (message_id, image_bytes, original_file_name)
    else {
        return Ok((StatusCode::BAD_REQUEST, "Missing file or metadata").into_response());
    };

    tracing::debug!("UPLOAD IMAGE: Metadata received");

    let uuid = Uuid::new_v4();

    let path_buf = PathBuf::from(&file_name);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let new_filename = format!("{}.{}", uuid, ext);
    let storage_path = format!("uploads/images/{}", new_filename);

    let mut file = tokio::fs::File::create(&storage_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.write_all(&bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!(
        "Saving image to server: filename = {}, storage_path = {}",
        &new_filename,
        &storage_path
    );

    sqlx::query(
        r#"
        INSERT INTO images (id, message_id, user_id, filename, content_type, storage_path)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(uuid)
    .bind(mid)
    .bind(user_id)
    .bind(&file_name)
    .bind(content_type.unwrap_or("application/octet-stream".into()))
    .bind(&storage_path)
    .execute(state.mm.db())
    .await
    .map_err(|err| {
        eprintln!("DB insert error: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::OK, "Image uploaded successfully").into_response())
}
