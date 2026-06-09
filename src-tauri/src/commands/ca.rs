use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use x509_parser::pem::parse_x509_pem;

use crate::config_store::ConfigStore;

#[derive(Debug, Serialize)]
pub struct CaStatus {
    pub has_cert: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub expires_at: Option<String>,
    pub is_trusted: bool,
}

fn get_ca_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile().ok_or("无法获取活跃 Profile")?;

    if let Some(ca_path) = profile.mitm.ca_path {
        Ok(PathBuf::from(ca_path))
    } else {
        app.path()
            .app_data_dir()
            .map_err(|e| e.to_string())
            .map(|d| d.join("ca"))
    }
}

fn parse_cert_expiration(cert_path: &PathBuf) -> Result<Option<String>, String> {
    let pem_content = fs::read_to_string(cert_path).map_err(|e| e.to_string())?;
    let (_, pem) = parse_x509_pem(pem_content.as_bytes()).map_err(|e| e.to_string())?;
    let x509 = pem.parse_x509().map_err(|e| e.to_string())?;
    let not_after = x509.validity().not_after;
    Ok(Some(not_after.to_string()))
}

#[tauri::command]
pub fn get_ca_status(app: AppHandle) -> Result<CaStatus, String> {
    let ca_dir = get_ca_dir(&app)?;
    let cert_path = ca_dir.join("cert.pem");
    let key_path = ca_dir.join("key.pem");
    let has_cert = cert_path.exists() && key_path.exists();

    let expires_at = if has_cert {
        parse_cert_expiration(&cert_path).ok().flatten()
    } else {
        None
    };

    let is_trusted = if has_cert {
        // Check macOS system keychain for certificate trust
        let result = std::process::Command::new("security")
            .args(["find-certificate", "-a", "-c", "Helio CA", "/Library/Keychains/System.keychain"])
            .output();
        result.map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).is_empty()).unwrap_or(false)
    } else {
        false
    };

    Ok(CaStatus {
        has_cert,
        cert_path: if has_cert {
            Some(cert_path.to_string_lossy().to_string())
        } else {
            None
        },
        key_path: if has_cert {
            Some(key_path.to_string_lossy().to_string())
        } else {
            None
        },
        expires_at,
        is_trusted,
    })
}

#[tauri::command]
pub fn generate_ca(app: AppHandle) -> Result<CaStatus, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile().ok_or("无法获取活跃 Profile")?;

    let ca_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("ca");

    fs::create_dir_all(&ca_dir).map_err(|e| e.to_string())?;

    let mut params = rcgen::CertificateParams::new(vec!["Helio CA".to_string()])
        .map_err(|e| e.to_string())?;
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let key_pair = rcgen::KeyPair::generate().map_err(|e| e.to_string())?;
    let cert = params.self_signed(&key_pair).map_err(|e| e.to_string())?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let cert_path = ca_dir.join("cert.pem");
    let key_path = ca_dir.join("key.pem");

    fs::write(&cert_path, cert_pem).map_err(|e| e.to_string())?;
    fs::write(&key_path, key_pem).map_err(|e| e.to_string())?;

    // Update profile
    let ca_dir_str = ca_dir.to_string_lossy().to_string();
    let cert_path_str = cert_path.to_string_lossy().to_string();
    let key_path_str = key_path.to_string_lossy().to_string();

    profile.mitm.ca_path = Some(ca_dir_str);
    profile.mitm.ca_cert = Some(cert_path_str);
    profile.mitm.ca_key = Some(key_path_str);
    store.save_profile(&profile)?;

    let expires_at = parse_cert_expiration(&cert_path).ok().flatten();

    Ok(CaStatus {
        has_cert: true,
        cert_path: Some(cert_path.to_string_lossy().to_string()),
        key_path: Some(key_path.to_string_lossy().to_string()),
        expires_at,
        is_trusted: false,
    })
}

#[tauri::command]
pub fn export_ca(app: AppHandle, format: String) -> Result<String, String> {
    let store = app.state::<ConfigStore>();
    let profile = store
        .get_active_profile()
        .ok_or("无法获取活跃 Profile")?;

    let ca_dir = profile
        .mitm
        .ca_path
        .map(PathBuf::from)
        .ok_or("CA 路径未配置")?;

    let cert_path = ca_dir.join("cert.pem");
    if !cert_path.exists() {
        return Err("证书文件不存在".to_string());
    }

    let export_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().desktop_dir())
        .map_err(|e| e.to_string())?;

    let export_path = match format.as_str() {
        "pem" => {
            let dest = export_dir.join("helio-ca-cert.pem");
            let content = fs::read_to_string(&cert_path).map_err(|e| e.to_string())?;
            fs::write(&dest, content).map_err(|e| e.to_string())?;
            dest
        }
        "der" => {
            let dest = export_dir.join("helio-ca-cert.der");
            let pem_content = fs::read_to_string(&cert_path).map_err(|e| e.to_string())?;
            let (_, pem) = parse_x509_pem(pem_content.as_bytes()).map_err(|e| e.to_string())?;
            fs::write(&dest, pem.contents).map_err(|e| e.to_string())?;
            dest
        }
        _ => return Err("不支持的导出格式".to_string()),
    };

    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn install_ca() -> Result<(), String> {
    Err("系统 CA 信任需要管理员权限，请在终端执行: sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain <cert_path>".to_string())
}
