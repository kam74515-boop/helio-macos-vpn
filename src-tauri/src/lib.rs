use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

struct AppState {
    engine_process: Mutex<Option<CommandChild>>,
}

#[tauri::command]
async fn start_engine(app: AppHandle, state: State<'_, AppState>, config: String) -> Result<(), String> {
    // Write config to app data dir
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    std::fs::create_dir_all(&config_dir).unwrap_or_default();
    let config_path = config_dir.join("config.json");
    
    if let Ok(mut file) = File::create(&config_path) {
        let _ = file.write_all(config.as_bytes());
    } else {
        return Err("Failed to write config.json".into());
    }

    // Stop existing process if any
    let mut current_process = state.engine_process.lock().unwrap();
    if let Some(child) = current_process.take() {
        let _ = child.kill();
    }

    // Launch sing-box sidecar
    let sidecar = app.shell().sidecar("sing-box").map_err(|e| e.to_string())?;
    
    let (mut rx, child) = sidecar
        .args(["run", "-c", config_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| e.to_string())?;

    // Store child
    *current_process = Some(child);

    // Optional: spawn a task to listen to logs
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                    println!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    eprintln!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_engine(state: State<'_, AppState>) -> Result<(), String> {
    let mut current_process = state.engine_process.lock().unwrap();
    if let Some(child) = current_process.take() {
        let _ = child.kill();
    }
    Ok(())
}

#[tauri::command]
async fn set_system_proxy(enable: bool) -> Result<(), String> {
    // Set macOS proxy (Wi-Fi interface as default for prototype)
    let state_str = if enable { "on" } else { "off" };
    
    std::process::Command::new("networksetup")
        .args(["-setsocksfirewallproxy", "Wi-Fi", "127.0.0.1", "6153", state_str])
        .output()
        .map_err(|e| e.to_string())?;
        
    std::process::Command::new("networksetup")
        .args(["-setwebproxy", "Wi-Fi", "127.0.0.1", "6152", state_str])
        .output()
        .map_err(|e| e.to_string())?;

    std::process::Command::new("networksetup")
        .args(["-setsecurewebproxy", "Wi-Fi", "127.0.0.1", "6152", state_str])
        .output()
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            engine_process: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            start_engine,
            stop_engine,
            set_system_proxy
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
