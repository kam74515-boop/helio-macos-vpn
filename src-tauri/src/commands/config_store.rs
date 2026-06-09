use tauri::{AppHandle, State, Manager};
use crate::config_store::{ConfigStore, Profile};
use crate::state::AppState;
use crate::commands::singbox::start_engine;

#[tauri::command]
pub async fn list_profiles(app: AppHandle) -> Result<Vec<String>, String> {
    let store = app.state::<ConfigStore>();
    Ok(store.list_profiles())
}

#[tauri::command]
pub async fn get_active_profile(app: AppHandle) -> Result<Profile, String> {
    let store = app.state::<ConfigStore>();
    store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())
}

#[tauri::command]
pub async fn switch_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    store.set_active_profile(&name)?;
    let profile = store.get_active_profile()
        .ok_or_else(|| "切换后无法加载 profile".to_string())?;
    let runtime = store.get_runtime_config(&profile);
    start_engine(app, state, Some(runtime)).await?;
    Ok(())
}

#[tauri::command]
pub async fn save_profile(app: AppHandle, profile: Profile) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    store.save_profile(&profile)?;
    Ok(())
}
