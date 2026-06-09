use tauri::{AppHandle, Manager};
use crate::config_store::ConfigStore;

#[tauri::command]
pub async fn set_mitm_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    profile.mitm.enabled = enabled;
    store.save_profile(&profile)?;
    Ok(())
}

#[tauri::command]
pub async fn get_mitm_hostnames(app: AppHandle) -> Result<Vec<String>, String> {
    let store = app.state::<ConfigStore>();
    let profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    Ok(profile.mitm.hostname_list)
}

#[tauri::command]
pub async fn add_mitm_hostname(app: AppHandle, hostname: String) -> Result<Vec<String>, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let hostname = hostname.trim().to_string();
    if hostname.is_empty() {
        return Err("主机名不能为空".to_string());
    }

    if !profile.mitm.hostname_list.contains(&hostname) {
        profile.mitm.hostname_list.push(hostname);
    }

    store.save_profile(&profile)?;
    Ok(profile.mitm.hostname_list)
}

#[tauri::command]
pub async fn remove_mitm_hostname(app: AppHandle, hostname: String) -> Result<Vec<String>, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    profile.mitm.hostname_list.retain(|h| h != &hostname);

    store.save_profile(&profile)?;
    Ok(profile.mitm.hostname_list)
}
