use tauri::{AppHandle, State, Manager};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use crate::state::AppState;
use crate::config_store::ConfigStore;
use crate::utils::{run_cmd, run_cmd_stderr};

#[derive(Debug, Serialize)]
pub struct TunStatus {
    pub enabled: bool,      // profile 中是否启用
    pub active: bool,       // 实际 utun 接口是否存在
    pub interface: String,  // 接口名，如 "utun3"
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionStatus {
    pub is_admin: bool,
    pub has_sudo: bool,
    pub has_ne_entitlement: bool,
    pub messages: Vec<String>,
}

/// 从可执行路径推导 .app bundle 路径
fn app_bundle_from_exe(exe_path: &str) -> Option<String> {
    if exe_path.contains(".app/") {
        let app_path = exe_path.split(".app/").next()
            .map(|p| format!("{}.app", p))?;
        Some(app_path)
    } else {
        None
    }
}

#[tauri::command]
pub async fn get_tun_status(app: AppHandle) -> Result<TunStatus, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile();
    let enabled = profile.as_ref().map(|p| p.tun.enabled).unwrap_or(false);

    // 通过 ifconfig 查找 utun 接口
    let ifconfig_out = run_cmd(&["sh", "-c", "ifconfig | grep -E '^utun'"])
        .unwrap_or_default();

    let mut active = false;
    let mut interface = String::new();

    for line in ifconfig_out.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(name) = parts.first() {
            let name = name.trim_end_matches(':');
            if name.starts_with("utun") {
                active = true;
                interface = name.to_string();
                break;
            }
        }
    }

    // 通过 netstat 检查 TUN 路由
    if active {
        let route_out = run_cmd(&["sh", "-c", "netstat -rn | grep utun"])
            .unwrap_or_default();
        if route_out.trim().is_empty() {
            active = false;
        }
    }

    let error = if enabled && !active {
        Some("TUN 已启用但接口未激活".to_string())
    } else {
        None
    };

    Ok(TunStatus {
        enabled,
        active,
        interface,
        error,
    })
}

#[tauri::command]
pub async fn get_permission_status() -> Result<PermissionStatus, String> {
    let mut messages = Vec::new();

    // is_admin: id -Gn 输出包含 "admin"
    let is_admin = match run_cmd(&["id", "-Gn"]) {
        Ok(groups) => groups.split_whitespace().any(|g| g == "admin"),
        Err(_) => false,
    };

    if is_admin {
        messages.push("当前用户是管理员".to_string());
    } else {
        messages.push("当前用户不是管理员".to_string());
    }

    // has_sudo: sudo -n true 返回成功，5 秒超时
    let has_sudo = match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("sudo")
            .args(["-n", "true"])
            .output(),
    ).await {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    if has_sudo {
        messages.push("有 sudo 权限".to_string());
    } else {
        messages.push("无 sudo 权限，启用增强模式需要管理员密码".to_string());
    }

    // has_ne_entitlement: 读取 app bundle 的 entitlements
    let mut has_ne_entitlement = false;
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy();
        if let Some(app_path) = app_bundle_from_exe(&exe_str) {
            if std::path::Path::new(&app_path).exists() {
                let entitlements = run_cmd_stderr(&["codesign", "-d", "--entitlements", "-", &app_path])
                    .unwrap_or_default();
                has_ne_entitlement = entitlements.contains("com.apple.developer.networking.networkextension");
            }
        }
    }

    if has_ne_entitlement {
        messages.push("已检测到 Network Extension entitlement".to_string());
    } else {
        messages.push("未检测到 Network Extension entitlement，正式分发需要申请".to_string());
    }

    Ok(PermissionStatus {
        is_admin,
        has_sudo,
        has_ne_entitlement,
        messages,
    })
}

#[tauri::command]
pub async fn toggle_enhanced_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    enable: bool,
) -> Result<TunStatus, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    // 启用前检查权限
    if enable {
        let perms = get_permission_status().await?;
        if !perms.has_sudo {
            return Err(
                "需要管理员权限才能启用增强模式。请在终端执行 `sudo <app_path>` 启动 Helio，或前往系统偏好设置授权。".to_string()
            );
        }
    }

    // 设置 TUN 开关并保存
    profile.tun.enabled = enable;
    store.save_profile(&profile)?;

    // 生成 runtime config 并写入 config.json
    let runtime = store.get_runtime_config(&profile);
    store.write_runtime_config(&runtime)?;

    // 重启引擎
    crate::commands::singbox::start_engine(app.clone(), state, Some(runtime)).await?;

    // 等待 2 秒让接口创建/销毁
    sleep(Duration::from_secs(2)).await;

    // 检查接口状态
    let status = get_tun_status(app).await?;

    if enable && !status.active {
        return Err("TUN 接口未创建，sing-box 可能未正确启动。请检查日志。".to_string());
    }

    Ok(status)
}
