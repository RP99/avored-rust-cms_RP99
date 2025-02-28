use std::path::Path;
use std::sync::Arc;

use crate::{
    avored_state::AvoRedState,
    error::Result,
    models::admin_user_model::AdminUserModel,
    providers::avored_session_provider::AvoRedSession,
};
use axum::{extract::State, Json, response::IntoResponse};
use axum::extract::Multipart;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Serialize;
use crate::models::asset_model::CreatableAssetModel;

pub async fn store_asset_handler(
    session: AvoRedSession,
    state: State<Arc<AvoRedState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    println!("->> {:<12} - store_asset_handler", "HANDLER");
    let logged_in_user = match session.get("logged_in_user") {
        Some(logged_in_user) => logged_in_user,
        None => AdminUserModel::default(),
    };
    let mut creatable_asset_model = CreatableAssetModel::default();
    creatable_asset_model.logged_in_username = logged_in_user.email;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_ref() {
            "file" => {
                let s: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();

                creatable_asset_model.file_type = field.content_type().unwrap().to_string();
                let file_name = field.file_name().unwrap().to_string();
                let data = field.bytes().await.unwrap();
                creatable_asset_model.file_size = i64::try_from(data.len()).unwrap_or(0);

                if !file_name.is_empty() {
                    let file_ext = file_name.split(".").last().unwrap_or(".png");
                    let new_file_name = format!("{}.{}", s, file_ext);

                    let file_name = Path::new(&new_file_name).file_name().unwrap();

                    let asset_file = format!("public/upload/{}", new_file_name.clone());
                    creatable_asset_model.file_name = new_file_name.clone();
                    creatable_asset_model.file_path = asset_file;

                    let full_path = Path::new("public").join("upload").join(file_name);
                    tokio::fs::write(full_path, data).await.unwrap();
                }
            },
            &_ => continue
        }
    }

    let asset_model = state.asset_service
        .create_asset(&state.db, creatable_asset_model)
        .await?;

    Ok(Json(asset_model).into_response())
}

#[derive(Serialize)]
pub struct AssetResponseViewModel {
    pub asset_path: String,
}
